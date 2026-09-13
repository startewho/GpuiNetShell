using System.Runtime.CompilerServices;

namespace GpuiNetShell.Entities;

/// <summary>
/// Owns every live entity's state and their observe/subscribe relationships,
/// mirroring GPUI's rule that the <c>App</c> owns all entity state.
/// </summary>
/// <remarks>
/// Entities are accessed only from the UI thread. The registry stores the state
/// behind an <see cref="object"/> box so one table serves every entity type;
/// <see cref="Entity{T}"/> re-types it. A notification cascade is re-entrancy
/// guarded per entity so an observer that notifies back does not loop.
/// </remarks>
public sealed class EntityRegistry
{
    private static readonly object Gate = new();

    private static readonly EntityRegistry Shared = new();

    /// <summary>The process-wide registry.</summary>
    public static EntityRegistry Default => Shared;

    /// <summary>
    /// Raised after an entity notifies, so a window rendering that entity can
    /// repaint just its subtree. Native code is told through
    /// <c>notify_entity</c>; the hook is process-wide because one entity may be
    /// rendered by more than one session.
    /// </summary>
    internal static event Action<ulong>? Notified;

    private readonly Dictionary<ulong, Slot> _slots = [];
    private readonly Dictionary<ulong, List<ObserverEdge>> _observers = [];
    private readonly Dictionary<ulong, Dictionary<Type, List<EventEdge>>> _listeners = [];
    private readonly HashSet<ulong> _notifying = [];
    private ulong _next;

    /// <summary>Creates an entity, running <paramref name="init"/> once.</summary>
    public Entity<T> Create<T>(Func<Context<T>, T> init)
        where T : class
    {
        ArgumentNullException.ThrowIfNull(init);
        ulong id;
        lock (Gate)
        {
            id = ++_next;
        }
        var context = new Context<T>(this, id);
        var state = init(context);
        _slots[id] = new Slot(state, typeof(T));
        return new Entity<T>(this, id);
    }

    internal T Read<T>(ulong id)
        where T : class
    {
        if (_slots.TryGetValue(id, out var slot) && slot.State is T typed)
        {
            return typed;
        }
        throw new InvalidOperationException($"Entity {id} is not alive.");
    }

    internal bool TryRead<T>(ulong id, out T state)
        where T : class
    {
        if (_slots.TryGetValue(id, out var slot) && slot.State is T typed)
        {
            state = typed;
            return true;
        }
        state = null!;
        return false;
    }

    internal R Update<T, R>(ulong id, Func<T, Context<T>, R> update)
        where T : class
    {
        var state = Read<T>(id);
        var context = new Context<T>(this, id);
        return update(state, context);
    }

    internal bool IsAlive(ulong id) => _slots.ContainsKey(id);

    internal void Release(ulong id)
    {
        _slots.Remove(id);
        _observers.Remove(id);
        _listeners.Remove(id);
        // Drop edges *other* entities hold toward this one.
        foreach (var edges in _observers.Values)
        {
            edges.RemoveAll(edge => edge.Target == id);
        }
        foreach (var byType in _listeners.Values)
        {
            foreach (var edges in byType.Values)
            {
                edges.RemoveAll(edge => edge.Target == id);
            }
        }
    }

    // -- Observe -----------------------------------------------------------

    internal Subscription Observe<TObserver, TTarget>(
        ulong observer,
        ulong target,
        Action<TObserver, Entity<TTarget>, Context<TObserver>> onNotify
    )
        where TObserver : class
        where TTarget : class
    {
        ArgumentNullException.ThrowIfNull(onNotify);
        var edge = new ObserverEdge(
            observer,
            target,
            () =>
            {
                var state = Read<TObserver>(observer);
                var context = new Context<TObserver>(this, observer);
                onNotify(state, new Entity<TTarget>(this, target), context);
            }
        );
        if (!_observers.TryGetValue(target, out var edges))
        {
            edges = [];
            _observers[target] = edges;
        }
        edges.Add(edge);
        return new Subscription(() => edges.Remove(edge));
    }

    // -- Subscribe / Emit --------------------------------------------------

    internal Subscription Subscribe<TObserver, TTarget, TEvent>(
        ulong observer,
        ulong target,
        Action<TObserver, Entity<TTarget>, TEvent, Context<TObserver>> onEvent
    )
        where TObserver : class
        where TTarget : class
    {
        ArgumentNullException.ThrowIfNull(onEvent);
        var edge = new EventEdge(
            observer,
            target,
            typeof(TEvent),
            value =>
            {
                var state = Read<TObserver>(observer);
                var context = new Context<TObserver>(this, observer);
                onEvent(state, new Entity<TTarget>(this, target), (TEvent)value, context);
            }
        );
        if (!_listeners.TryGetValue(target, out var byType))
        {
            byType = [];
            _listeners[target] = byType;
        }
        if (!byType.TryGetValue(typeof(TEvent), out var edges))
        {
            edges = [];
            byType[typeof(TEvent)] = edges;
        }
        edges.Add(edge);
        return new Subscription(() => edges.Remove(edge));
    }

    internal void Emit<TEvent>(ulong emitter, TEvent value)
    {
        if (!_listeners.TryGetValue(emitter, out var byType))
        {
            return;
        }
        if (!byType.TryGetValue(typeof(TEvent), out var edges) || edges.Count == 0)
        {
            return;
        }
        // Copy so a handler that unsubscribes does not mutate the live list.
        foreach (var edge in edges.ToArray())
        {
            edge.Invoke(value!);
        }
    }

    // -- Notify ------------------------------------------------------------

    internal void Notify(ulong id)
    {
        if (!_notifying.Add(id))
        {
            // Re-entrant notify for the same entity: drop it rather than loop.
            return;
        }
        try
        {
            if (_observers.TryGetValue(id, out var edges) && edges.Count > 0)
            {
                foreach (var edge in edges.ToArray())
                {
                    edge.Invoke();
                }
            }
        }
        finally
        {
            _notifying.Remove(id);
        }

        // A window rendering this entity repaints just its subtree.
        Notified?.Invoke(id);
    }

    /// <summary>Test seam: drops all state. Not part of the public model.</summary>
    internal void ResetForTests()
    {
        _slots.Clear();
        _observers.Clear();
        _listeners.Clear();
        _notifying.Clear();
        lock (Gate)
        {
            _next = 0;
        }
    }

    private sealed record Slot(object State, Type EntityType);

    private sealed class ObserverEdge
    {
        private readonly Action _invoke;

        internal ObserverEdge(ulong observer, ulong target, Action invoke)
        {
            Observer = observer;
            Target = target;
            _invoke = invoke;
        }

        internal ulong Observer { get; }

        internal ulong Target { get; }

        internal void Invoke() => _invoke();
    }

    private sealed class EventEdge
    {
        private readonly Action<object> _invoke;

        internal EventEdge(ulong observer, ulong target, Type eventType, Action<object> invoke)
        {
            Observer = observer;
            Target = target;
            EventType = eventType;
            _invoke = invoke;
        }

        internal ulong Observer { get; }

        internal ulong Target { get; }

        internal Type EventType { get; }

        internal void Invoke(object value) => _invoke(value);
    }
}
