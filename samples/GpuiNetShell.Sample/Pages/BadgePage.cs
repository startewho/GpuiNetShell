using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class BadgePage : GalleryPage
{
    public override string Title => "Badge";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Badge",
            "Counts and dots positioned over ordinary children.",
            ui.HStack(
                    ui.Badge().Count(3),
                    ui.Badge().Count(12),
                    ui.Badge().Count(120).Max(99),
                    ui.Badge().Dot(),
                    ui.Badge().Count(5).Color("red-500")
                )
                .Gap(12)
                .ItemsCenter(),
            ui.HStack(
                    ui.Badge().Count(2).Size(ControlSize.Small),
                    ui.Badge().Count(4).Size(ControlSize.Medium),
                    ui.Badge().Count(6).Size(ControlSize.Large)
                )
                .Gap(12)
                .ItemsCenter()
        );
}
