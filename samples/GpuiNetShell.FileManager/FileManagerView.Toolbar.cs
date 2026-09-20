using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

internal sealed partial class FileManagerView
{
    /// <summary>
    /// The navigation row: history buttons, the address bar, and the view
    /// toggles (list and grid). Search and settings live in the title bar.
    /// </summary>
    private Element BuildToolbar(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    ) =>
        ui.HStack(
                IconButton(ui, "fm-back", "icons/arrow-left.svg", state.CanGoBack, () => GoBack(cx)),
                IconButton(
                    ui,
                    "fm-forward",
                    "icons/arrow-right.svg",
                    state.CanGoForward,
                    () => GoForward(cx)
                ),
                IconButton(
                    ui,
                    "fm-up",
                    "icons/arrow-up.svg",
                    FileSystemService.Parent(state.CurrentPath) is not null,
                    () => GoUp(cx, state)
                ),
                IconButton(ui, "fm-refresh", "icons/refresh-cw.svg", true, () => Reload(cx, state)),
                BuildAddressBar(ui, state, cx).Flex1().MinW(160),
                ViewToggle(ui, "fm-view-details", state, cx, "☰", FileView.Details),
                ViewToggle(ui, "fm-view-grid", state, cx, "▦", FileView.LargeIcons),
                PreviewToggle(ui, state)
            )
            .Gap(6)
            .ItemsCenter()
            .WFull()
            .P(8)
            .BorderB(1)
            .BorderColor(Divider);

    private static Element IconButton(
        RenderContext ui,
        string id,
        string icon,
        bool enabled,
        Action onClick
    )
    {
        var button = ui.Div(ui.Icon(icon).Size(ControlSize.Medium))
            .Size(32)
            .Flex()
            .ItemsCenter()
            .JustifyCenter()
            .Rounded(6);
        if (enabled)
        {
            button.OnClick(id, onClick);
        }
        else
        {
            button.Opacity(0.35);
        }
        return button;
    }

    private Element ViewToggle(
        RenderContext ui,
        string id,
        FileManagerState state,
        Context<FileManagerState> cx,
        string glyph,
        FileView view
    )
    {
        var selected = state.View == view;
        var button = ui.Div(ui.Label(glyph).TextSize(15))
            .Size(32)
            .Flex()
            .ItemsCenter()
            .JustifyCenter()
            .Rounded(6);
        if (selected)
        {
            button.Bg(NeutralSelection);
        }
        button.OnClick(id, () => SetView(view));
        return button;
    }

    /// <summary>Toggles the right-hand preview pane.</summary>
    private Element PreviewToggle(RenderContext ui, FileManagerState state)
    {
        var button = ui.Div(ui.Label("◫").TextSize(15))
            .Size(32)
            .Flex()
            .ItemsCenter()
            .JustifyCenter()
            .Rounded(6);
        if (state.ShowPreview)
        {
            button.Bg(NeutralSelection);
        }
        button.OnClick("fm-preview-toggle", TogglePreview);
        return button;
    }
}
