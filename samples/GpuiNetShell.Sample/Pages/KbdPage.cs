using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class KbdPage : GalleryPage
{
    public override string Title => "Kbd";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Kbd",
            "Platform-formatted keyboard shortcut keycaps.",
            ui.HStack(
                    ui.Kbd("ctrl-k"),
                    ui.Kbd("cmd-shift-p").Outline(),
                    ui.Kbd("alt-enter").Appearance(false)
                )
                .Gap(8)
                .ItemsCenter()
        );
}
