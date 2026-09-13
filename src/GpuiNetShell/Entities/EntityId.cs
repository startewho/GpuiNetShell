namespace GpuiNetShell.Entities;

/// <summary>
/// A stable identity for one <see cref="Entity{T}"/>. The value is handed to the
/// native host as a retained key, so it is unique for the life of the process
/// and never reused.
/// </summary>
public readonly struct EntityId : IEquatable<EntityId>
{
    internal EntityId(ulong value) => Value = value;

    public ulong Value { get; }

    public bool Equals(EntityId other) => Value == other.Value;

    public override bool Equals(object? obj) => obj is EntityId other && Equals(other);

    public override int GetHashCode() => Value.GetHashCode();

    public override string ToString() => Value.ToString(System.Globalization.CultureInfo.InvariantCulture);

    public static bool operator ==(EntityId left, EntityId right) => left.Equals(right);

    public static bool operator !=(EntityId left, EntityId right) => !left.Equals(right);
}
