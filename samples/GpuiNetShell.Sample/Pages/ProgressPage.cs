using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ProgressPage : GalleryPage
{
    public override string Title => "Progress";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Progress",
            "Determinate values and the indeterminate animation.",
            ui.VStack(
                    ui.Progress("p-25").Value(25).Full(),
                    ui.Progress("p-60").Value(60).Full(),
                    ui.Progress("p-90").Value(90).Full(),
                    ui.Progress("p-loading").Loading().Full()
                )
                .Gap(12),
            ui.HStack(
                    ui.Progress("p-small").Value(40).Size(ControlSize.Small),
                    ui.Progress("p-large").Value(70).Size(ControlSize.Large)
                )
                .Gap(12)
                .ItemsCenter()
        );
}
