using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class HoverCardPage : GalleryPage
{
    public override string Title => "HoverCard";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "HoverCard",
            "A hover-triggered card with lazily rendered content.",
            ui.HoverCard("hover")
                .TriggerElement(ui.Button("hover-trigger").Label("Hover me").Secondary())
                .Content(
                    ui.VStack(ui.Label("Hover card"), ui.Text("Shown on hover."))
                        .Gap(4)
                        .P(8)
                )
        );
}
