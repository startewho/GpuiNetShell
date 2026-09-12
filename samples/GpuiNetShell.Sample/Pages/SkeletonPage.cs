using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SkeletonPage : GalleryPage
{
    public override string Title => "Skeleton";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Skeleton",
            "Animated loading placeholders.",
            ui.VStack(
                    ui.HStack(
                            ui.Skeleton().Size(40),
                            ui.VStack(
                                    ui.Skeleton().W(200).H(12),
                                    ui.Skeleton().Secondary().W(140).H(12)
                                )
                                .Gap(8)
                        )
                        .Gap(12)
                        .ItemsCenter(),
                    ui.Skeleton().W(320).H(12),
                    ui.Skeleton().W(260).H(12)
                )
                .Gap(12)
        );
}
