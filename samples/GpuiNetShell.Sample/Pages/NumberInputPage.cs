using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class NumberInputPage : GalleryPage<NumberInputPage.State>
{
    internal sealed class State
    {
        public string Value { get; set; } = "";
    }

    public override string Title => "NumberInput";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
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
