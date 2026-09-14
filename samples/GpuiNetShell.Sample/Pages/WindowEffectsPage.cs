using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class WindowEffectsPage : GalleryPage<WindowEffectsPage.State>
{
    internal sealed class State
    {
        public string Status { get; set; } = "(none)";
    }

    public override string Title => "Window Effects";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
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
                        .OnOk(() => SetStatus("dialog ok"))
                        .OnCancel(() => SetStatus("dialog cancel"))
                        .OnClose(() => SetStatus("dialog closed")),
                    ui.AlertDialog("alert", "Open Alert")
                        .Title("Delete file?")
                        .Description("This action cannot be undone.")
                        .ShowCancel()
                        .OnOk(() => SetStatus("alert ok"))
                        .OnCancel(() => SetStatus("alert cancel")),
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
                        .OnClose(() => SetStatus("sheet closed")),
                    ui.Notification("note", "Notify")
                        .Title("Saved")
                        .Message("Your changes were saved.")
                        .Type("success")
                        .OnClose(() => SetStatus("notification closed"))
                )
                .Gap(12),
            ui.Label($"Last effect: {state.Status}")
        );

    private void SetStatus(string status) =>
        Update(
            (s, c) =>
            {
                s.Status = status;
                c.Notify();
            }
        );
}
