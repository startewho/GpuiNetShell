namespace GpuiNetShell.Events;

/// <summary>
/// Maps stable callback tokens to managed handlers. Native code carries only the
/// token across the ABI.
/// </summary>
/// <remarks>
/// Handlers are grouped by the snapshot generation that registered them, so a
/// retired snapshot releases exactly its own handlers. Tokens are never reused,
/// so a click dispatched against the previous frame still resolves while that
/// snapshot is retained. Parameterless handlers (<c>on_click</c>) are stored
/// alongside typed ones; both share the token space.
/// </remarks>
public sealed class EventRegistry
{
    private readonly Dictionary<ulong, Action<EventValue>> _handlers = [];
    private readonly Dictionary<ulong, List<ulong>> _generations = [];
    private ulong _next = 1;
    private ulong _generation;

    /// <summary>Marks the snapshot generation new handlers belong to.</summary>
    public void BeginGeneration(ulong generation)
    {
        _generation = generation;
        _generations[generation] = [];
    }

    /// <summary>Registers a parameterless handler and returns its never-reused token.</summary>
    public ulong Register(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        return Register(_ => handler());
    }

    /// <summary>Registers a typed handler and returns its never-reused token.</summary>
    public ulong Register(Action<EventValue> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = _next++;
        _handlers[token] = handler;
        if (_generations.TryGetValue(_generation, out var tokens))
        {
            tokens.Add(token);
        }
        return token;
    }

    /// <summary>Runs the handler for <paramref name="token"/>; false when retired.</summary>
    public bool Dispatch(ulong token) => DispatchValue(token, EventValue.None);

    /// <summary>
    /// Runs the handler for <paramref name="token"/> with <paramref name="value"/>;
    /// false when retired.
    /// </summary>
    public bool DispatchValue(ulong token, EventValue value)
    {
        if (!_handlers.TryGetValue(token, out var handler))
        {
            return false;
        }
        handler(value);
        return true;
    }

    /// <summary>Retires every handler registered for a snapshot generation.</summary>
    public void Retire(ulong generation)
    {
        if (_generations.Remove(generation, out var tokens))
        {
            foreach (var token in tokens)
            {
                _handlers.Remove(token);
            }
        }
    }

    public void Clear()
    {
        _handlers.Clear();
        _generations.Clear();
        _next = 1;
        _generation = 0;
    }
}
