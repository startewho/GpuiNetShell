using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ClipboardPage : GalleryPage
{
    public override string Title => "Clipboard";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Clipboard",
            "A button that copies a configured string to the system clipboard.",
            ui.Clipboard("copy").Value("gpui-net-shell").Tooltip("Copy the project name"),
            ui.Label("Press the button to copy the text.")
        );
}
