namespace GpuiNetShell.Entities;

/// <summary>
/// Process-wide typed state, mirroring GPUI's <c>Global</c>. Unlike an entity it
/// has no identity, ownership, or lifetime: there is exactly one value per type,
/// reachable from any <see cref="Context{T}"/>.
/// </summary>
/// <remarks>
/// Access is thread-safe so a background task may set a global, but the value
/// itself is meant to be read on the UI thread. A change raises
/// <see cref="Changed"/>; the application uses that to request a repaint.
/// </remarks>
public sealed class GlobalStore
{
    private static readonly GlobalStore Shared = new();

    /// <summary>The process-wide store.</summary>
    public static GlobalStore Default => Shared;

    /// <summary>Raised after a global of the given type is set or removed.</summary>
    internal static event Action<Type>? Changed;

    private readonly Dictionary<Type, object> _values = [];
    private readonly object _gate = new();

    /// <summary>Sets the global of type <typeparamref name="T"/>, replacing any previous value.</summary>
    public void Set<T>(T value)
        where T : class
    {
        ArgumentNullException.ThrowIfNull(value);
        lock (_gate)
        {
            _values[typeof(T)] = value;
        }
        Changed?.Invoke(typeof(T));
    }

    /// <summary>Reads the global of type <typeparamref name="T"/>; false when unset.</summary>
    public bool TryGet<T>(out T value)
        where T : class
    {
        lock (_gate)
        {
            if (_values.TryGetValue(typeof(T), out var boxed) && boxed is T typed)
            {
                value = typed;
                return true;
            }
        }
        value = null!;
        return false;
    }

    /// <summary>The global of type <typeparamref name="T"/>, or <see langword="null"/> when unset.</summary>
    public T? Get<T>()
        where T : class => TryGet<T>(out var value) ? value : null;

    /// <summary>Removes the global of type <typeparamref name="T"/>; false when it was unset.</summary>
    public bool Remove<T>()
        where T : class
    {
        bool removed;
        lock (_gate)
        {
            removed = _values.Remove(typeof(T));
        }
        if (removed)
        {
            Changed?.Invoke(typeof(T));
        }
        return removed;
    }

    /// <summary>Removes every global.</summary>
    public void Clear()
    {
        lock (_gate)
        {
            _values.Clear();
        }
    }

    /// <summary>Test seam: drops all values. Not part of the public model.</summary>
    internal void ResetForTests() => Clear();
}
