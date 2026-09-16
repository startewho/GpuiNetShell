using System.Runtime.ExceptionServices;
using System.Runtime.InteropServices;
using System.Text;
using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Events;
using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell;

/// <summary>
/// Owns one native GPUI application and its windows. The first window renders
/// the root <see cref="View"/> given to the constructor; additional windows are
/// opened with <see cref="OpenWindow"/>, each with its own view and session.
/// The native host blocks inside <see cref="Run"/> until the last window closes.
/// </summary>
public sealed class GpuiApplication
{
    private static readonly object Gate = new();
    private static readonly Dictionary<ulong, Session> Instances = [];
    private static ulong _nextSession;

    static GpuiApplication()
    {
        EntityRegistry.Notified += OnEntityNotified;
        GlobalStore.Changed += OnGlobalChanged;
        UiDispatcher.SetWake(InvalidateAll);
    }

    /// <summary>
    /// Bridges a managed entity notification to the native host: every live
    /// session currently rendering that entity repaints just its subtree.
    /// </summary>
    private static void OnEntityNotified(ulong entityId)
    {
        Session[] sessions;
        lock (Gate)
        {
            sessions = [.. Instances.Values];
        }
        foreach (var session in sessions)
        {
            session.NotifyEntity(entityId);
        }
    }

    /// <summary>Repaints every live session when a global value changes.</summary>
    private static void OnGlobalChanged(Type _) => InvalidateAll();

    private readonly Session _primary;
    private readonly List<Session> _children = [];

    /// <summary>
    /// Draw a custom title bar instead of the native one. Set before
    /// <see cref="Run"/>; the application renders it via the theme's title bar.
    /// </summary>
    public bool UseCustomTitlebar { get; set; }

    /// <summary>
    /// Keep scrollbars visible instead of auto-hiding them. Set before
    /// <see cref="Run"/>.
    /// </summary>
    public bool AlwaysShowScrollbars { get; set; }

    /// <summary>
    /// Creates an entity owned by the application, mirroring GPUI's
    /// <c>cx.new(|cx| ..)</c>. The <paramref name="init"/> closure runs once and
    /// may set up observers/subscriptions through its <see cref="Context{T}"/>.
    /// </summary>
    public Entity<T> New<T>(Func<Context<T>, T> init)
        where T : class => EntityRegistry.Default.Create(init);

    /// <summary>The process-wide entity registry behind <see cref="New{T}"/>.</summary>
    public EntityRegistry Entities => EntityRegistry.Default;

    /// <summary>
    /// Sets the process-wide global of type <typeparamref name="T"/>, mirroring
    /// GPUI's <c>cx.set_global</c>. Every live window repaints.
    /// </summary>
    public void SetGlobal<T>(T value)
        where T : class => GlobalStore.Default.Set(value);

    /// <summary>Reads the process-wide global of type <typeparamref name="T"/>; false when unset.</summary>
    public bool TryGetGlobal<T>(out T value)
        where T : class => GlobalStore.Default.TryGet(out value);

    /// <summary>The process-wide global of type <typeparamref name="T"/>, or <see langword="null"/> when unset.</summary>
    public T? Global<T>()
        where T : class => GlobalStore.Default.Get<T>();

    public GpuiApplication(Func<View> rootFactory)
    {
        ArgumentNullException.ThrowIfNull(rootFactory);
        _primary = new Session(this, rootFactory, resolveCustomTitlebar(null));
        lock (Gate)
        {
            _primary.SessionId = ++_nextSession;
            Instances[_primary.SessionId] = _primary;
        }
    }

    /// <summary>
    /// Resolves the title-bar mode of a window: an explicit per-window choice
    /// wins; otherwise the primary window's setting is inherited.
    /// </summary>
    private bool resolveCustomTitlebar(bool? explicitChoice) =>
        explicitChoice ?? UseCustomTitlebar;

    /// <summary>The primary window's session id.</summary>
    public ulong SessionId => _primary.SessionId;

    internal static Session? Find(ulong sessionId)
    {
        lock (Gate)
        {
            return Instances.TryGetValue(sessionId, out var session) ? session : null;
        }
    }

    /// <summary>
    /// Opens a new top-level window whose content is built by
    /// <paramref name="rootFactory"/>, inheriting the primary window's title-bar
    /// mode. The returned <see cref="WindowHandle"/> can invalidate the window
    /// and close it. Call before or during <see cref="Run"/>; a call before
    /// <see cref="Run"/> is queued until the event loop starts.
    /// </summary>
    public WindowHandle OpenWindow(Func<View> rootFactory) => OpenWindow(rootFactory, null);

    /// <summary>
    /// Opens a new top-level window with explicit <paramref name="options"/>.
    /// A <see langword="null"/> <see cref="WindowOptions.UseCustomTitlebar"/>
    /// inherits the primary window's title-bar mode.
    /// </summary>
    public unsafe WindowHandle OpenWindow(Func<View> rootFactory, WindowOptions? options)
    {
        ArgumentNullException.ThrowIfNull(rootFactory);
        var useCustomTitlebar = resolveCustomTitlebar(options?.UseCustomTitlebar);
        var session = new Session(this, rootFactory, useCustomTitlebar);
        lock (Gate)
        {
            _children.Add(session);
        }

        if (_running)
        {
            var opened = OpenNativeWindow(session);
            if (opened <= 0)
            {
                throw new InvalidOperationException(
                    $"The native host refused to open a window (status {opened})."
                );
            }
            session.SessionId = (ulong)opened;
            lock (Gate)
            {
                Instances[session.SessionId] = session;
            }
        }
        else
        {
            _pendingOpen.Add(session);
        }

        return new WindowHandle(this, session);
    }

    /// <summary>Opens one native window for an already-created session.</summary>
    private unsafe long OpenNativeWindow(Session session)
    {
        var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        if (api == null || api->OpenWindow == null)
        {
            throw new InvalidOperationException("The native host does not support opening windows.");
        }
        var flags = session.UseCustomTitlebar ? 1u : 0u;
        if (AlwaysShowScrollbars)
        {
            flags |= 2u;
        }
        return api->OpenWindow(_primary.SessionId, flags);
    }

    private readonly List<Session> _pendingOpen = [];
    private volatile bool _running;

    /// <summary>Closes a window previously opened by <see cref="OpenWindow"/>.</summary>
    internal unsafe void CloseWindow(Session session)
    {
        if (session.SessionId == 0)
        {
            _pendingOpen.Remove(session);
            return;
        }
        var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        if (api == null || api->CloseWindow == null)
        {
            return;
        }
        _ = api->CloseWindow(session.SessionId);
    }

    /// <summary>Repaints every live session. Called after a Hot Reload update.</summary>
    internal static void InvalidateAll()
    {
        ulong[] sessions;
        lock (Gate)
        {
            sessions = [.. Instances.Keys];
        }
        foreach (var session in sessions)
        {
            Find(session)?.Owner.InvalidateSession(session);
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

    internal void ForgetSession(ulong sessionId)
    {
        lock (Gate)
        {
            Instances.Remove(sessionId);
            _children.RemoveAll(session => session.SessionId == sessionId);
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
            var flags = 0u;
            _primary.UseCustomTitlebar = UseCustomTitlebar;
            if (UseCustomTitlebar)
            {
                flags |= 1u;
            }
            if (AlwaysShowScrollbars)
            {
                flags |= 2u;
            }
            _ = api->Configure(_primary.SessionId, flags);
        }

        _running = true;

        var callbacks = ManagedCallbacks.Create();
        var status = api->RunApplication(_primary.SessionId, &callbacks);
        if (status != NativeProtocol.StatusOk)
        {
            throw new InvalidOperationException($"The native host exited with status {status}.");
        }
    }

    /// <summary>Opens any windows queued before the event loop started.</summary>
    internal unsafe void FlushPendingWindows()
    {
        if (_pendingOpen.Count == 0)
        {
            return;
        }
        var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        if (api == null || api->OpenWindow == null)
        {
            return;
        }
        var pending = _pendingOpen.ToArray();
        _pendingOpen.Clear();
        foreach (var session in pending)
        {
            var opened = OpenNativeWindow(session);
            if (opened <= 0)
            {
                continue;
            }
            session.SessionId = (ulong)opened;
            lock (Gate)
            {
                Instances[session.SessionId] = session;
            }
        }
    }

    /// <summary>
    /// Requests a re-render of the primary window. For a secondary window, use
    /// its <see cref="WindowHandle.Invalidate"/>.
    /// </summary>
    public void Invalidate() => InvalidateSession(_primary.SessionId);

    internal void InvalidateSessionFor(Session session)
    {
        if (session.SessionId == 0)
        {
            return;
        }
        InvalidateSession(session.SessionId);
    }

    private unsafe void InvalidateSession(ulong sessionId)
    {
        GpuiNetShellApi* api;
        try
        {
            api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        }
        catch (DllNotFoundException)
        {
            // Best-effort: without a native host there is nothing to repaint.
            return;
        }
        catch (EntryPointNotFoundException)
        {
            return;
        }
        if (api == null || api->Invalidate == null)
        {
            return;
        }
        _ = api->Invalidate(sessionId);
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
            _ = api->SetTheme(_primary.SessionId, (uint)mode, pointer, (uint)bytes.Length);
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

    /// <summary>
    /// One window's managed state: its view, render arenas, and event registry.
    /// A session survives across frames and is released when its window closes.
    /// </summary>
    internal sealed class Session
    {
        private readonly EventRegistry _events = new();
        private readonly RenderArena _arena = new();
        private readonly RenderArena _elementArena = new();

        private RenderContext _renderContext = null!;
        private View? _root;

        internal Session(GpuiApplication owner, Func<View> rootFactory, bool useCustomTitlebar)
        {
            Owner = owner;
            RootFactory = rootFactory;
            UseCustomTitlebar = useCustomTitlebar;
            _renderContext = new RenderContext(_arena, _events, () => Owner.InvalidateSession(SessionId));
        }

        internal GpuiApplication Owner { get; }

        internal ulong SessionId { get; set; }

        internal Func<View> RootFactory { get; }

        /// <summary>This window's resolved title-bar mode.</summary>
        internal bool UseCustomTitlebar { get; set; }

        internal EventRegistry Events => _events;

        internal int OnStarted() => NativeProtocol.StatusOk;

        internal int OnWindowClosed(int status)
        {
            Owner.ForgetSession(SessionId);
            // The arenas own unmanaged buffers; release them with the window
            // rather than leaving them to a finalizer that may never run.
            _arena.Dispose();
            _elementArena.Dispose();
            return status;
        }

        /// <summary>
        /// Builds one description for <paramref name="generation"/> and publishes
        /// it into the arena. Event handlers registered here belong to that
        /// generation and are released when its snapshot is retired.
        /// </summary>
        internal unsafe int RenderInto(ulong generation, NativeArena* arena, uint* root)
        {
            // The primary window's first frame is the earliest point where the
            // native event loop (and its ingress) is live, so any window queued
            // before `Run` opens now.
            if (Owner._primary.SessionId == SessionId)
            {
                Owner.FlushPendingWindows();
            }

            // Background tasks post completions here; they run on the UI thread
            // before the tree is built so the frame reflects their state changes.
            UiDispatcher.Drain();

            _arena.Reset();
            _events.BeginGeneration(generation);

            _root ??= RootFactory();
            _root.AttachInvalidator(() => Owner.InvalidateSession(SessionId));

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
                NativeProtocol.CallbackValueString => EventValue.FromString(
                    ReadUtf8(data, dataLength)
                ),
                _ => EventValue.None,
            };
            return _events.DispatchValue(token, value) ? NativeProtocol.StatusOk : -1;
        }

        /// <summary>
        /// Fills <paramref name="buffer"/> with the tab-separated rows a row
        /// provider returns for <paramref name="token"/>. Reports the required
        /// byte count so native code can retry with a larger buffer.
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
        /// Renders one managed subtree for an element callback (P7) into a
        /// scratch arena. The native host decodes the arena and materializes it
        /// before this call returns.
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
            var context = new RenderContext(
                _elementArena,
                _events,
                () => Owner.InvalidateSession(SessionId)
            );
            context.BeginRender();
            // Scope the handlers this callback registers: the native host
            // replaces this subtree on the next invocation, so the previous
            // invocation's handlers are dead once it starts.
            _events.BeginCallbackScope(token);
            Element root;
            try
            {
                root = renderer(context, args);
            }
            finally
            {
                context.EndRender();
                _events.EndCallbackScope();
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
        /// Repaints one entity subtree, if this window currently renders it. The
        /// call is safe from any thread; the native host marshals it to the GPUI
        /// thread.
        /// </summary>
        internal unsafe void NotifyEntity(ulong entityId)
        {
            if (SessionId == 0 || !_events.RendersEntity(entityId))
            {
                return;
            }
            var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
            if (api == null || api->NotifyEntity == null)
            {
                return;
            }
            _ = api->NotifyEntity(SessionId, entityId);
        }
    }

    private static unsafe string ReadUtf8(byte* data, uint length)
    {
        if (data == null || length == 0)
        {
            return string.Empty;
        }
        return Marshal.PtrToStringUTF8((IntPtr)data, (int)length) ?? string.Empty;
    }
}

/// <summary>
/// A handle to a window opened with <see cref="GpuiApplication.OpenWindow"/>.
/// </summary>
public sealed class WindowHandle
{
    private readonly GpuiApplication _application;
    private readonly GpuiApplication.Session _session;

    internal WindowHandle(GpuiApplication application, GpuiApplication.Session session)
    {
        _application = application;
        _session = session;
    }

    /// <summary>The window's session id, or 0 until the window opens.</summary>
    public ulong SessionId => _session.SessionId;

    /// <summary>Requests a repaint of this window.</summary>
    public void Invalidate()
    {
        if (_session.SessionId != 0)
        {
            _application.InvalidateSessionFor(_session);
        }
    }

    /// <summary>Closes this window.</summary>
    public void Close() => _application.CloseWindow(_session);
}
