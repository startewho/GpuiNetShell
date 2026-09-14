using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A self-painted surface. Its children are paint primitives
/// (<see cref="PaintRectElement"/>, <see cref="PaintLineElement"/>,
/// <see cref="PaintPathElement"/>, <see cref="PaintGradientElement"/>,
/// <see cref="PaintShadowElement"/>), painted in declaration order against the
/// canvas bounds. The canvas itself is laid out by its style, so
/// <c>.Full()</c>, <c>.W(...)</c>, <c>.Bg(...)</c> still apply.
/// </summary>
public sealed class CanvasElement : Element
{
    internal CanvasElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Clips painting to the canvas bounds.</summary>
    public CanvasElement Clip(bool clip = true)
    {
        if (clip)
        {
            Arena.AddMethod(Index, "clip");
        }
        return this;
    }

    /// <summary>
    /// Runs <paramref name="token"/> each prepaint with the canvas bounds
    /// (x, y, width, height) and uses the paint primitives and regions it
    /// returns as dynamic canvas content.
    /// </summary>
    public CanvasElement Prepaint(ulong token)
    {
        Arena.AddCallback(Index, "prepaint", token);
        return this;
    }

    /// <summary>Appends paint primitives and hit regions.</summary>
    public CanvasElement Add(params Element[] children)
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
/// A clickable region on a <see cref="CanvasElement"/>. Coordinates are
/// <see cref="PaintLength"/>s (fractions of the canvas by default). The region
/// paints nothing.
/// </summary>
public sealed class HitRegionElement : Element
{
    internal HitRegionElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Runs when the region is clicked.</summary>
    public HitRegionElement OnClick(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_click", Events.Register(handler));
        return this;
    }

    /// <summary>Blocks mouse events from reaching regions beneath this one.</summary>
    public HitRegionElement BlockMouse(bool block = true)
    {
        Arena.AddMethodNumber(Index, "block_mouse", block ? 1 : 0);
        return this;
    }
}
