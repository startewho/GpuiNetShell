using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// Demonstrates native painting and hit regions: a gradient/plot drawn from a
/// per-frame prepaint callback, an SVG paint, a measured custom gauge view, and
/// clickable halves whose clicks update the page's entity state.
/// </summary>
[GpuiCallbacks]
internal sealed partial class CanvasPage : GalleryPage<CanvasPage.State>
{
    internal sealed class State
    {
        public int Clicks { get; set; }
        public string Last { get; set; } = "(none)";
    }

    private readonly GaugeView _gauge = new();

    public override string Title => "Canvas";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Canvas",
            "Native GPUI painting and hit regions: gradient plot, SVG, a measured custom view, and clicks.",
            ui.Label($"clicks: {state.Clicks}   last: {state.Last}"),
            ui.Canvas("paint-demo")
                .WFull()
                .H(240)
                .Rounded(12)
                .Clip()
                .Prepaint(PlotToken)
                .Add(
                    ui.HitRegion("left", 0.0, 0.0, 0.5, 1.0)
                        .Cursor(CursorKind.Pointer)
                        .OnHoverEnter(() =>
                            Update(
                                (s, c) =>
                                {
                                    s.Last = "hover left";
                                    c.Notify();
                                }
                            )
                        )
                        .OnHoverExit(() =>
                            Update(
                                (s, c) =>
                                {
                                    s.Last = "(none)";
                                    c.Notify();
                                }
                            )
                        )
                        .OnClick(() =>
                            Update(
                                (s, c) =>
                                {
                                    s.Clicks++;
                                    s.Last = "left";
                                    c.Notify();
                                }
                            )
                        ),
                    ui.HitRegion("right", 0.5, 0.0, 0.5, 1.0)
                        .Cursor(CursorKind.Pointer)
                        .OnClick(() =>
                            Update(
                                (s, c) =>
                                {
                                    s.Clicks++;
                                    s.Last = "right";
                                    c.Notify();
                                }
                            )
                        )
                ),
            Section(
                ref ui,
                "Measured custom view",
                "A canvas measured at layout time; the gauge is painted by an ICanvasView."
            ),
            ui.Canvas("gauge").WFull().Rounded(10).Measure(MeasureToken).Prepaint(GaugeToken)
        );

    // The prepaint callback runs each frame with the canvas bounds and returns
    // the drawing; it is not part of the retained snapshot.
    [GpuiCallback("Plot")]
    private Element RenderPlot(RenderContext ui, IReadOnlyList<string> bounds) =>
        ui.Canvas("prepaint")
            .Add(
                ui.PaintGradient(0.0, 0.0, 1.0, 1.0, 45, "#1e3a8a", "#0f766e"),
                ui
                    .PaintRect(0.06, 0.12, 0.34, 0.5)
                    .Fill("#ffffff22")
                    .Color("#bfdbfe")
                    .Stroke(1.5)
                    .Radius(12),
                ui
                    .PaintPath(
                        "M 0.06 0.86 L 0.3 0.6 Q 0.4 0.5 0.5 0.62 C 0.6 0.74 0.72 0.78 0.94 0.55"
                    )
                    .Stroke(2.5)
                    .Color("#fbbf24"),
                ui.PaintLine(0.5, 0.0, 0.5, 1.0).Stroke(1).Color("#ffffff33"),
                ui.PaintImage(0.78, 0.12, 0.14, 0.4, "icons/check.svg").Tint("#ffffff")
            );

    // A measured canvas: the callback receives the available space and returns
    // "width\theight".
    [GpuiCallback("Measure")]
    private string MeasureGauge(double availableWidth, double availableHeight) =>
        $"{availableWidth}\t120";

    [GpuiCallback("Gauge")]
    private Element RenderGauge(RenderContext ui, IReadOnlyList<string> bounds)
    {
        _gauge.Value = Math.Clamp(Entity.Read().Clicks / 10.0, 0.0, 1.0);
        var painter = ui.Painter("gauge-draw");
        _gauge.Paint(painter, CanvasBounds.Parse(bounds));
        return painter.Build();
    }

    /// <summary>A reusable custom view that paints a progress bar.</summary>
    private sealed class GaugeView : ICanvasView
    {
        public double Value { get; set; }

        public void Paint(CanvasPainter painter, CanvasBounds bounds)
        {
            painter
                .Gradient(0.0, 0.0, 1.0, 1.0, 90, "#0f172a", "#1e293b", radius: 10)
                .Rect(0.05, 0.55, 0.9, 0.2, stroke: 1, color: "#334155", radius: 6)
                .Rect(0.05, 0.55, 0.9 * Value, 0.2, fill: "#22c55e", radius: 6)
                .Line(0.05, 0.25, 0.95, 0.25, stroke: 1, color: "#ffffff22", dashOn: 4, dashOff: 4);
        }
    }
}
