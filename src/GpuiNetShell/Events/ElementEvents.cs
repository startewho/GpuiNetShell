using System.Globalization;

namespace GpuiNetShell.Events;

/// <summary>
/// A `Div` element event the managed side can subscribe to. Only subscribed
/// events cross the ABI; the native host binds a GPUI listener only for those.
/// </summary>
public enum DivEvent
{
    Click,
    AuxClick,
    Hover,
    MouseDown,
    MouseUp,
    MouseDownOut,
    MouseUpOut,
    MouseMove,
    MousePressure,
    ScrollWheel,
    KeyDown,
    KeyUp,
}

/// <summary>Decodes the tab-separated payload the native host sends.</summary>
internal static class EventPayload
{
    internal static string[] Fields(string? payload) =>
        string.IsNullOrEmpty(payload) ? [] : payload.Split('\t');

    internal static double Number(string[] fields, int index) =>
        index < fields.Length
        && double.TryParse(fields[index], NumberStyles.Float, CultureInfo.InvariantCulture, out var value)
            ? value
            : 0;

    internal static string Text(string[] fields, int index) =>
        index < fields.Length ? fields[index] : string.Empty;
}

/// <summary>A mouse pointer event: position, button, and modifiers.</summary>
public readonly struct PointerEvent
{
    internal PointerEvent(double x, double y, int button, int clickCount, InputModifiers modifiers)
    {
        X = x;
        Y = y;
        Button = button;
        ClickCount = clickCount;
        Modifiers = modifiers;
    }

    /// <summary>X position in the window, in pixels.</summary>
    public double X { get; }

    /// <summary>Y position in the window, in pixels.</summary>
    public double Y { get; }

    /// <summary>Mouse button: 0 left, 1 right, 2 middle, -1 none.</summary>
    public int Button { get; }

    /// <summary>The number of consecutive clicks.</summary>
    public int ClickCount { get; }

    public InputModifiers Modifiers { get; }

    internal static PointerEvent Decode(string? payload)
    {
        var fields = EventPayload.Fields(payload);
        return new(
            EventPayload.Number(fields, 0),
            EventPayload.Number(fields, 1),
            (int)EventPayload.Number(fields, 2),
            (int)EventPayload.Number(fields, 3),
            (InputModifiers)(uint)EventPayload.Number(fields, 4)
        );
    }
}

/// <summary>A click event, including non-primary ("aux") clicks.</summary>
public readonly struct ClickEvent
{
    internal ClickEvent(double x, double y, int button, int clickCount, InputModifiers modifiers)
    {
        X = x;
        Y = y;
        Button = button;
        ClickCount = clickCount;
        Modifiers = modifiers;
    }

    public double X { get; }

    public double Y { get; }

    public int Button { get; }

    public int ClickCount { get; }

    public InputModifiers Modifiers { get; }

    internal static ClickEvent Decode(string? payload)
    {
        var fields = EventPayload.Fields(payload);
        return new(
            EventPayload.Number(fields, 0),
            EventPayload.Number(fields, 1),
            (int)EventPayload.Number(fields, 2),
            (int)EventPayload.Number(fields, 3),
            (InputModifiers)(uint)EventPayload.Number(fields, 4)
        );
    }
}

/// <summary>A scroll-wheel event: pointer position and the wheel delta.</summary>
public readonly struct ScrollEvent
{
    internal ScrollEvent(double x, double y, double dx, double dy, InputModifiers modifiers)
    {
        X = x;
        Y = y;
        DeltaX = dx;
        DeltaY = dy;
        Modifiers = modifiers;
    }

    public double X { get; }

    public double Y { get; }

    public double DeltaX { get; }

    public double DeltaY { get; }

    public InputModifiers Modifiers { get; }

    internal static ScrollEvent Decode(string? payload)
    {
        var fields = EventPayload.Fields(payload);
        return new(
            EventPayload.Number(fields, 0),
            EventPayload.Number(fields, 1),
            EventPayload.Number(fields, 2),
            EventPayload.Number(fields, 3),
            (InputModifiers)(uint)EventPayload.Number(fields, 4)
        );
    }
}

/// <summary>A keyboard event.</summary>
public readonly struct KeyEvent
{
    internal KeyEvent(string key, InputModifiers modifiers, bool isHeld)
    {
        Key = key;
        Modifiers = modifiers;
        IsHeld = isHeld;
    }

    /// <summary>The key name.</summary>
    public string Key { get; }

    public InputModifiers Modifiers { get; }

    /// <summary>Whether the key is held down (key-down events only).</summary>
    public bool IsHeld { get; }

    internal static KeyEvent Decode(string? payload)
    {
        var fields = EventPayload.Fields(payload);
        return new(
            EventPayload.Text(fields, 0),
            (InputModifiers)(uint)EventPayload.Number(fields, 1),
            EventPayload.Number(fields, 2) != 0
        );
    }
}

/// <summary>A force-touch pressure event.</summary>
public readonly struct PressureEvent
{
    internal PressureEvent(double pressure, double x, double y, InputModifiers modifiers, int stage)
    {
        Pressure = pressure;
        X = x;
        Y = y;
        Modifiers = modifiers;
        Stage = stage;
    }

    public double Pressure { get; }

    public double X { get; }

    public double Y { get; }

    public InputModifiers Modifiers { get; }

    /// <summary>Pressure stage: 0 none, 1 normal, 2 force.</summary>
    public int Stage { get; }

    internal static PressureEvent Decode(string? payload)
    {
        var fields = EventPayload.Fields(payload);
        return new(
            EventPayload.Number(fields, 0),
            EventPayload.Number(fields, 1),
            EventPayload.Number(fields, 2),
            (InputModifiers)(uint)EventPayload.Number(fields, 3),
            (int)EventPayload.Number(fields, 4)
        );
    }
}
