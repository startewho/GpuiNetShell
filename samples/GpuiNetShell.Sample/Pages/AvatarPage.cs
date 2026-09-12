using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class AvatarPage : GalleryPage
{
    public override string Title => "Avatar";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Avatar",
            "Circular avatars with name-derived initials fallbacks.",
            ui.HStack(
                    ui.Avatar().Name("Ada Lovelace").Size(ControlSize.Small),
                    ui.Avatar().Name("Grace Hopper"),
                    ui.Avatar().Name("Linus Torvalds").Size(ControlSize.Large)
                )
                .Gap(12)
                .ItemsCenter()
        );
}
