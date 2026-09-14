using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class NumberInputPage : GalleryPage<NumberInputPage.State>
{
    internal sealed class State
    {
        public string Value { get; set; } = "";
    }

    public override string Title => "NumberInput";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "NumberInput",
            "A retained numeric text field with increment/decrement controls.",
            ui.NumberInput("amount")
                .Placeholder("0")
                .OnChange(value =>
                {
                    Update((s, c) => { s.Value = value; c.Notify(); });
                }),
            ui.Label($"Value: {state.Value}")
        );
}
