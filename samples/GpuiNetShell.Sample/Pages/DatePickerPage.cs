using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class DatePickerPage : GalleryPage
{
    private string _date = "";

    public override string Title => "DatePicker";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "DatePicker",
            "A retained single-date picker reporting the chosen date (ISO).",
            ui.HStack(
                    ui.DatePicker("due").Placeholder("Pick a due date").OnChange(date =>
                    {
                        _date = date;
                        Invalidate();
                    }),
                    ui.DatePicker("locked").Placeholder("Disabled").Disabled()
                )
                .Gap(12)
                .ItemsCenter(),
            ui.Label($"Chosen date: {_date}")
        );
}
