using System.Globalization;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>The canvas-local bounds passed to a prepaint callback.</summary>
public readonly struct CanvasBounds
{
    public CanvasBounds(double x, double y, double width, double height)
    {
        X = x;
        Y = y;
        Width = width;
        Height = height;
    }

    public double X { get; }

    public double Y { get; }

    public double Width { get; }

    public double Height { get; }

    /// <summary>Parses the <c>[x, y, width, height]</c> arguments of a prepaint callback.</summary>
    public static CanvasBounds Parse(IReadOnlyList<string> arguments) =>
        new(At(arguments, 0), At(arguments, 1), At(arguments, 2), At(arguments, 3));

    private static double At(IReadOnlyList<string> arguments, int index) =>
        arguments.Count > index
        && double.TryParse(
            arguments[index],
            NumberStyles.Float,
            CultureInfo.InvariantCulture,
            out var value
        )
            ? value
            : 0;
}

/// <summary>
/// A reusable painter over a canvas, used by custom views to draw declaratively
/// inside a prepaint callback. Coordinates are <see cref="PaintLength"/>s
/// (fractions of the canvas by default).
/// </summary>
public interface ICanvasView
{
    /// <summary>Draws onto <paramref name="painter"/> using the canvas bounds.</summary>
    void Paint(CanvasPainter painter, CanvasBounds bounds);
}

/// <summary>Collects paint primitives and hit regions into a canvas.</summary>
public sealed class CanvasPainter
{
    private readonly RenderContext _ui;
    private readonly CanvasElement _canvas;

    internal CanvasPainter(RenderContext ui, CanvasElement canvas)
    {
        _ui = ui;
        _canvas = canvas;
    }

    /// <summary>Draws a rectangle.</summary>
    public CanvasPainter Rect(
        PaintLength x,
        PaintLength y,
        PaintLength w,
        PaintLength h,
        string? fill = null,
        double radius = 0,
        double stroke = 0,
        string? color = null
    )
    {
        var element = _ui.PaintRect(x, y, w, h);
        if (fill is not null)
        {
            element.Fill(fill);
        }
        if (radius > 0)
        {
            element.Radius(radius);
        }
        if (stroke > 0)
        {
            element.Stroke(stroke);
        }
        if (color is not null)
        {
            element.Color(color);
        }
        _canvas.Add(element);
        return this;
    }

    /// <summary>Draws a line.</summary>
    public CanvasPainter Line(
        PaintLength x1,
        PaintLength y1,
        PaintLength x2,
        PaintLength y2,
        double stroke = 1,
        string? color = null,
        double dashOn = 0,
        double dashOff = 0
    )
    {
        var element = _ui.PaintLine(x1, y1, x2, y2).Stroke(stroke);
        if (color is not null)
        {
            element.Color(color);
        }
        if (dashOn > 0 && dashOff > 0)
        {
            element.Dash(dashOn, dashOff);
        }
        _canvas.Add(element);
        return this;
    }

    /// <summary>Draws a path from a compact DSL.</summary>
    public CanvasPainter Path(
        string dsl,
        double stroke = 1,
        string? color = null,
        string? fill = null,
        double dashOn = 0,
        double dashOff = 0
    )
    {
        var element = _ui.PaintPath(dsl);
        if (stroke > 0)
        {
            element.Stroke(stroke);
        }
        if (color is not null)
        {
            element.Color(color);
        }
        if (fill is not null)
        {
            element.Fill(fill);
        }
        if (dashOn > 0 && dashOff > 0)
        {
            element.Dash(dashOn, dashOff);
        }
        _canvas.Add(element);
        return this;
    }

    /// <summary>Draws a two-stop linear gradient rectangle.</summary>
    public CanvasPainter Gradient(
        PaintLength x,
        PaintLength y,
        PaintLength w,
        PaintLength h,
        double angle,
        string from,
        string to,
        double radius = 0
    )
    {
        var element = _ui.PaintGradient(x, y, w, h, angle, from, to);
        if (radius > 0)
        {
            element.Radius(radius);
        }
        _canvas.Add(element);
        return this;
    }

    /// <summary>Draws a drop or inset shadow.</summary>
    public CanvasPainter Shadow(
        PaintLength x,
        PaintLength y,
        PaintLength w,
        PaintLength h,
        double offsetX,
        double offsetY,
        double blur,
        string color,
        double radius = 0,
        bool inset = false
    )
    {
        var element = _ui.PaintShadow(x, y, w, h, offsetX, offsetY, blur, color);
        if (radius > 0)
        {
            element.Radius(radius);
        }
        if (inset)
        {
            element.Inset();
        }
        _canvas.Add(element);
        return this;
    }

    /// <summary>Draws an SVG.</summary>
    public CanvasPainter Image(
        PaintLength x,
        PaintLength y,
        PaintLength w,
        PaintLength h,
        string source,
        string? tint = null
    )
    {
        var element = _ui.PaintImage(x, y, w, h, source);
        if (tint is not null)
        {
            element.Tint(tint);
        }
        _canvas.Add(element);
        return this;
    }

    /// <summary>Adds a clickable region.</summary>
    public CanvasPainter Region(
        string id,
        PaintLength x,
        PaintLength y,
        PaintLength w,
        PaintLength h,
        Action onClick
    )
    {
        _canvas.Add(_ui.HitRegion(id, x, y, w, h).OnClick(onClick));
        return this;
    }

    /// <summary>The canvas the painter drew into.</summary>
    public CanvasElement Build() => _canvas;
}
