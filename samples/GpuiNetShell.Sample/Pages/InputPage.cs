using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class InputPage : GalleryPage
{
    private string _value = "";

    public override string Title => "Input";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Input",
            "A retained single-line text field reporting edits through a callback.",
            ui.Input("name")
                .Placeholder("Enter your name…")
                .OnChange(value =>
                {
                    _value = value;
                    Invalidate();
                }),
            ui.Label($"Value: {_value}"),
            ui.Input("disabled").Placeholder("Disabled").Disabled()
        );
}
