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

    /// <summary>The theme's enter easing.</summary>
    Enter,

    /// <summary>The theme's exit easing.</summary>
    Exit,

    /// <summary>The theme's move easing.</summary>
    Move,
}

/// <summary>A semantic duration from the theme.</summary>
public enum DurationToken
{
    Instant,
    Fast,
    Normal,
    Slow,
}

/// <summary>A semantic spring from the theme.</summary>
public enum SpringToken
{
    Control,
    Move,
}

/// <summary>A playback direction for keyframe animation.</summary>
public enum KeyframeDirection
{
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

/// <summary>One keyframe of a scalar track.</summary>
public readonly record struct Keyframe(double Offset, double Value, Easing Easing = Easing.Linear);

/// <summary>
/// How an animated value moves toward its target: a timed
/// <see cref="Transition"/>, a <see cref="Spring"/>, or a
/// <see cref="Keyframes(TimeSpan, Easing, Keyframe[])"/> track. Durations and
/// easings can be semantic theme tokens.
/// </summary>
public sealed record MotionSpec
{
    internal MotionKind Kind { get; init; } = MotionKind.Transition;

    internal int DurationMs { get; init; } = 200;

    internal DurationToken? DurationThemeToken { get; init; }

    internal Easing Easing { get; init; } = Easing.Out;

    internal int DelayMs { get; init; }

    internal int SpringResponseMs { get; init; } = 180;

    internal double SpringDamping { get; init; } = 1.0;

    internal bool SpringTravel { get; init; } = true;

    internal SpringToken? SpringThemeToken { get; init; }

    internal string? KeyframeTrack { get; init; }

    internal int Iterations { get; init; } = 1;

    internal KeyframeDirection Direction { get; init; } = KeyframeDirection.Normal;

    /// <summary>A timed transition with an easing curve.</summary>
    public static MotionSpec Transition(TimeSpan duration, Easing easing = Easing.Out) =>
        new() { DurationMs = Ms(duration), Easing = easing };

    /// <summary>A timed transition using the theme's semantic duration and easing.</summary>
    public static MotionSpec Themed(DurationToken duration, Easing easing = Easing.Enter) =>
        new() { DurationThemeToken = duration, Easing = easing };

    /// <summary>A spring with an explicit response.</summary>
    public static MotionSpec Spring(TimeSpan response, double damping = 1.0, bool travel = true) =>
        new()
        {
            Kind = MotionKind.Spring,
            SpringResponseMs = Ms(response),
            SpringDamping = damping,
            SpringTravel = travel,
        };

    /// <summary>A spring using the theme's semantic spring.</summary>
    public static MotionSpec Spring(SpringToken token) =>
        new() { Kind = MotionKind.Spring, SpringThemeToken = token };

    /// <summary>A keyframe track driving opacity.</summary>
    public static MotionSpec Keyframes(
        TimeSpan duration,
        Easing easing,
        params Keyframe[] frames
    ) =>
        new()
        {
            Kind = MotionKind.Keyframes,
            DurationMs = Ms(duration),
            Easing = easing,
            KeyframeTrack = BuildTrack(frames),
        };

    /// <summary>Delays the animation; used for staggered lists.</summary>
    public MotionSpec WithDelay(TimeSpan delay) => this with { DelayMs = Ms(delay) };

    /// <summary>Repeats the keyframe track forever.</summary>
    public MotionSpec Loop() => this with { Iterations = 0 };

    /// <summary>Repeats the keyframe track a fixed number of times.</summary>
    public MotionSpec Repeat(int count) => this with { Iterations = count };

    /// <summary>Sets the keyframe playback direction.</summary>
    public MotionSpec WithDirection(KeyframeDirection direction) =>
        this with { Direction = direction };

    internal void AppendTo(RenderArena arena, int index)
    {
        arena.AddMethodEnum(index, "kind", Wire(Kind));
        if (DurationThemeToken is { } duration)
        {
            arena.AddMethodEnum(index, "duration_token", Wire(duration));
        }
        else
        {
            arena.AddMethodNumber(index, "duration_ms", DurationMs);
        }
        arena.AddMethodEnum(index, "easing", Wire(Easing));
        if (DelayMs != 0)
        {
            arena.AddMethodNumber(index, "delay_ms", DelayMs);
        }
        if (Kind == MotionKind.Spring)
        {
            if (SpringThemeToken is { } spring)
            {
                arena.AddMethodEnum(index, "spring_token", Wire(spring));
            }
            else
            {
                arena.AddMethodNumber(index, "spring_response_ms", SpringResponseMs);
                arena.AddMethodNumber(index, "spring_damping", SpringDamping);
                arena.AddMethodNumber(index, "spring_travel", SpringTravel ? 1 : 0);
            }
        }
        if (Kind == MotionKind.Keyframes && KeyframeTrack is { } track)
        {
            arena.AddMethodString(index, "keyframes", track);
            arena.AddMethodNumber(index, "iterations", Iterations);
            arena.AddMethodEnum(index, "direction", Wire(Direction));
        }
    }

    private static int Ms(TimeSpan duration) => (int)Math.Max(0, duration.TotalMilliseconds);

    private static string BuildTrack(Keyframe[] frames)
    {
        ArgumentNullException.ThrowIfNull(frames);
        if (frames.Length < 2)
        {
            throw new ArgumentException("Keyframes need at least two frames.", nameof(frames));
        }
        return string.Join(
            ';',
            frames.Select(frame =>
                $"{Length.Format(frame.Offset)}:{Length.Format(frame.Value)}:{Wire(frame.Easing)}"
            )
        );
    }

    internal static string Wire(MotionKind kind) =>
        kind switch
        {
            MotionKind.Spring => "spring",
            MotionKind.Keyframes => "keyframes",
            _ => "transition",
        };

    internal static string Wire(Easing easing) =>
        easing switch
        {
            Easing.Linear => "linear",
            Easing.Ease => "ease",
            Easing.In => "ease_in",
            Easing.InOut => "ease_in_out",
            Easing.Enter => "enter",
            Easing.Exit => "exit",
            Easing.Move => "move",
            _ => "ease_out",
        };

    internal static string Wire(DurationToken token) =>
        token switch
        {
            DurationToken.Instant => "instant",
            DurationToken.Fast => "fast",
            DurationToken.Slow => "slow",
            _ => "normal",
        };

    internal static string Wire(SpringToken token) =>
        token switch
        {
            SpringToken.Move => "move",
            _ => "control",
        };

    internal static string Wire(KeyframeDirection direction) =>
        direction switch
        {
            KeyframeDirection.Reverse => "reverse",
            KeyframeDirection.Alternate => "alternate",
            KeyframeDirection.AlternateReverse => "alternate_reverse",
            _ => "normal",
        };
}

internal enum MotionKind
{
    Transition,
    Spring,
    Keyframes,
}

/// <summary>
/// An animated container. Its animated values are targets uploaded each render;
/// the native host interpolates from the current value to the new target and
/// requests frames while moving. Translation is a visual offset (the wrapper is
/// relatively positioned), so it does not affect layout. A keyframe spec drives
/// opacity.
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

/// <summary>
/// A measured, clipped vertical reveal driven by animated progress. Set its
/// open flag each render; the native host animates the measured height.
/// </summary>
public sealed class RevealElement : Element
{
    internal RevealElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Adds the revealed content.</summary>
    public RevealElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
