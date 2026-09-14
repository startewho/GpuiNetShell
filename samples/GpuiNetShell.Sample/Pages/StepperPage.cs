using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class StepperPage : GalleryPage<StepperPage.State>
{
    internal sealed class State
    {
        public int Step { get; set; } = 1;
    }

    public override string Title => "Stepper";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Stepper",
            "A typed progress stepper accepting only StepperItem children.",
            ui.Stepper("steps")
                .SelectedIndex(state.Step)
                .TextCenter()
                .OnChange(index =>
                {
                    Update((s, c) =>
                    {
                        s.Step = index;
                        c.Notify();
                    });
                })
                .Add(
                    ui.StepperItem().Add(ui.Label("Account")),
                    ui.StepperItem().Add(ui.Label("Profile")),
                    ui.StepperItem().Add(ui.Label("Review"))
                ),
            ui.Label($"Current step: {state.Step}")
        );
}
