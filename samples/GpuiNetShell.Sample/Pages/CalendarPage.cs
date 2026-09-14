using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class CalendarPage : GalleryPage<CalendarPage.State>
{
    internal sealed class State
    {
        public string Date { get; set; } = "";
    }

    public override string Title => "Calendar";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Calendar",
            "A retained calendar reporting the selected date (ISO).",
            ui.Calendar("calendar").NumberOfMonths(2).OnChange(date =>
            {
                Update((s, c) =>
                {
                    s.Date = date;
                    c.Notify();
                });
            }),
            ui.Label($"Selected date: {state.Date}")
        );
}
