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
    // Parameterless handlers live apart so `Register(Action)` stores the
    // delegate directly instead of allocating a wrapper closure per call.
    private readonly Dictionary<ulong, Action> _actions = [];
    private readonly Dictionary<ulong, Func<string>> _rowProviders = [];
    private readonly Dictionary<
        ulong,
        Func<RenderContext, IReadOnlyList<string>, Element>
    > _elementRenderers = [];
    private readonly Dictionary<ulong, List<ulong>> _generations = [];

    // Retired token lists are pooled: a frame reuses the list retired a frame
    // earlier instead of allocating a new one.
    private readonly Stack<List<ulong>> _tokenPool = [];

    // Tokens registered while rendering one element callback (an entity subtree
    // or a row/cell). The native host replaces that subtree's handlers on the
    // next render, so the previous invocation's tokens are retired then. Without
    // this, every entity repaint would append handlers to the generation until
    // it retired.
    private readonly Dictionary<ulong, List<ulong>> _callbackScopes = [];
    private readonly Stack<(ulong Key, List<ulong> Tokens)> _scopeStack = [];
    // Retired scope token lists are pooled like generation lists.
    private readonly Stack<List<ulong>> _scopePool = [];

    // An entity-backed subtree is retained natively, so its renderer is not
    // generation-scoped: it is keyed by entity id and replaced in place.
    private readonly Dictionary<ulong, ulong> _persistentElements = [];
    // Entity-view renderers are keyed by a stable name (the generated callback
    // key) so re-registering the same view replaces its token.
    private readonly Dictionary<string, ulong> _entityViews = [];
    // Generation-independent handler tokens keyed by a stable name.
    private readonly Dictionary<string, ulong> _stableTokens = [];
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
        var tokens = _tokenPool.Count > 0 ? _tokenPool.Pop() : [];
        tokens.Clear();
        _generations[generation] = tokens;
    }

    /// <summary>Registers a parameterless handler and returns its never-reused token.</summary>
    public ulong Register(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = _next++;
        _actions[token] = handler;
        if (_generations.TryGetValue(_generation, out var tokens))
        {
            tokens.Add(token);
        }
        TrackScope(token);
        return token;
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
        TrackScope(token);
        return token;
    }

    // -- Stable (generation-independent) registration ----------------------
    //
    // Source-generated callbacks register once, not every frame. Their tokens
    // must therefore outlive snapshot generations, so they are keyed by a
    // stable name and never retired by `Retire`.

    /// <summary>Registers a stable parameterless handler, replacing by key.</summary>
    public ulong RegisterStable(string key, Action handler)
    {
        ArgumentException.ThrowIfNullOrEmpty(key);
        ArgumentNullException.ThrowIfNull(handler);
        if (_stableTokens.TryGetValue(key, out var existing))
        {
            _actions[existing] = handler;
            return existing;
        }
        var token = _next++;
        _actions[token] = handler;
        _stableTokens[key] = token;
        return token;
    }

    /// <summary>Registers a stable typed handler, replacing by key.</summary>
    public ulong RegisterStable(string key, Action<EventValue> handler)
    {
        ArgumentException.ThrowIfNullOrEmpty(key);
        ArgumentNullException.ThrowIfNull(handler);
        if (_stableTokens.TryGetValue(key, out var existing))
        {
            _handlers[existing] = handler;
            return existing;
        }
        var token = _next++;
        _handlers[token] = handler;
        _stableTokens[key] = token;
        return token;
    }

    /// <summary>Registers a stable row-snapshot provider, replacing by key.</summary>
    public ulong RegisterStableRows(string key, Func<string> provider)
    {
        ArgumentException.ThrowIfNullOrEmpty(key);
        ArgumentNullException.ThrowIfNull(provider);
        if (_stableTokens.TryGetValue(key, out var existing))
        {
            _rowProviders[existing] = provider;
            return existing;
        }
        var token = _next++;
        _rowProviders[token] = provider;
        _stableTokens[key] = token;
        return token;
    }

    /// <summary>Registers a stable element renderer, replacing by key.</summary>
    public ulong RegisterStableElement(
        string key,
        Func<RenderContext, IReadOnlyList<string>, Element> renderer
    )
    {
        ArgumentException.ThrowIfNullOrEmpty(key);
        ArgumentNullException.ThrowIfNull(renderer);
        if (_stableTokens.TryGetValue(key, out var existing))
        {
            _elementRenderers[existing] = renderer;
            return existing;
        }
        var token = _next++;
        _elementRenderers[token] = renderer;
        _stableTokens[key] = token;
        return token;
    }

    // -- Element-callback scope -------------------------------------------

    /// <summary>
    /// Starts a scope for one element-callback invocation, retiring the tokens
    /// the previous invocation of the same callback registered. The native host
    /// rebuilds that subtree from the new arena, so the old handlers are dead.
    /// </summary>
    public void BeginCallbackScope(ulong token)
    {
        ForgetCallbackScope(token);
        var tokens = _scopePool.Count > 0 ? _scopePool.Pop() : [];
        tokens.Clear();
        _scopeStack.Push((token, tokens));
    }

    /// <summary>Ends the current element-callback scope.</summary>
    public void EndCallbackScope()
    {
        if (_scopeStack.Count == 0)
        {
            return;
        }
        var (key, tokens) = _scopeStack.Pop();
        _callbackScopes[key] = tokens;
    }

    /// <summary>Records <paramref name="token"/> in the active scope, if any.</summary>
    private void TrackScope(ulong token)
    {
        if (_scopeStack.Count > 0)
        {
            _scopeStack.Peek().Tokens.Add(token);
        }
    }

    /// <summary>Retires and forgets the tokens of the scope keyed by <paramref name="token"/>.</summary>
    private void ForgetCallbackScope(ulong token)
    {
        if (_callbackScopes.Remove(token, out var tokens))
        {
            Release(tokens);
            if (_scopePool.Count < 16)
            {
                tokens.Clear();
                _scopePool.Push(tokens);
            }
        }
    }

    /// <summary>Removes tokens from every callback table.</summary>
    private void Release(IEnumerable<ulong> tokens)
    {
        foreach (var token in tokens)
        {
            _handlers.Remove(token);
            _actions.Remove(token);
            _rowProviders.Remove(token);
            _elementRenderers.Remove(token);
        }
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
        TrackScope(token);
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
        TrackScope(token);
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
        // Reuse the token for a re-registration. A retained subtree keeps the
        // token it was built with, so replacing it would strand that subtree.
        // Its previous invocation's handlers are dead now, though.
        if (_persistentElements.TryGetValue(entityId, out var existing))
        {
            _elementRenderers[existing] = renderer;
            ForgetCallbackScope(existing);
            return existing;
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
            ForgetCallbackScope(token);
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
        // Reuse the token for a re-registration; see RegisterPersistentElement.
        if (_entityViews.TryGetValue(key, out var existing))
        {
            _elementRenderers[existing] = renderer;
            ForgetCallbackScope(existing);
            return existing;
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

    /// <summary>Live handler count (test/diagnostics).</summary>
    internal int HandlerCount => _handlers.Count + _actions.Count;

    /// <summary>Live row-provider count (test/diagnostics).</summary>
    internal int RowProviderCount => _rowProviders.Count;

    /// <summary>Live element-renderer count (test/diagnostics).</summary>
    internal int ElementRendererCount => _elementRenderers.Count;

    /// <summary>Runs the handler for <paramref name="token"/>; false when retired.</summary>
    public bool Dispatch(ulong token) => DispatchValue(token, EventValue.None);

    /// <summary>
    /// Runs the handler for <paramref name="token"/> with <paramref name="value"/>;
    /// false when retired.
    /// </summary>
    public bool DispatchValue(ulong token, EventValue value)
    {
        if (_handlers.TryGetValue(token, out var handler))
        {
            handler(value);
            return true;
        }
        if (_actions.TryGetValue(token, out var action))
        {
            action();
            return true;
        }
        return false;
    }

    /// <summary>Retires every handler registered for a snapshot generation.</summary>
    public void Retire(ulong generation)
    {
        if (_generations.Remove(generation, out var tokens))
        {
            foreach (var token in tokens)
            {
                _handlers.Remove(token);
                _actions.Remove(token);
                _rowProviders.Remove(token);
                _elementRenderers.Remove(token);
                // A generation-scoped token may also have keyed a callback scope.
                ForgetCallbackScope(token);
            }
            // Reuse the (now empty) list for a future generation.
            if (_tokenPool.Count < 16)
            {
                tokens.Clear();
                _tokenPool.Push(tokens);
            }
        }
    }

    public void Clear()
    {
        _handlers.Clear();
        _actions.Clear();
        _rowProviders.Clear();
        _elementRenderers.Clear();
        _generations.Clear();
        _tokenPool.Clear();
        _persistentElements.Clear();
        _entityViews.Clear();
        _stableTokens.Clear();
        _callbackScopes.Clear();
        _scopeStack.Clear();
        _scopePool.Clear();
        _renderedEntities.Clear();
        _next = 1;
        _generation = 0;
    }
}
