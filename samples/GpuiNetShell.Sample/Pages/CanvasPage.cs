using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// Demonstrates native painting and hit regions: a gradient/plot drawn from a
/// per-frame prepaint callback, and two clickable halves whose clicks update the
/// page's entity state.
/// </summary>
[GpuiCallbacks]
internal sealed partial class CanvasPage : GalleryPage<CanvasPage.State>
{
    internal sealed class State
    {
        public int Clicks { get; set; }
        public string Last { get; set; } = "(none)";
    }

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
            "Native GPUI painting plus clickable regions. The plot is drawn from the prepaint callback (bounds-aware); click a half to count.",
            ui.Label($"clicks: {state.Clicks}   last: {state.Last}"),
            ui.Canvas("paint-demo")
                .WFull()
                .H(240)
                .Rounded(12)
                .Clip()
                .Prepaint(PlotToken)
                .Add(
                    ui.HitRegion("left", 0.0, 0.0, 0.5, 1.0)
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
                )
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
                ui.PaintLine(0.5, 0.0, 0.5, 1.0).Stroke(1).Color("#ffffff33")
            );
}
