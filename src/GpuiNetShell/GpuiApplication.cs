using System.Runtime.ExceptionServices;
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

    private View? _root;
    private ulong _revision;

    public GpuiApplication(Func<View> rootFactory)
    {
        _rootFactory = rootFactory ?? throw new ArgumentNullException(nameof(rootFactory));
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

    internal int OnStarted() => NativeProtocol.StatusOk;

    internal int OnWindowClosed(int status) => status;

    internal unsafe int RenderInto(NativeArena* arena, uint* root, ulong* revision)
    {
        _arena.Reset();
        _events.Reset();

        var ui = new RenderContext(_arena, _events);
        _root ??= _rootFactory();
        var element = _root.RenderRoot(ref ui);

        *arena = _arena.Publish();
        *root = (uint)element.Index;
        *revision = ++_revision;
        return NativeProtocol.StatusOk;
    }

    internal int OnRenderCompleted(ulong revision, int status) => status;

    internal int OnClick(ulong token) =>
        _events.Dispatch(token) ? NativeProtocol.StatusOk : -1;

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
