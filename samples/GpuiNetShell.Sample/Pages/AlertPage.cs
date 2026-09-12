using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class AlertPage : GalleryPage
{
    public override string Title => "Alert";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Alert",
            "Message banners with semantic variants.",
            ui.InfoAlert("info", "A short informational message.").Title("Heads up"),
            ui.SuccessAlert("ok", "Everything completed successfully."),
            ui.WarningAlert("warn", "Check your connection.").Banner(),
            ui.ErrorAlert("err", "Something went wrong.").Title("Error")
        );
}
