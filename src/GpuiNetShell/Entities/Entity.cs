namespace GpuiNetShell.Entities;

/// <summary>
/// A typed handle to a state object owned by the <see cref="EntityRegistry"/>.
/// The handle carries only an id and a type tag; the state is reached through an
/// <see cref="EntityRegistry"/> lookup, mirroring GPUI's <c>Entity&lt;T&gt;</c>.
/// </summary>
public sealed class Entity<T> : IEquatable<Entity<T>>
    where T : class
{
    private readonly EntityRegistry _registry;

    internal Entity(EntityRegistry registry, ulong id)
    {
        _registry = registry;
        Id = new EntityId(id);
    }

    public EntityId Id { get; }

    public bool IsAlive => _registry.IsAlive(Id.Value);

    /// <summary>Reads the state immutably.</summary>
    public T Read(out Context<T> cx)
    {
        var context = new Context<T>(_registry, Id.Value);
        cx = context;
        return _registry.Read<T>(Id.Value);
    }

    /// <summary>Reads the state immutably, discarding the context.</summary>
    public T Read() => _registry.Read<T>(Id.Value);

    /// <summary>Mutates the state, returning a result.</summary>
    public R Update<R>(Func<T, Context<T>, R> update)
    {
        ArgumentNullException.ThrowIfNull(update);
        return _registry.Update(Id.Value, update);
    }

    /// <summary>Mutates the state.</summary>
    public void Update(Action<T, Context<T>> update)
    {
        ArgumentNullException.ThrowIfNull(update);
        _ = _registry.Update<T, bool>(
            Id.Value,
            (state, context) =>
            {
                update(state, context);
                return true;
            }
        );
    }

    /// <summary>A non-owning handle; <see cref="WeakEntity{T}.Upgrade"/> fails once released.</summary>
    public WeakEntity<T> Downgrade() => new(_registry, Id.Value);

    public bool Equals(Entity<T>? other) => other is not null && other.Id == Id;

    public override bool Equals(object? obj) => obj is Entity<T> other && Equals(other);

    public override int GetHashCode() => Id.GetHashCode();

    public override string ToString() => $"Entity<{typeof(T).Name}>({Id.Value})";
}

/// <summary>A weak handle to an entity, safe to hold without keeping it alive.</summary>
public readonly struct WeakEntity<T>
    where T : class
{
    private readonly EntityRegistry _registry;
    private readonly ulong _id;

    internal WeakEntity(EntityRegistry registry, ulong id)
    {
        _registry = registry;
        _id = id;
    }

    /// <summary>The entity, or <see langword="null"/> once released.</summary>
    public Entity<T>? Upgrade() => _registry.IsAlive(_id) ? new Entity<T>(_registry, _id) : null;
}
