namespace GpuiNetShell.Entities;

/// <summary>
/// The lifetime of one observe/subscribe registration. Disposing it cancels the
/// subscription, mirroring GPUI's <c>Subscription</c>.
/// </summary>
public sealed class Subscription : IDisposable
{
    private Action? _cancel;

    internal Subscription(Action cancel) => _cancel = cancel;

    /// <summary>An inert subscription that cancels nothing.</summary>
    public static Subscription Empty { get; } = new(() => { });

    /// <summary>Cancels the subscription; safe to call more than once.</summary>
    public void Dispose()
    {
        var cancel = _cancel;
        _cancel = null;
        cancel?.Invoke();
    }
}
