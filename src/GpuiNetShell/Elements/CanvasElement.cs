using System.Globalization;
using GpuiNetShell.Events;
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

    /// <summary>
    /// Runs <paramref name="token"/> at layout time with the available space and
    /// uses the size it returns; the callback returns a <c>Text</c> node whose
    /// data is <c>"width\theight"</c>.
    /// </summary>
    public CanvasElement Measure(ulong token)
    {
        Arena.AddCallback(Index, "measure", token);
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

/// <summary>The cursor shown while hovering a hit region.</summary>
public enum CursorKind
{
    Default,
    Pointer,
    Text,
    Crosshair,
    Grab,
    Grabbing,
    ResizeHorizontal,
    ResizeVertical,
}

/// <summary>
/// A clickable/hoverable region on a <see cref="CanvasElement"/>. Coordinates
/// are <see cref="PaintLength"/>s (fractions of the canvas by default). The
/// region paints nothing.
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

    /// <summary>Runs when the pointer enters the region.</summary>
    public HitRegionElement OnHoverEnter(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_hover_enter", Events.Register(_ => handler()));
        return this;
    }

    /// <summary>Runs when the pointer leaves the region.</summary>
    public HitRegionElement OnHoverExit(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_hover_exit", Events.Register(_ => handler()));
        return this;
    }

    /// <summary>Runs when the region is pressed.</summary>
    public HitRegionElement OnPress(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_press", Events.Register(_ => handler()));
        return this;
    }

    /// <summary>Runs when the region is released.</summary>
    public HitRegionElement OnRelease(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_release", Events.Register(_ => handler()));
        return this;
    }

    /// <summary>Runs as the pointer moves over the region, with its position.</summary>
    public HitRegionElement OnMove(Action<double, double> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(
            Index,
            "on_move",
            Events.Register(value =>
            {
                var (x, y) = ParsePair(value.String);
                handler(x, y);
            })
        );
        return this;
    }

    /// <summary>Runs when the region is scrolled, with the scroll delta.</summary>
    public HitRegionElement OnScroll(Action<double, double> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(
            Index,
            "on_scroll",
            Events.Register(value =>
            {
                var (x, y) = ParsePair(value.String);
                handler(x, y);
            })
        );
        return this;
    }

    /// <summary>Sets the cursor shown while hovering.</summary>
    public HitRegionElement Cursor(CursorKind kind)
    {
        Arena.AddMethodEnum(Index, "cursor", Wire(kind));
        return this;
    }

    /// <summary>Blocks mouse events from reaching regions beneath this one.</summary>
    public HitRegionElement BlockMouse(bool block = true)
    {
        Arena.AddMethodNumber(Index, "block_mouse", block ? 1 : 0);
        return this;
    }

    /// <summary>Blocks mouse events but lets scroll pass through.</summary>
    public HitRegionElement BlockMouseExceptScroll(bool block = true)
    {
        Arena.AddMethodNumber(Index, "block_mouse_except_scroll", block ? 1 : 0);
        return this;
    }

    private static (double A, double B) ParsePair(string? payload)
    {
        if (string.IsNullOrEmpty(payload))
        {
            return (0, 0);
        }
        var parts = payload.Split('\t');
        static double Parse(string value) =>
            double.TryParse(value, NumberStyles.Float, CultureInfo.InvariantCulture, out var result)
                ? result
                : 0;
        return (parts.Length > 1 ? Parse(parts[1]) : 0, parts.Length > 2 ? Parse(parts[2]) : 0);
    }

    private static string Wire(CursorKind kind) =>
        kind switch
        {
            CursorKind.Pointer => "pointer",
            CursorKind.Text => "text",
            CursorKind.Crosshair => "crosshair",
            CursorKind.Grab => "grab",
            CursorKind.Grabbing => "grabbing",
            CursorKind.ResizeHorizontal => "ew-resize",
            CursorKind.ResizeVertical => "ns-resize",
            _ => "default",
        };
}
