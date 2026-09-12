using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class StepperPage : GalleryPage
{
    private int _step = 1;

    public override string Title => "Stepper";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Stepper",
            "A typed progress stepper accepting only StepperItem children.",
            ui.Stepper("steps")
                .SelectedIndex(_step)
                .TextCenter()
                .OnChange(index =>
                {
                    _step = index;
                    Invalidate();
                })
                .Add(
                    ui.StepperItem().Add(ui.Label("Account")),
                    ui.StepperItem().Add(ui.Label("Profile")),
                    ui.StepperItem().Add(ui.Label("Review"))
                ),
            ui.Label($"Current step: {_step}")
        );
}
