using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class RatingPage : GalleryPage<RatingPage.State>
{
    internal sealed class State
    {
        public int Rating { get; set; } = 3;
    }

    public override string Title => "Rating";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Rating",
            "Interactive star ratings; the clicked value is reported as a number.",
            ui.Rating("quality")
                .Max(5)
                .Value(state.Rating)
                .OnChange(value =>
                {
                    Update((s, c) =>
                    {
                        s.Rating = value;
                        c.Notify();
                    });
                }),
            ui.Rating("color").Max(5).Value(4).Color("orange-500"),
            ui.Rating("small").Max(5).Value(2).Size(ControlSize.Small),
            ui.Label($"Rating: {state.Rating}/5")
        );
}
