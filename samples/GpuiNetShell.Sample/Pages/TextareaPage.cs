using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class TextareaPage : GalleryPage<TextareaPage.State>
{
    internal sealed class State
    {
        public string Value { get; set; } = "";
    }

    public override string Title => "Textarea";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Textarea",
            "A retained multi-line text field.",
            ui.Textarea("notes")
                .Placeholder("Write a note…")
                .OnChange(value =>
                {
                    Update((s, c) =>
                    {
                        s.Value = value;
                        c.Notify();
                    });
                }),
            ui.Label($"Length: {state.Value.Length} characters")
        );
}
