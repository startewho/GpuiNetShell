using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SliderPage : GalleryPage<SliderPage.State>
{
    internal sealed class State
    {
        public double Value { get; set; } = 40;
    }

    public override string Title => "Slider";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Slider",
            "A retained numeric slider reporting value changes.",
            ui.Slider("volume")
                .Min(0)
                .Max(100)
                .Value(state.Value)
                .OnChange(value =>
                {
                    Update((s, c) =>
                    {
                        s.Value = value;
                        c.Notify();
                    });
                }),
            ui.Label($"Volume: {state.Value:0}"),
            ui.HStack(
                    ui.Slider("reversed").Min(0).Max(10).Value(7).Reverse(),
                    ui.Label("Reversed")
                )
                .Gap(12)
                .ItemsCenter()
        );
}
