using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class AccordionPage : GalleryPage
{
    public override string Title => "Accordion";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Accordion",
            "A typed accordion accepting only AccordionItem children.",
            ui.Accordion("faq")
                .Multiple()
                .Bordered()
                .Add(
                    ui.AccordionItem()
                        .Title(ui.Label("What is gpui-net-shell?"))
                        .Open()
                        .Add(ui.Text("A C#-hosted shell over a Rust GPUI runtime.")),
                    ui.AccordionItem()
                        .Title(ui.Label("How do the samples work?"))
                        .Add(ui.Text("Each page is a managed view rendered through the ABI.")),
                    ui.AccordionItem()
                        .Title(ui.Label("Unavailable"))
                        .Disabled()
                        .Add(ui.Text("This item is disabled."))
                )
        );
}
