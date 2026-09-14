using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TextareaPage : GalleryPage<TextareaPage.State>
{
    internal sealed class State
    {
        public string Value { get; set; } = "";
    }

    public override string Title => "Textarea";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
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
