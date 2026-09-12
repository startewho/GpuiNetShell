using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class DescriptionListPage : GalleryPage
{
    public override string Title => "DescriptionList";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "DescriptionList",
            "A structured label/value list accepting DescriptionItem children.",
            ui.DescriptionList()
                .Columns(2)
                .Add(
                    ui.DescriptionItem("Name").Value("Ada Lovelace"),
                    ui.DescriptionItem("Role").Value("Engineer"),
                    ui.DescriptionItem("Location").Value("London"),
                    ui.DescriptionItem("Bio").Value("Mathematician and writer.").Span(2)
                ),
            ui.DescriptionList()
                .Vertical()
                .Bordered(false)
                .Add(
                    ui.DescriptionItem("Status").Value("Active"),
                    ui.DescriptionItem("Plan").Value("Enterprise")
                )
        );
}
