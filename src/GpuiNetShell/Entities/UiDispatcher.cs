using System.Collections.Concurrent;

namespace GpuiNetShell.Entities;

/// <summary>
/// Marshals work onto the GPUI UI thread. <see cref="Context{T}.Spawn(Action{T, Context{T}})"/>
/// posts here; the application installs a wake callback that requests a repaint,
/// and the UI thread drains the queue at the start of each managed render.
/// </summary>
/// <remarks>
/// Entity state is single-threaded, so a completion callback must run on the UI
/// thread. <see cref="Post"/> may be called from any thread; the queued action
/// runs later, on the UI thread.
/// </remarks>
public static class UiDispatcher
{
    private static readonly ConcurrentQueue<Action> Queue = new();
    private static Action? _wake;

    /// <summary>Installs the callback that wakes the UI thread; set once by the application.</summary>
    public static void SetWake(Action wake)
    {
        ArgumentNullException.ThrowIfNull(wake);
        _wake = wake;
    }

    /// <summary>Queues <paramref name="action"/> and wakes the UI thread.</summary>
    public static void Post(Action action)
    {
        ArgumentNullException.ThrowIfNull(action);
        Queue.Enqueue(action);
        _wake?.Invoke();
    }

    /// <summary>The number of actions waiting to run.</summary>
    internal static int PendingCount => Queue.Count;

    /// <summary>
    /// Runs every queued action on the caller's thread. Exceptions are isolated
    /// so one failed task cannot abort the frame.
    /// </summary>
    internal static void Drain()
    {
        while (Queue.TryDequeue(out var action))
        {
            try
            {
                action();
            }
            catch
            {
                // A background task must never crash the render loop.
            }
        }
    }

    /// <summary>Test seam: drops queued work and the wake callback.</summary>
    internal static void ResetForTests()
    {
        while (Queue.TryDequeue(out _)) { }
        _wake = null;
    }
}
