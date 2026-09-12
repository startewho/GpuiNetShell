using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class WindowEffectsPage : GalleryPage
{
    private string _status = "(none)";

    public override string Title => "Window Effects";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Window Effects",
            "Real buttons that open native dialogs, alert dialogs, sheets, and notifications.",
            ui.HStack(
                    ui.Dialog("dialog", "Open Dialog")
                        .Title("Dialog")
                        .Content(
                            ui.VStack(
                                    ui.Label("Native dialog content."),
                                    ui.Text("Opened only from the button's click event.")
                                )
                                .Gap(4)
                        )
                        .OnOk(() =>
                        {
                            _status = "dialog ok";
                            Invalidate();
                        })
                        .OnCancel(() =>
                        {
                            _status = "dialog cancel";
                            Invalidate();
                        })
                        .OnClose(() =>
                        {
                            _status = "dialog closed";
                            Invalidate();
                        }),
                    ui.AlertDialog("alert", "Open Alert")
                        .Title("Delete file?")
                        .Description("This action cannot be undone.")
                        .ShowCancel()
                        .OnOk(() =>
                        {
                            _status = "alert ok";
                            Invalidate();
                        })
                        .OnCancel(() =>
                        {
                            _status = "alert cancel";
                            Invalidate();
                        }),
                    ui.Sheet("sheet", "Open Sheet")
                        .Title("Sheet")
                        .Placement("right")
                        .Content(
                            ui.VStack(
                                    ui.Label("Sheet content."),
                                    ui.Text("Anchored to the right edge.")
                                )
                                .Gap(4)
                        )
                        .OnClose(() =>
                        {
                            _status = "sheet closed";
                            Invalidate();
                        }),
                    ui.Notification("note", "Notify")
                        .Title("Saved")
                        .Message("Your changes were saved.")
                        .Type("success")
                        .OnClose(() =>
                        {
                            _status = "notification closed";
                            Invalidate();
                        })
                )
                .Gap(12),
            ui.Label($"Last effect: {_status}")
        );
}
