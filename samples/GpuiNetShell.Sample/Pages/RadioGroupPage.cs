using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class RadioGroupPage : GalleryPage<RadioGroupPage.State>
{
    internal sealed class State
    {
        public int Index { get; set; }
    }

    public override string Title => "Radio Group";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Radio Group",
            "A controlled radio set; selection is reported as a zero-based index.",
            ui.RadioGroup("theme-group")
                .SelectedIndex(state.Index)
                .OnChange(index =>
                {
                    Update((s, c) =>
                    {
                        s.Index = index;
                        c.Notify();
                    });
                })
                .Add(
                    ui.Radio("group-light").Label("Light"),
                    ui.Radio("group-dark").Label("Dark"),
                    ui.Radio("group-system").Label("System")
                ),
            ui.Label($"Selected index: {state.Index}")
        );
}
