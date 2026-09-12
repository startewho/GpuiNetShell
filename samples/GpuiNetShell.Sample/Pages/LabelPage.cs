using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class LabelPage : GalleryPage
{
    public override string Title => "Label";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Label",
            "Labels with optional secondary text, masking, and highlights.",
            ui.Label("Default label"),
            ui.Label("With secondary").Secondary("supporting text"),
            ui.Label("Masked secret").Masked(),
            ui.Label("Highlighted fragment").Highlights("fragment")
        );
}
