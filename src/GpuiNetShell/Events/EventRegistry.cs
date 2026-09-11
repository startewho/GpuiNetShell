namespace GpuiNetShell.Events;

/// <summary>
/// Maps stable callback tokens to managed click handlers. Native code carries
/// only the token across the ABI.
/// </summary>
public sealed class EventRegistry
{
    private readonly Dictionary<ulong, Action> _handlers = [];
    private ulong _next = 1;

    /// <summary>Registers a handler and returns its never-reused token.</summary>
    public ulong Register(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = _next++;
        _handlers[token] = handler;
        return token;
    }

    /// <summary>Runs the handler for <paramref name="token"/>; false when retired.</summary>
    public bool Dispatch(ulong token)
    {
        if (!_handlers.TryGetValue(token, out var handler))
        {
            return false;
        }
        handler();
        return true;
    }

    /// <summary>Retires all handlers but keeps tokens monotonic.</summary>
    public void Reset() => _handlers.Clear();

    public void Clear()
    {
        _handlers.Clear();
        _next = 1;
    }
}
