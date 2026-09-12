using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ContextMenuPage : GalleryPage
{
    private string _status = "(none)";

    public override string Title => "Context Menu";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Context Menu",
            "Right-click the target to open a menu attached to the element.",
            ui.ContextMenu("context-menu")
                .Target(
                    ui.Div(ui.Label("Right-click me"))
                        .P(16)
                        .Border(1)
                        .Rounded(6)
                        .Bg("#f4f4f5")
                )
                .Items(
                    ui.ContextMenuItem("Copy")
                        .OnSelect(() =>
                        {
                            _status = "copy";
                            Invalidate();
                        }),
                    ui.ContextMenuItem("Paste")
                        .OnSelect(() =>
                        {
                            _status = "paste";
                            Invalidate();
                        }),
                    ui.ContextMenuSeparator(),
                    ui.ContextMenuItem("Disabled").Disabled(),
                    ui.ContextMenuItem("Checked").Checked()
                ),
            ui.Label($"Last action: {_status}")
        );
}
