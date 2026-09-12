using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class IconPage : GalleryPage
{
    public override string Title => "Icon";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Icon",
            "SVG icons loaded from a relative path beneath the asset root.",
            ui.HStack(
                    ui.Icon("icons/check.svg").Size(ControlSize.Small),
                    ui.Icon("icons/check.svg").Size(ControlSize.Medium),
                    ui.Icon("icons/check.svg").Size(ControlSize.Large).Color("blue-600"),
                    ui.Icon("icons/check.svg").Rotate(0.5)
                )
                .Gap(12)
                .ItemsCenter()
        );
}
