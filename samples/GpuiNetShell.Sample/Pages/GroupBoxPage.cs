using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class GroupBoxPage : GalleryPage
{
    public override string Title => "GroupBox";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "GroupBox",
            "Titled containers for grouping related content.",
            ui.GroupBox()
                .Title("Options")
                .Variant(GroupBoxVariant.Outline)
                .Add(ui.Label("Grouped content inside a titled container.")),
            ui.GroupBox()
                .Title("Filled")
                .Variant(GroupBoxVariant.Fill)
                .Add(ui.Text("A filled group body."))
        );
}
