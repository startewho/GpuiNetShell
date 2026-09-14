using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class PopoverPage : GalleryPage<PopoverPage.State>
{
    internal sealed class State
    {
        public bool Open { get; set; }
    }

    public override string Title => "Popover";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Popover",
            "A button-triggered popover with a styled content surface.",
            ui.Popover("popover", "Show details")
                .Open(state.Open)
                .OverlayClosable()
                .OnOpenChange(open =>
                {
                    Update((s, c) => { s.Open = open; c.Notify(); });
                })
                .Content(
                    ui.VStack(
                            ui.Label("Details").FontSemibold(),
                            ui.Text("Anchored content painted above the window.")
                        )
                        .Gap(4)
                        .P(8)
                ),
            ui.Label($"Open: {state.Open}")
        );
}
