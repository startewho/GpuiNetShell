using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>An easing curve for a <see cref="MotionSpec"/>.</summary>
public enum Easing
{
    Linear,
    Ease,
    In,
    Out,
    InOut,
}

/// <summary>
/// How an animated value moves toward its target. P4 exposes timed
/// <see cref="Transition"/>; springs and keyframes follow later.
/// </summary>
public readonly struct MotionSpec
{
    internal int DurationMs { get; }

    internal Easing Easing { get; }

    private MotionSpec(int durationMs, Easing easing)
    {
        DurationMs = durationMs;
        Easing = easing;
    }

    /// <summary>A timed transition with an easing curve.</summary>
    public static MotionSpec Transition(TimeSpan duration, Easing easing = Easing.Out) =>
        new((int)Math.Max(0, duration.TotalMilliseconds), easing);

    internal static string Wire(Easing easing) =>
        easing switch
        {
            Easing.Linear => "linear",
            Easing.Ease => "ease",
            Easing.In => "ease_in",
            Easing.InOut => "ease_in_out",
            _ => "ease_out",
        };
}

/// <summary>
/// An animated container. Its animated values are targets uploaded each render;
/// the native host interpolates from the current value to the new target and
/// requests frames while moving. Translation is a visual offset (the wrapper is
/// relatively positioned), so it does not affect layout.
/// </summary>
public sealed class MotionElement : Element
{
    internal MotionElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Animates opacity toward <paramref name="target"/> (0–1).</summary>
    public MotionElement Opacity(double target)
    {
        Arena.AddMethodNumber(Index, "opacity", target);
        return this;
    }

    /// <summary>Animates a horizontal offset, in pixels.</summary>
    public MotionElement TranslateX(double pixels)
    {
        Arena.AddMethodNumber(Index, "translate_x", pixels);
        return this;
    }

    /// <summary>Animates a vertical offset, in pixels.</summary>
    public MotionElement TranslateY(double pixels)
    {
        Arena.AddMethodNumber(Index, "translate_y", pixels);
        return this;
    }

    /// <summary>Animates width, in pixels.</summary>
    public MotionElement Width(double pixels)
    {
        Arena.AddMethodNumber(Index, "width", pixels);
        return this;
    }

    /// <summary>Animates height, in pixels.</summary>
    public MotionElement Height(double pixels)
    {
        Arena.AddMethodNumber(Index, "height", pixels);
        return this;
    }

    /// <summary>Adds the animated content.</summary>
    public MotionElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>
/// A container that keeps its content mounted through its exit animation.
/// Set its present flag each render; the native host animates opacity and a
/// vertical offset between the from/to ranges while mounting or unmounting.
/// </summary>
public sealed class PresenceElement : Element
{
    internal PresenceElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Animates the entry/exit opacity range.</summary>
    public PresenceElement Fade(double from, double to)
    {
        Arena.AddMethodNumber(Index, "fade_from", from);
        Arena.AddMethodNumber(Index, "fade_to", to);
        return this;
    }

    /// <summary>Animates the entry/exit vertical offset range, in pixels.</summary>
    public PresenceElement SlideY(double from, double to)
    {
        Arena.AddMethodNumber(Index, "slide_from", from);
        Arena.AddMethodNumber(Index, "slide_to", to);
        return this;
    }

    /// <summary>Adds the animated content.</summary>
    public PresenceElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
