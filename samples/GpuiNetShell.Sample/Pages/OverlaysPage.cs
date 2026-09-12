using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class OverlaysPage : GalleryPage
{
    public override string Title => "Overlays";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Overlays",
            "Dialogs, sheets, and notifications are native layers over the content.",
            ui.HStack(
                    ui.Button("dialog")
                        .Label("Open dialog")
                        .Primary()
                        .OnClick(() =>
                            Application.OpenDialog(
                                "Native dialog",
                                "Dialogs, sheets, and notifications are rendered by the native root."
                            )
                        ),
                    ui.Button("sheet")
                        .Label("Open sheet")
                        .Secondary()
                        .OnClick(() =>
                            Application.OpenSheet(
                                SheetPlacement.Right,
                                "Details",
                                "A right-side sheet hosting native content."
                            )
                        ),
                    ui.Button("notify")
                        .Label("Notify")
                        .Success()
                        .OnClick(() =>
                            Application.PushNotification(
                                "Saved successfully.",
                                NotificationLevel.Success
                            )
                        )
                )
                .Gap(12)
                .ItemsCenter()
        );
}
