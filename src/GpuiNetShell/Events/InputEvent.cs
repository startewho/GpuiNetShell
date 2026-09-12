using GpuiNetShell.Interop;

namespace GpuiNetShell.Events;

/// <summary>The kind of window input event.</summary>
public enum InputEventKind : uint
{
    KeyDown = 0,
    KeyUp = 1,
    MouseDown = 2,
    MouseUp = 3,
    MouseMove = 4,
    Scroll = 5,
}

/// <summary>Modifier keys held during an input event.</summary>
[Flags]
public enum InputModifiers : uint
{
    None = 0,
    Shift = NativeProtocol.ModifierShift,
    Control = NativeProtocol.ModifierControl,
    Alt = NativeProtocol.ModifierAlt,
    Platform = NativeProtocol.ModifierPlatform,
}

/// <summary>
/// One window input event delivered to a <see cref="View"/>: mouse, wheel, or
/// keyboard. Coordinates and deltas are in pixels.
/// </summary>
public readonly struct InputEvent
{
    internal InputEvent(
        InputEventKind kind,
        InputModifiers modifiers,
        float x,
        float y,
        float button,
        string key
    )
    {
        Kind = kind;
        Modifiers = modifiers;
        X = x;
        Y = y;
        Button = (int)button;
        Key = key;
    }

    public InputEventKind Kind { get; }

    public InputModifiers Modifiers { get; }

    /// <summary>X position (mouse) or horizontal wheel delta (scroll).</summary>
    public float X { get; }

    /// <summary>Y position (mouse) or vertical wheel delta (scroll).</summary>
    public float Y { get; }

    /// <summary>Mouse button: 0 left, 1 right, 2 middle.</summary>
    public int Button { get; }

    /// <summary>Key name for keyboard events; empty otherwise.</summary>
    public string Key { get; }

    public override string ToString() =>
        Kind switch
        {
            InputEventKind.KeyDown or InputEventKind.KeyUp => $"{Kind} {Key} [{Modifiers}]",
            InputEventKind.Scroll => $"{Kind} ({X:0.#}, {Y:0.#})",
            _ => $"{Kind} button {Button} at ({X:0.#}, {Y:0.#})",
        };
}
