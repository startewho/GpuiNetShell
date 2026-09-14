using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

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
    private readonly Dictionary<ulong, Func<string>> _rowProviders = [];
    private readonly Dictionary<
        ulong,
        Func<RenderContext, IReadOnlyList<string>, Element>
    > _elementRenderers = [];
    private readonly Dictionary<ulong, List<ulong>> _generations = [];

    // An entity-backed subtree is retained natively, so its renderer is not
    // generation-scoped: it is keyed by entity id and replaced in place.
    private readonly Dictionary<ulong, ulong> _persistentElements = [];
    // Entity-view renderers are keyed by a stable name (the generated callback
    // key) so re-registering the same view replaces its token.
    private readonly Dictionary<string, ulong> _entityViews = [];
    private readonly HashSet<ulong> _renderedEntities = [];
    private ulong _next = 1;
    private ulong _generation;

    /// <summary>Marks the snapshot generation new handlers belong to.</summary>
    public void BeginGeneration(ulong generation)
    {
        // An entity that was not rendered last generation has left the tree;
        // release its persistent renderer so the token table does not grow.
        List<ulong>? stale = null;
        foreach (var (entityId, token) in _persistentElements)
        {
            if (!_renderedEntities.Contains(entityId))
            {
                (stale ??= []).Add(entityId);
                _elementRenderers.Remove(token);
            }
        }
        if (stale is not null)
        {
            foreach (var entityId in stale)
            {
                _persistentElements.Remove(entityId);
            }
        }

        _generation = generation;
        _renderedEntities.Clear();
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

    /// <summary>
    /// Registers a row-snapshot provider — a callback the native host invokes
    /// to obtain tab-separated rows — and returns its never-reused token.
    /// </summary>
    public ulong RegisterRows(Func<string> provider)
    {
        ArgumentNullException.ThrowIfNull(provider);
        var token = _next++;
        _rowProviders[token] = provider;
        if (_generations.TryGetValue(_generation, out var tokens))
        {
            tokens.Add(token);
        }
        return token;
    }

    /// <summary>Resolves a row-snapshot provider; false when retired.</summary>
    public bool TryGetRows(ulong token, out Func<string> provider) =>
        _rowProviders.TryGetValue(token, out provider!);

    /// <summary>
    /// Registers an element renderer — a callback the native host invokes to
    /// build one subtree per row or cell (P7) — and returns its token.
    /// </summary>
    public ulong RegisterElement(
        Func<RenderContext, IReadOnlyList<string>, Element> renderer
    )
    {
        ArgumentNullException.ThrowIfNull(renderer);
        var token = _next++;
        _elementRenderers[token] = renderer;
        if (_generations.TryGetValue(_generation, out var tokens))
        {
            tokens.Add(token);
        }
        return token;
    }

    /// <summary>Resolves an element renderer; false when retired.</summary>
    public bool TryGetElement(
        ulong token,
        out Func<RenderContext, IReadOnlyList<string>, Element> renderer
    ) => _elementRenderers.TryGetValue(token, out renderer!);

    /// <summary>
    /// Registers the element renderer for an entity-backed subtree. The native
    /// host retains the returned token across frames, so it outlives snapshot
    /// generations; re-registering the same <paramref name="entityId"/> replaces
    /// the previous renderer (and releases its token).
    /// </summary>
    public ulong RegisterPersistentElement(
        ulong entityId,
        Func<RenderContext, IReadOnlyList<string>, Element> renderer
    )
    {
        ArgumentNullException.ThrowIfNull(renderer);
        if (_persistentElements.TryGetValue(entityId, out var previous))
        {
            _elementRenderers.Remove(previous);
        }
        var token = _next++;
        _elementRenderers[token] = renderer;
        _persistentElements[entityId] = token;
        return token;
    }

    /// <summary>Releases an entity's persistent element renderer.</summary>
    public void ReleasePersistentElement(ulong entityId)
    {
        if (_persistentElements.Remove(entityId, out var token))
        {
            _elementRenderers.Remove(token);
        }
    }

    /// <summary>
    /// Registers an entity-view renderer under a stable <paramref name="key"/>
    /// (the generated callback key), replacing any previous registration for
    /// that key. The renderer receives the native argument list, whose first
    /// entry is the entity id.
    /// </summary>
    public ulong RegisterEntityView(
        string key,
        Func<RenderContext, IReadOnlyList<string>, Element> renderer
    )
    {
        ArgumentException.ThrowIfNullOrEmpty(key);
        ArgumentNullException.ThrowIfNull(renderer);
        if (_entityViews.TryGetValue(key, out var previous))
        {
            _elementRenderers.Remove(previous);
        }
        var token = _next++;
        _elementRenderers[token] = renderer;
        _entityViews[key] = token;
        return token;
    }

    /// <summary>Notes that <paramref name="entityId"/> was rendered this generation.</summary>
    public void MarkEntityRendered(ulong entityId) => _renderedEntities.Add(entityId);

    /// <summary>Whether an entity subtree is present in the current generation.</summary>
    public bool RendersEntity(ulong entityId) => _renderedEntities.Contains(entityId);

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
                _rowProviders.Remove(token);
                _elementRenderers.Remove(token);
            }
        }
    }

    public void Clear()
    {
        _handlers.Clear();
        _rowProviders.Clear();
        _elementRenderers.Clear();
        _generations.Clear();
        _persistentElements.Clear();
        _entityViews.Clear();
        _renderedEntities.Clear();
        _next = 1;
        _generation = 0;
    }
}
