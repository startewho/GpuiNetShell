using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class DatePickerPage : GalleryPage<DatePickerPage.State>
{
    internal sealed class State
    {
        public string Date { get; set; } = "";
    }

    public override string Title => "DatePicker";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "DatePicker",
            "A retained single-date picker reporting the chosen date (ISO).",
            ui.HStack(
                    ui.DatePicker("due").Placeholder("Pick a due date").OnChange(date =>
                    {
                        Update((s, c) =>
                        {
                            s.Date = date;
                            c.Notify();
                        });
                    }),
                    ui.DatePicker("locked").Placeholder("Disabled").Disabled()
                )
                .Gap(12)
                .ItemsCenter(),
            ui.Label($"Chosen date: {state.Date}")
        );
}
