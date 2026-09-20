using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>The file list pane (details and large icons) and the status bar.</summary>
internal sealed partial class FileManagerView
{
    private Element BuildContentPane(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    ) =>
        ui.Div(BuildBody(ui, state, cx))
            .Flex1()
            .MinW(0)
            .MinH(0)
            .HFull()
            .Flex()
            .FlexColumn();

    private Element BuildBody(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    )
    {
        if (state.Loading)
        {
            return ui.Div(
                    ui.VStack(ui.Spinner().Size(ControlSize.Large), ui.Label("正在加载…").TextSize(12))
                        .Gap(8)
                        .ItemsCenter()
                )
                .Full()
                .Flex()
                .ItemsCenter()
                .JustifyCenter();
        }

        if (state.Visible.Count == 0)
        {
            var message = state.Error;
            if (string.IsNullOrEmpty(message))
            {
                message = state.SearchText.Length > 0 ? "没有匹配的项目" : "此文件夹为空";
            }
            return ui.Div(ui.Label(message).TextSize(13))
                .Full()
                .Flex()
                .ItemsCenter()
                .JustifyCenter();
        }

        return state.View == FileView.Details
            ? BuildDetails(ui, state, cx)
            : BuildIcons(ui, state, cx);
    }

    private Element BuildDetails(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    ) =>
        ui.VStack(
                ui.HStack(
                        ui.Div().W(24),
                        SortHeader(ui, state, cx, "名称", "name", 0),
                        SortHeader(ui, state, cx, "修改日期", "modified", 150),
                        SortHeader(ui, state, cx, "类型", "type", 110),
                        SortHeader(ui, state, cx, "大小", "size", 90)
                    )
                    .Gap(8)
                    .ItemsCenter()
                    .WFull()
                    .H(30)
                    .Px(8)
                    .BorderB(1)
                    .BorderColor(Divider),
                ui.VirtualList("fm-details", state.Visible.Count)
                    .ItemSize(30)
                    .Vertical()
                    .Flex1()
                    .MinW(0)
                    .MinH(0)
                    .OnSelect(index => OnEntryActivate(cx, state, index))
                    .OnMiddleClick(index => OpenInNewTab(index))
                    .RowMenu(
                        ui.ContextMenuItem("打开")
                            .OnSelectStable("fm-row-open", row => OpenIndex(cx, state, row)),
                        ui.ContextMenuItem("在新标签页中打开")
                            .OnSelectStable("fm-row-newtab", row => OpenInNewTab(row)),
                        ui.ContextMenuItem("在文件资源管理器中显示")
                            .OnSelectStable("fm-row-reveal", row => RevealIndex(state, row)),
                        ui.ContextMenuSeparator(),
                        ui.ContextMenuItem("刷新")
                            .OnSelectStable("fm-row-refresh", () => Reload(cx, state))
                    )
                    .RenderItem((ctx, index) => DetailsRow(ctx, state, index))
            )
            .Flex1()
            .MinH(0)
            .WFull();

    private Element SortHeader(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx,
        string label,
        string key,
        int width
    )
    {
        var children = new List<Element> { ui.Label(label).TextSize(12).FontMedium() };
        if (state.SortKey == key)
        {
            children.Add(
                ui.Icon(state.SortAscending ? "icons/chevron-up.svg" : "icons/chevron-down.svg")
                    .Size(ControlSize.Small)
            );
        }

        var cell = ui.HStack(children.ToArray()).Gap(4).ItemsCenter().H(28).Px(8);
        if (width > 0)
        {
            cell.W(width);
        }
        else
        {
            cell.Flex1();
        }
        cell.OnClick(
            "fm-sort-" + key,
            () =>
                Update(
                    (s, c) =>
                    {
                        s.SortBy(key);
                        c.Notify();
                    }
                )
        );
        return cell;
    }

    private Element DetailsRow(RenderContext ui, FileManagerState state, int index)
    {
        var entry = state.Visible[index];
        var selected = index == state.SelectedIndex;

        var name = ui.Label(entry.Name).TextSize(13);
        var modified = ui.Label(Formatting.Date(entry.Modified)).TextSize(12);
        var type = ui.Label(Formatting.TypeName(entry)).TextSize(12);
        if (selected)
        {
            name.TextColor("#ffffff");
            modified.TextColor("#ffffff");
            type.TextColor("#ffffff");
        }

        // A folder has no size; the native Label rejects an empty string, so the
        // cell becomes an empty spacer of the same width instead.
        var sizeText = Formatting.Size(entry);
        Element sizeCell;
        if (sizeText.Length == 0)
        {
            sizeCell = ui.Div().W(90);
        }
        else
        {
            var size = ui.Label(sizeText).TextSize(12).TextAlign(TextAlignKind.Right);
            if (selected)
            {
                size.TextColor("#ffffff");
            }
            sizeCell = size.W(90);
        }

        var row = ui.HStack(
                ui.Icon(FileIcons.For(entry))
                    .Size(ControlSize.Medium)
                    .Color(selected ? "#ffffff" : IconColor(entry)),
                ui.Div(name).Flex1().MinW(0),
                modified.W(150),
                type.W(110),
                sizeCell
            )
            .Gap(8)
            .ItemsCenter()
            .WFull()
            .H(30)
            .Px(8);
        if (selected)
        {
            row.Bg(state.AccentHex);
        }
        return row;
    }

    private Element BuildIcons(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    )
    {
        // A wrapping row of fixed-width cells: the column count follows the pane
        // width, so the grid adapts to the window without knowing its size.
        var cells = new List<Element>(state.Visible.Count);
        for (var index = 0; index < state.Visible.Count; index++)
        {
            cells.Add(IconCell(ui, state, cx, index));
        }

        var grid = ui.HStack(cells.ToArray()).Wrap().Gap(2).Px(4).Pt(4).WFull();
        return ui.Scroll("fm-grid")
            .Axis(ScrollAxis.Vertical)
            .Add(grid)
            .Flex1()
            .MinW(0)
            .MinH(0)
            .WFull();
    }

    private Element IconCell(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx,
        int index
    )
    {
        var entry = state.Visible[index];
        var selected = index == state.SelectedIndex;

        var name = ui.Label(entry.Name).TextSize(12).TextAlign(TextAlignKind.Center).LineClamp(2);
        if (selected)
        {
            name.TextColor("#ffffff");
        }

        var cell = ui.VStack(
                ui.Icon(FileIcons.For(entry))
                    .Size(40)
                    .Color(selected ? "#ffffff" : IconColor(entry)),
                name
            )
            .Gap(6)
            .ItemsCenter()
            .JustifyCenter()
            .W(100)
            .H(96)
            .P(6)
            .Rounded(6);
        if (selected)
        {
            cell.Bg(state.AccentHex);
        }
        cell.OnClick(
            "fm-icon-" + entry.FullPath,
            () => OnEntryActivate(cx, state, index)
        );
        cell.OnMouseDown(pointer =>
        {
            if (pointer.Button == 2)
            {
                OpenInNewTab(index);
            }
        });
        return cell;
    }

    private static string IconColor(FileEntry entry) => entry.IsDirectory ? "amber-500" : "#808080";

    private Element BuildStatusBar(RenderContext ui, FileManagerState state)
    {
        var selected =
            state.SelectedIndex >= 0 && state.SelectedIndex < state.Visible.Count
                ? state.Visible[state.SelectedIndex].Name
                : string.Empty;
        var bar = ui.StatusBar()
            .LeftContent(ui.Label($"{state.Visible.Count} 个项目").TextSize(12))
            .RightContent(ui.Label(state.CurrentPath).TextSize(12));
        if (selected.Length > 0)
        {
            bar.Add(ui.Label(selected).TextSize(12));
        }
        return bar;
    }
}
