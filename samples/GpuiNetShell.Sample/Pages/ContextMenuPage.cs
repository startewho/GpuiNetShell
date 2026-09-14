using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ContextMenuPage : GalleryPage<ContextMenuPage.State>
{
    internal sealed class State
    {
        public string Status { get; set; } = "(none)";
    }

    public override string Title => "Context Menu";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
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
                        .OnSelect(() => SetStatus("copy")),
                    ui.ContextMenuItem("Paste")
                        .OnSelect(() => SetStatus("paste")),
                    ui.ContextMenuSeparator(),
                    ui.ContextMenuItem("Disabled").Disabled(),
                    ui.ContextMenuItem("Checked").Checked()
                ),
            ui.Label($"Last action: {state.Status}")
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
