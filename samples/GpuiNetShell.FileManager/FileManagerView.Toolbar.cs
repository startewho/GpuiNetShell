using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

internal sealed partial class FileManagerView
{
    /// <summary>
    /// The navigation row, with breadcrumbs. The second (command) row is
    /// optional and hidden by default; it is toggled from the settings popover.
    /// Search and settings live in the title bar.
    /// </summary>
    private Element BuildToolbar(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    )
    {
        var locationRow = ui.HStack(
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
                BuildAddressBar(ui, state, cx).Flex1().MinW(160)
            )
            .Gap(6)
            .ItemsCenter()
            .WFull();

        var rows = new List<Element> { locationRow };
        if (state.ShowToolbar)
        {
            rows.Add(
                ui.HStack(
                        ui.Div().Flex1(),
                        ViewToggle(
                            ui,
                            "fm-view-details",
                            state,
                            cx,
                            "icons/list.svg",
                            FileView.Details
                        ),
                        ViewToggle(
                            ui,
                            "fm-view-icons",
                            state,
                            cx,
                            "icons/layout-grid.svg",
                            FileView.LargeIcons
                        )
                    )
                    .Gap(6)
                    .ItemsCenter()
                    .WFull()
            );
        }

        return ui.VStack(rows.ToArray())
            .Gap(6)
            .P(8)
            .WFull()
            .BorderB(1)
            .BorderColor(Divider);
    }

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
        string icon,
        FileView view
    )
    {
        var selected = state.View == view;
        var glyph = ui.Icon(icon).Size(ControlSize.Medium);
        if (selected)
        {
            glyph.Color("#ffffff");
        }
        var button = ui.Div(glyph).Size(32).ItemsCenter().JustifyCenter().Rounded(6);
        if (selected)
        {
            button.Bg(state.AccentHex);
        }
        button.OnClick(id, () => SetView(view));
        return button;
    }
}
