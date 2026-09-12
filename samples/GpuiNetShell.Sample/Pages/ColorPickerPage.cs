using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ColorPickerPage : GalleryPage
{
    private string _color = "";

    public override string Title => "ColorPicker";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "ColorPicker",
            "A retained color picker reporting the committed color as hex.",
            ui.ColorPicker("accent").Label("Accent color").OnChange(hex =>
            {
                _color = hex;
                Invalidate();
            }),
            ui.Label($"Color: {_color}")
        );
}
