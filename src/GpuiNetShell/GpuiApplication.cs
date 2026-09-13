using System.Runtime.ExceptionServices;
using System.Runtime.InteropServices;
using System.Text;
using GpuiNetShell.Elements;
using GpuiNetShell.Events;
using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell;

/// <summary>
/// Owns one native GPUI application and its root <see cref="View"/>. The native
/// host blocks inside <see cref="Run"/> until the window closes.
/// </summary>
public sealed class GpuiApplication
{
    private static readonly object Gate = new();
    private static readonly Dictionary<ulong, GpuiApplication> Instances = [];
    private static ulong _nextSession;

    private readonly ulong _sessionId;
    private readonly Func<View> _rootFactory;
    private readonly EventRegistry _events = new();
    private readonly RenderArena _arena = new();
    private readonly RenderArena _elementArena = new();

    private RenderContext _renderContext = null!;
    private View? _root;

    /// <summary>
    /// Draw a custom title bar instead of the native one. Set before
    /// <see cref="Run"/>; the application renders it via the theme's title bar.
    /// </summary>
    public bool UseCustomTitlebar { get; set; }

    public GpuiApplication(Func<View> rootFactory)
    {
        _rootFactory = rootFactory ?? throw new ArgumentNullException(nameof(rootFactory));
        _renderContext = new RenderContext(_arena, _events, Invalidate);
        lock (Gate)
        {
            _sessionId = ++_nextSession;
            Instances[_sessionId] = this;
        }
    }

    internal static GpuiApplication? Find(ulong sessionId)
    {
        lock (Gate)
        {
            return Instances.TryGetValue(sessionId, out var application) ? application : null;
        }
    }

    /// <summary>Repaints every live session. Called after a Hot Reload update.</summary>
    internal static void InvalidateAll()
    {
        GpuiApplication[] applications;
        lock (Gate)
        {
            applications = [.. Instances.Values];
        }
        foreach (var application in applications)
        {
            application.Invalidate();
        }
    }

    /// <summary>
    /// Drops managed render caches before a Hot Reload update. The element tree
    /// is rebuilt every frame, so there is nothing held here beyond the root
    /// view's own state; this is the seam for any future cached render state.
    /// </summary>
    internal static void ClearRenderCaches()
    {
        // Intentional no-op today.
    }

    internal int OnStarted() => NativeProtocol.StatusOk;

    internal int OnWindowClosed(int status) => status;

    /// <summary>
    /// Builds one description for <paramref name="generation"/> and publishes it
    /// into the arena. Event handlers registered here belong to that generation
    /// and are released when its snapshot is retired.
    /// </summary>
    internal unsafe int RenderInto(ulong generation, NativeArena* arena, uint* root)
    {
        _arena.Reset();
        _events.BeginGeneration(generation);

        _root ??= _rootFactory();
        _root.AttachInvalidator(Invalidate);

        _renderContext.BeginRender();
        var element = _root.RenderRoot(ref _renderContext);
        _renderContext.EndRender();

        *arena = _arena.Publish();
        *root = (uint)element.Index;
        return NativeProtocol.StatusOk;
    }

    internal int OnRenderCompleted(ulong generation, int status) => status;

    internal int OnClick(ulong token) =>
        _events.Dispatch(token) ? NativeProtocol.StatusOk : -1;

    /// <summary>
    /// Delivers a typed callback value (a boolean, number, or string) to the
    /// handler registered for <paramref name="token"/>.
    /// </summary>
    internal unsafe int OnInvoke(
        ulong token,
        uint kind,
        double number,
        byte* data,
        uint dataLength
    )
    {
        var value = kind switch
        {
            NativeProtocol.CallbackValueBoolean => EventValue.FromBoolean(number != 0),
            NativeProtocol.CallbackValueNumber => EventValue.FromNumber(number),
            NativeProtocol.CallbackValueString => EventValue.FromString(ReadUtf8(data, dataLength)),
            _ => EventValue.None,
        };
        return _events.DispatchValue(token, value) ? NativeProtocol.StatusOk : -1;
    }

    private static unsafe string ReadUtf8(byte* data, uint length)
    {
        if (data == null || length == 0)
        {
            return string.Empty;
        }
        return Marshal.PtrToStringUTF8((IntPtr)data, (int)length) ?? string.Empty;
    }

    /// <summary>
    /// Fills <paramref name="buffer"/> with the tab-separated rows a row
    /// provider returns for <paramref name="token"/>. Reports the required byte
    /// count so native code can retry with a larger buffer.
    /// </summary>
    internal unsafe int OnResolveRows(ulong token, byte* buffer, uint capacity, uint* outLen)
    {
        if (!_events.TryGetRows(token, out var provider))
        {
            return -1;
        }
        var bytes = Encoding.UTF8.GetBytes(provider() ?? string.Empty);
        *outLen = (uint)bytes.Length;
        if (bytes.Length > capacity)
        {
            return NativeProtocol.StatusTruncated;
        }
        if (bytes.Length > 0)
        {
            Marshal.Copy(bytes, 0, (IntPtr)buffer, bytes.Length);
        }
        return NativeProtocol.StatusOk;
    }

    /// <summary>
    /// Renders one managed subtree for an element callback (P7) into a scratch
    /// arena. The native host decodes the arena and materializes it before this
    /// call returns.
    /// </summary>
    internal unsafe int OnRenderElement(
        ulong token,
        byte* arguments,
        uint argumentsLength,
        NativeArena* outArena,
        uint* outRoot
    )
    {
        if (!_events.TryGetElement(token, out var renderer))
        {
            return -1;
        }
        var text = ReadUtf8(arguments, argumentsLength);
        var args = text.Length == 0 ? Array.Empty<string>() : text.Split('\n');

        _elementArena.Reset();
        var context = new RenderContext(_elementArena, _events, Invalidate);
        context.BeginRender();
        Element root;
        try
        {
            root = renderer(context, args);
        }
        finally
        {
            context.EndRender();
        }
        *outArena = _elementArena.Publish();
        *outRoot = (uint)root.Index;
        return NativeProtocol.StatusOk;
    }

    /// <summary>Delivers one window input event to the managed view.</summary>
    internal unsafe int OnInputEvent(
        uint kind,
        uint flags,
        float a,
        float b,
        float c,
        byte* text,
        uint textLength
    )
    {
        var name = ReadUtf8(text, textLength);
        var input = new InputEvent((InputEventKind)kind, (InputModifiers)flags, a, b, c, name);
        _root?.DispatchInput(input);
        return NativeProtocol.StatusOk;
    }

    /// <summary>Releases the event handlers of a retired snapshot generation.</summary>
    internal int OnRetireCallbacks(ulong generation)
    {
        _events.Retire(generation);
        return NativeProtocol.StatusOk;
    }

    /// <summary>
    /// Requests a re-render from any thread. Equivalent to shell's
    /// <c>cx.notify()</c>: the native view marks itself dirty and repaints.
    /// </summary>
    public unsafe void Invalidate()
    {
        var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        if (api == null || api->Invalidate == null)
        {
            return;
        }
        _ = api->Invalidate(_sessionId);
    }

    /// <summary>
    /// Applies a theme at runtime. <paramref name="colors"/> maps semantic names
    /// (<c>background</c>, <c>foreground</c>, <c>primary</c>, <c>border</c>, …)
    /// to <c>#rrggbb</c> values; pass <see langword="null"/> to clear overrides.
    /// </summary>
    public unsafe void SetTheme(
        ThemeMode mode,
        IReadOnlyDictionary<string, string>? colors = null
    )
    {
        var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        if (api == null || api->SetTheme == null)
        {
            return;
        }
        var text =
            colors is null || colors.Count == 0
                ? string.Empty
                : string.Join('\n', colors.Select(pair => $"{pair.Key}={pair.Value}"));
        var bytes = Encoding.UTF8.GetBytes(text);
        fixed (byte* pointer = bytes)
        {
            _ = api->SetTheme(_sessionId, (uint)mode, pointer, (uint)bytes.Length);
        }
    }

    /// <summary>Runs the native application event loop. Blocking.</summary>
    public void Run() => RunOnUiThread(RunCore);

    private unsafe void RunCore()
    {
        var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        if (api == null)
        {
            throw new InvalidOperationException(
                "The native gpui-net-shell host rejected the requested ABI version."
            );
        }
        if (api->SchemaHash != NativeProtocol.SchemaHash)
        {
            throw new InvalidOperationException(
                "The managed and native schema hashes do not match; rebuild both."
            );
        }

        if (api->Configure != null)
        {
            _ = api->Configure(_sessionId, UseCustomTitlebar ? 1u : 0u);
        }

        var callbacks = ManagedCallbacks.Create();
        var status = api->RunApplication(_sessionId, &callbacks);
        if (status != NativeProtocol.StatusOk)
        {
            throw new InvalidOperationException($"The native host exited with status {status}.");
        }
    }

    /// <summary>
    /// Runs the event loop on a single-threaded-apartment thread. GPUI's
    /// Windows platform initializes OLE as STA; a managed entry thread is
    /// already MTA, which fails with <c>RPC_E_CHANGED_MODE</c>. Other platforms
    /// run in place.
    /// </summary>
    private static void RunOnUiThread(Action run)
    {
        if (
            !OperatingSystem.IsWindows()
            || Thread.CurrentThread.GetApartmentState() == ApartmentState.STA
        )
        {
            run();
            return;
        }

        Exception? failure = null;
        var uiThread = new Thread(() =>
        {
            try
            {
                run();
            }
            catch (Exception exception)
            {
                failure = exception;
            }
        })
        {
            IsBackground = false,
            Name = "GPUI UI",
        };
        uiThread.SetApartmentState(ApartmentState.STA);
        uiThread.Start();
        uiThread.Join();

        if (failure is not null)
        {
            ExceptionDispatchInfo.Capture(failure).Throw();
        }
    }
}
