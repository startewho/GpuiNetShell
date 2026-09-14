using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class FocusPage : GalleryPage<FocusPage.State>
{
    internal sealed class State
    {
        public string Status { get; set; } = "(none)";
    }

    public override string Title => "Focus";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Focus",
            "The retained inputs report focus and blur; tab between them to see it.",
            ui.Input("focus-input")
                .Placeholder("Click or Tab to me")
                .OnFocus(() => Set("Input gained focus"))
                .OnBlur(() => Set("Input lost focus")),
            ui.Textarea("focus-area")
                .Placeholder("Then Tab to me")
                .H(96)
                .OnFocus(() => Set("Textarea gained focus"))
                .OnBlur(() => Set("Textarea lost focus")),
            ui.Label($"Last focus event: {state.Status}")
        );

    private void Set(string status) =>
        Update(
            (s, c) =>
            {
                s.Status = status;
                c.Notify();
            }
        );
}
