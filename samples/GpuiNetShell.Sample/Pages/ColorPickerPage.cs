using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ColorPickerPage : GalleryPage<ColorPickerPage.State>
{
    internal sealed class State
    {
        public string Color { get; set; } = "";
    }

    public override string Title => "ColorPicker";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "ColorPicker",
            "A retained color picker reporting the committed color as hex.",
            ui.ColorPicker("accent").Label("Accent color").OnChange(hex =>
            {
                Update((s, c) =>
                {
                    s.Color = hex;
                    c.Notify();
                });
            }),
            ui.Label($"Color: {state.Color}")
        );
}
