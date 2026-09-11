namespace GpuiNetShell.Events;

/// <summary>The kind of value carried by a typed callback.</summary>
public enum EventValueKind : uint
{
    None = 0,
    Boolean = 1,
    Number = 2,
    String = 3,
}

/// <summary>
/// A value delivered to a typed callback, such as a radio's checked state or a
/// popover's open state.
/// </summary>
public readonly struct EventValue
{
    private EventValue(EventValueKind kind, bool boolean, double number, string? text)
    {
        Kind = kind;
        Boolean = boolean;
        Number = number;
        String = text;
    }

    public EventValueKind Kind { get; }

    public bool Boolean { get; }

    public double Number { get; }

    public string? String { get; }

    public static EventValue None => default;

    public static EventValue FromBoolean(bool value) =>
        new(EventValueKind.Boolean, value, value ? 1 : 0, null);

    public static EventValue FromNumber(double value) =>
        new(EventValueKind.Number, value != 0, value, null);

    public static EventValue FromString(string value) =>
        new(EventValueKind.String, !string.IsNullOrEmpty(value), 0, value);
}
