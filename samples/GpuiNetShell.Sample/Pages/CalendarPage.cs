using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class CalendarPage : GalleryPage
{
    private string _date = "";

    public override string Title => "Calendar";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Calendar",
            "A retained calendar reporting the selected date (ISO).",
            ui.Calendar("calendar").NumberOfMonths(2).OnChange(date =>
            {
                _date = date;
                Invalidate();
            }),
            ui.Label($"Selected date: {_date}")
        );
}
