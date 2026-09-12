using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TextPage : GalleryPage
{
    public override string Title => "Text";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Text",
            "Plain gpui-component text in a styleable wrapper.",
            ui.Text("The quick brown fox jumps over the lazy dog."),
            ui.Text("A second run of text, styled through the shared surface.")
                .FontSemibold()
                .TextSize(18)
        );
}
