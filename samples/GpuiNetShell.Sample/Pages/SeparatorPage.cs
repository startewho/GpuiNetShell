using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SeparatorPage : GalleryPage
{
    public override string Title => "Separator";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Separator",
            "Horizontal and vertical, solid and dashed separators.",
            ui.Separator(),
            ui.Separator().Label("Account"),
            ui.DashedSeparator().Color("red-500"),
            ui.HStack(
                    ui.Label("Left"),
                    ui.VerticalSeparator().H(40),
                    ui.Label("Right"),
                    ui.VerticalDashedSeparator().H(40),
                    ui.Label("Far right")
                )
                .Gap(12)
                .ItemsCenter()
        );
}
