using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SliderPage : GalleryPage
{
    private double _value = 40;

    public override string Title => "Slider";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Slider",
            "A retained numeric slider reporting value changes.",
            ui.Slider("volume").Min(0).Max(100).Value(_value).OnChange(value =>
            {
                _value = value;
                Invalidate();
            }),
            ui.Label($"Volume: {_value:0}"),
            ui.HStack(
                    ui.Slider("reversed").Min(0).Max(10).Value(7).Reverse(),
                    ui.Label("Reversed")
                )
                .Gap(12)
                .ItemsCenter()
        );
}
