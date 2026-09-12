using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class NumberInputPage : GalleryPage
{
    private string _value = "";

    public override string Title => "NumberInput";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "NumberInput",
            "A retained numeric text field with increment/decrement controls.",
            ui.NumberInput("amount")
                .Placeholder("0")
                .OnChange(value =>
                {
                    _value = value;
                    Invalidate();
                }),
            ui.Label($"Value: {_value}")
        );
}
