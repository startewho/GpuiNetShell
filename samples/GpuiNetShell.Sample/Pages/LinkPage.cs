using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class LinkPage : GalleryPage
{
    public override string Title => "Link";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Link",
            "External-resource links; disabled state and activation are honored.",
            ui.HStack(
                    ui.Link("docs")
                        .Href("https://gpui-kit.com")
                        .Add(ui.Text("Documentation")),
                    ui.Link("disabled")
                        .Href("https://example.com")
                        .Disabled()
                        .Add(ui.Text("Disabled link"))
                )
                .Gap(16)
                .ItemsCenter()
        );
}
