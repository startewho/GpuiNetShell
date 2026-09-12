using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class BreadcrumbPage : GalleryPage
{
    public override string Title => "Breadcrumb";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Breadcrumb",
            "A navigation trail built from an ordered list of labels.",
            ui.Breadcrumb("Home", "Library", "Data", "Profile")
        );
}
