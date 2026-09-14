using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A length for paint geometry. An <see cref="int"/> is logical pixels; a
/// <see cref="double"/> in <c>0..1</c> is a fraction of the corresponding canvas
/// dimension. Use <see cref="From"/> for an explicit <see cref="Length"/>.
/// </summary>
public readonly struct PaintLength
{
    internal string Wire { get; }

    private PaintLength(string wire) => Wire = wire;

    /// <summary>Logical pixels.</summary>
    public static implicit operator PaintLength(int pixels) => new(Length.Px(pixels).Wire);

    /// <summary>A fraction of the canvas dimension (0–1).</summary>
    public static implicit operator PaintLength(double fraction) =>
        new(Length.Relative(fraction).Wire);

    /// <summary>An explicit length.</summary>
    public static PaintLength From(Length length) => new(length.Wire);
}

/// <summary>Base of a paint primitive consumed by a <see cref="CanvasElement"/>.</summary>
public abstract class PaintElement : Element
{
    private protected PaintElement(RenderContext ui, int index)
        : base(ui, index) { }
}

/// <summary>A filled/outlined rectangle drawn on a canvas.</summary>
public sealed class PaintRectElement : PaintElement
{
    internal PaintRectElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Fills the rectangle with a color.</summary>
    public PaintRectElement Fill(string color)
    {
        Arena.AddMethodString(Index, "fill", color);
        return this;
    }

    /// <summary>Sets the border width, in pixels.</summary>
    public PaintRectElement Stroke(double pixels)
    {
        Arena.AddMethodNumber(Index, "stroke", pixels);
        return this;
    }

    /// <summary>Sets the border color.</summary>
    public PaintRectElement Color(string color)
    {
        Arena.AddMethodString(Index, "color", color);
        return this;
    }

    /// <summary>Sets the corner radius, in pixels.</summary>
    public PaintRectElement Radius(double pixels)
    {
        Arena.AddMethodNumber(Index, "radius", pixels);
        return this;
    }
}

/// <summary>A straight line drawn on a canvas.</summary>
public sealed class PaintLineElement : PaintElement
{
    internal PaintLineElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the line width, in pixels.</summary>
    public PaintLineElement Stroke(double pixels)
    {
        Arena.AddMethodNumber(Index, "stroke", pixels);
        return this;
    }

    /// <summary>Sets the line color.</summary>
    public PaintLineElement Color(string color)
    {
        Arena.AddMethodString(Index, "color", color);
        return this;
    }

    /// <summary>Sets a dash pattern, in pixels.</summary>
    public PaintLineElement Dash(double on, double off)
    {
        Arena.AddMethodString(Index, "dash", $"{Length.Format(on)} {Length.Format(off)}");
        return this;
    }
}

/// <summary>
/// A path drawn on a canvas. <paramref name="dsl"/> is a compact path:
/// <c>M x y</c>, <c>L x y</c>, <c>Q cx cy x y</c>, <c>C c1x c1y c2x c2y x y</c>,
/// <c>A rx ry rot large sweep x y</c>, <c>Z</c>. Coordinates are pixels, or a
/// fraction with an <c>f</c> suffix (<c>0.5f</c>).
/// </summary>
public sealed class PaintPathElement : PaintElement
{
    internal PaintPathElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Fills the path with a color.</summary>
    public PaintPathElement Fill(string color)
    {
        Arena.AddMethodString(Index, "fill", color);
        return this;
    }

    /// <summary>Strokes the path, in pixels.</summary>
    public PaintPathElement Stroke(double pixels)
    {
        Arena.AddMethodNumber(Index, "stroke", pixels);
        return this;
    }

    /// <summary>Sets the stroke color.</summary>
    public PaintPathElement Color(string color)
    {
        Arena.AddMethodString(Index, "color", color);
        return this;
    }

    /// <summary>Sets a dash pattern, in pixels.</summary>
    public PaintPathElement Dash(double on, double off)
    {
        Arena.AddMethodString(Index, "dash", $"{Length.Format(on)} {Length.Format(off)}");
        return this;
    }
}

/// <summary>A two-stop linear gradient rectangle drawn on a canvas.</summary>
public sealed class PaintGradientElement : PaintElement
{
    internal PaintGradientElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the corner radius, in pixels.</summary>
    public PaintGradientElement Radius(double pixels)
    {
        Arena.AddMethodNumber(Index, "radius", pixels);
        return this;
    }
}

/// <summary>A drop or inset shadow drawn on a canvas.</summary>
public sealed class PaintShadowElement : PaintElement
{
    internal PaintShadowElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Draws the shadow inside the bounds.</summary>
    public PaintShadowElement Inset(bool inset = true)
    {
        Arena.AddMethodNumber(Index, "inset", inset ? 1 : 0);
        return this;
    }

    /// <summary>Sets the corner radius, in pixels.</summary>
    public PaintShadowElement Radius(double pixels)
    {
        Arena.AddMethodNumber(Index, "radius", pixels);
        return this;
    }
}
