namespace GpuiNetShell.Entities;

/// <summary>
/// The entity-scoped API handed to an <c>update</c>/<c>observe</c> callback:
/// access to the owning entity, notification (<see cref="Notify"/>), typed
/// events (<see cref="Emit{TEvent}"/>), and subscriptions. Mirrors GPUI's
/// <c>Context&lt;T&gt;</c>.
/// </summary>
public sealed class Context<T>
    where T : class
{
    private readonly EntityRegistry _registry;

    internal Context(EntityRegistry registry, ulong id)
    {
        _registry = registry;
        Entity = new Entity<T>(registry, id);
    }

    /// <summary>A handle to the entity this context is bound to.</summary>
    public Entity<T> Entity { get; }

    /// <summary>
    /// Marks this entity's state changed: its observers run, and the part of the
    /// tree rendered from it is repainted.
    /// </summary>
    public void Notify() => _registry.Notify(Entity.Id.Value);

    /// <summary>Emits a typed event to every subscriber of this entity.</summary>
    public void Emit<TEvent>(TEvent value) => _registry.Emit(Entity.Id.Value, value);

    /// <summary>
    /// Runs <paramref name="onNotify"/> whenever <paramref name="target"/>
    /// notifies, with this entity's state. This is the "state changed" channel.
    /// </summary>
    public Subscription Observe<TTarget>(
        Entity<TTarget> target,
        Action<T, Entity<TTarget>, Context<T>> onNotify
    )
        where TTarget : class
    {
        ArgumentNullException.ThrowIfNull(target);
        return _registry.Observe<T, TTarget>(Entity.Id.Value, target.Id.Value, onNotify);
    }

    /// <summary>
    /// Runs <paramref name="onEvent"/> whenever <paramref name="target"/> emits a
    /// <typeparamref name="TEvent"/>. This is the typed-event channel.
    /// </summary>
    public Subscription Subscribe<TTarget, TEvent>(
        Entity<TTarget> target,
        Action<T, Entity<TTarget>, TEvent, Context<T>> onEvent
    )
        where TTarget : class
    {
        ArgumentNullException.ThrowIfNull(target);
        return _registry.Subscribe<T, TTarget, TEvent>(
            Entity.Id.Value,
            target.Id.Value,
            onEvent
        );
    }

    /// <summary>Releases this entity and its subscriptions.</summary>
    public void Release() => _registry.Release(Entity.Id.Value);

    // -- Global state ------------------------------------------------------

    /// <summary>Reads a process-wide global value, or <see langword="null"/> when unset.</summary>
    public TGlobal? TryGlobal<TGlobal>()
        where TGlobal : class => GlobalStore.Default.Get<TGlobal>();

    /// <summary>Sets a process-wide global value; observers repaint.</summary>
    public void SetGlobal<TGlobal>(TGlobal value)
        where TGlobal : class => GlobalStore.Default.Set(value);

    // -- Spawn -------------------------------------------------------------

    /// <summary>
    /// Runs <paramref name="action"/> on the UI thread with this entity's
    /// current state. Safe to call from any thread.
    /// </summary>
    public void Spawn(Action<T, Context<T>> action)
    {
        ArgumentNullException.ThrowIfNull(action);
        var id = Entity.Id.Value;
        UiDispatcher.Post(() =>
        {
            if (!_registry.IsAlive(id))
            {
                return;
            }
            action(_registry.Read<T>(id), new Context<T>(_registry, id));
        });
    }

    /// <summary>
    /// Runs <paramref name="work"/> on the thread pool, then applies its result
    /// on the UI thread with this entity's current state. <paramref name="onError"/>
    /// runs off the UI thread when <paramref name="work"/> throws.
    /// </summary>
    public void Spawn<TResult>(
        Func<CancellationToken, Task<TResult>> work,
        Action<T, TResult, Context<T>> apply,
        Action<Exception>? onError = null
    )
    {
        ArgumentNullException.ThrowIfNull(work);
        ArgumentNullException.ThrowIfNull(apply);
        var id = Entity.Id.Value;
        _ = Task.Run(async () =>
        {
            TResult result;
            try
            {
                result = await work(CancellationToken.None).ConfigureAwait(false);
            }
            catch (Exception exception)
            {
                onError?.Invoke(exception);
                return;
            }
            UiDispatcher.Post(() =>
            {
                if (!_registry.IsAlive(id))
                {
                    return;
                }
                apply(_registry.Read<T>(id), result, new Context<T>(_registry, id));
            });
        });
    }
}
