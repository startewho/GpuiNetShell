using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ImagePage : GalleryPage
{
    public override string Title => "Image";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Image",
            "Images and SVGs loaded by path (bundled assets or filesystem). gpui's asset cache releases decoded data when unused.",
            ui.HStack(
                    ui.Image("icons/sun.svg").Fit("contain").Size(72),
                    ui.Image("icons/moon.svg").Fit("contain").Size(72),
                    ui.Image("icons/palette.svg").Fit("cover").Size(72).Rounded(12),
                    ui.Image("icons/image.svg").Fit("scale_down").Size(72)
                )
                .Gap(12)
                .ItemsCenter(),
            ui.Label("Size and radius come from style; `fit` selects the object-fit mode.")
        );
}
