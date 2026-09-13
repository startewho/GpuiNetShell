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
}
