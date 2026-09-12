using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class PopoverPage : GalleryPage
{
    private bool _open;

    public override string Title => "Popover";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Popover",
            "A button-triggered popover with a styled content surface.",
            ui.Popover("popover", "Show details")
                .Open(_open)
                .OverlayClosable()
                .OnOpenChange(open =>
                {
                    _open = open;
                    Invalidate();
                })
                .Content(
                    ui.VStack(
                            ui.Label("Details").FontSemibold(),
                            ui.Text("Anchored content painted above the window.")
                        )
                        .Gap(4)
                        .P(8)
                ),
            ui.Label($"Open: {_open}")
        );
}
