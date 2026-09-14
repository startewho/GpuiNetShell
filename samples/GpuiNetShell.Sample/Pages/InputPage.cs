using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class InputPage : GalleryPage<InputPage.State>
{
    internal sealed class State
    {
        public string Value { get; set; } = "";
    }

    public override string Title => "Input";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Input",
            "A retained single-line text field reporting edits through a callback.",
            ui.Input("name")
                .Placeholder("Enter your name…")
                .OnChange(value =>
                {
                    Update((s, c) => { s.Value = value; c.Notify(); });
                }),
            ui.Label($"Value: {state.Value}"),
            ui.Input("disabled").Placeholder("Disabled").Disabled()
        );
}
