using System.Diagnostics;
using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>
/// A Windows 11 style file manager. The root view owns an
/// <see cref="Entity{T}"/>; the whole interface is rendered from that entity, so
/// a change repaints only this subtree and not the window.
/// </summary>
[GpuiCallbacks]
internal sealed partial class FileManagerView : View
{
    /// <summary>A translucent separator that reads on both light and dark backgrounds.</summary>
    private const string Divider = "#80808055";

    private const double IconCellHeight = 108;

    private readonly GpuiApplication _application;
    private Entity<FileManagerState>? _entity;
    private bool _themeApplied;

    public FileManagerView(GpuiApplication application, string? initialPath)
    {
        _application = application;
        var path =
            initialPath is { Length: > 0 } && Directory.Exists(initialPath)
                ? initialPath
                : FileSystemService.DefaultPath();

        var listing = FileSystemService.ListDirectory(path);
        Entity.Update(
            (state, _) =>
            {
                state.SetListing(listing.Path, listing);
                state.PushHistory(state.CurrentPath);
                BuildTree(state);
            }
        );
    }

    private Entity<FileManagerState> Entity =>
        _entity ??= _application.New<FileManagerState>(_ => new FileManagerState());

    protected override Element Render(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        EnsureTheme();
        return ui.Child(Entity, PageToken);
    }

    /// <summary>Applies the configured accent once, after the window exists.</summary>
    private void EnsureTheme()
    {
        if (_themeApplied)
        {
            return;
        }
        _themeApplied = true;
        var state = Entity.Read();
        _application.SetTheme(state.Mode, ThemePresets.Palette(state.AccentHex));
    }

    private void Update(Action<FileManagerState, Context<FileManagerState>> update) =>
        Entity.Update(update);

    // -- Page ---------------------------------------------------------------

    [GpuiCallback("Page")]
    private Element RenderPage(
        FileManagerState state,
        RenderContext ui,
        Context<FileManagerState> cx
    ) =>
        ui.VStack(
                BuildToolbar(ui, state, cx),
                ui.HStack(BuildTreePane(ui, state, cx), BuildContentPane(ui, state, cx))
                    .Gap(0)
                    .Flex1()
                    .MinH(0),
                BuildStatusBar(ui, state)
            )
            .Full();

    // -- Toolbar ------------------------------------------------------------

    private Element BuildToolbar(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    )
    {
        // Two rows, like Windows 11: navigation and the address bar on top, the
        // command strip below. One row cannot hold every control at the default
        // width without clipping the trailing buttons.
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
                ui.Input("fm-address")
                    .Placeholder("地址")
                    .Value(state.AddressText)
                    .OnChange(text =>
                        Update(
                            (s, c) =>
                            {
                                s.AddressText = text;
                                c.Notify();
                            }
                        )
                    )
                    .Flex1()
                    .MinW(120),
                ui.Button("fm-go")
                    .Label("转到")
                    .Compact()
                    .OnClick(() => NavigateTo(cx, state.AddressText, true))
            )
            .Gap(6)
            .ItemsCenter()
            .WFull();

        var commandRow = ui.HStack(
                ui.Input("fm-search")
                    .Placeholder("搜索")
                    .W(200)
                    .OnChange(text =>
                        Update(
                            (s, c) =>
                            {
                                s.SearchText = text;
                                s.Recompute();
                                c.Notify();
                            }
                        )
                    ),
                ui.Div().Flex1(),
                ViewToggle(ui, "fm-view-details", state, cx, "icons/list.svg", FileView.Details),
                ViewToggle(ui, "fm-view-icons", state, cx, "icons/layout-grid.svg", FileView.LargeIcons),
                BuildThemeButton(ui, state, cx)
            )
            .Gap(6)
            .ItemsCenter()
            .WFull();

        return ui.VStack(locationRow, commandRow)
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
        button.OnClick(id, () => SetView(cx, view));
        return button;
    }

    private Element BuildThemeButton(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    ) =>
        ui.Popover("fm-theme", "主题")
            .Open(state.ThemeOpen)
            .CardAnchor(PopoverAnchor.BottomRight)
            .OverlayClosable()
            .OnOpenChange(open =>
                Update(
                    (s, c) =>
                    {
                        s.ThemeOpen = open;
                        c.Notify();
                    }
                )
            )
            .Content(BuildThemePanel(ui, state, cx));

    private Element BuildThemePanel(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    )
    {
        var swatches = new List<Element>(ThemePresets.Accents.Count);
        foreach (var accent in ThemePresets.Accents)
        {
            var swatch = ui.Div().Size(24).Rounded(12).Bg(accent.Hex);
            if (accent.Key == state.AccentKey)
            {
                swatch.Border(2).BorderColor("gray-700");
            }
            var selected = accent;
            swatch.OnClick(
                "fm-accent-" + selected.Key,
                () => SetAccent(cx, state, selected.Hex, selected.Key)
            );
            swatches.Add(swatch);
        }

        return ui.VStack(
                ui.Label("主题模式").FontSemibold().TextSize(12),
                ui.HStack(
                        ModeButton(ui, state, cx, "浅色", ThemeMode.Light),
                        ModeButton(ui, state, cx, "深色", ThemeMode.Dark),
                        ModeButton(ui, state, cx, "跟随系统", ThemeMode.System)
                    )
                    .Gap(6),
                ui.Label("强调色").FontSemibold().TextSize(12),
                ui.HStack(swatches.ToArray()).Gap(8).ItemsCenter(),
                ui.ColorPicker("fm-accent")
                    .Label("自定义强调色")
                    .OnChange(hex => SetAccent(cx, state, hex, key: null))
            )
            .Gap(10)
            .P(12)
            .W(320);
    }

    private Element ModeButton(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx,
        string label,
        ThemeMode mode
    )
    {
        var id = "fm-mode-" + mode;
        return ui.Button(id)
            .Label(label)
            .Selected(state.Mode == mode)
            .OnClick(() => SetMode(cx, state, mode));
    }

    // -- Tree ---------------------------------------------------------------

    private Element BuildTreePane(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    )
    {
        var rows = new List<Element>();
        foreach (var root in state.TreeRoots)
        {
            Flatten(ui, state, cx, root, 0, rows);
        }

        return ui.Div(
                ui.HStack(ui.Icon(FileIcons.Tree).Size(ControlSize.Small), ui.Label("导航").TextSize(12))
                    .Gap(6)
                    .ItemsCenter()
                    .P(8),
                ui.Scroll("fm-tree").Flex1().MinH(0).Axis(ScrollAxis.Vertical).Add(rows.ToArray())
            )
            .W(240)
            .HFull()
            .Flex()
            .FlexColumn()
            .BorderR(1)
            .BorderColor(Divider);
    }

    private void Flatten(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx,
        TreeNode node,
        int depth,
        List<Element> rows
    )
    {
        rows.Add(TreeRow(ui, state, cx, node, depth));
        if (!node.Expanded)
        {
            return;
        }
        foreach (var child in node.Children)
        {
            Flatten(ui, state, cx, child, depth + 1, rows);
        }
    }

    private Element TreeRow(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx,
        TreeNode node,
        int depth
    )
    {
        var selected =
            node.Path.Length > 0
            && string.Equals(node.Path, state.CurrentPath, StringComparison.OrdinalIgnoreCase);

        var label = ui.Label(node.Label).TextSize(13);
        if (selected)
        {
            label.TextColor("#ffffff");
        }

        var row = ui.HStack(
                ui.Icon(node.Expanded ? "icons/chevron-down.svg" : "icons/chevron-right.svg")
                    .Size(ControlSize.Small)
                    .Color(selected ? "#ffffff" : "#808080"),
                ui.Icon(node.Icon)
                    .Size(ControlSize.Medium)
                    .Color(selected ? "#ffffff" : "amber-500"),
                label
            )
            .Gap(6)
            .ItemsCenter()
            .WFull()
            .H(26)
            .Pl(6 + depth * 14)
            .Pr(6)
            .Rounded(4);
        if (selected)
        {
            row.Bg(state.AccentHex);
        }
        row.OnClick("fm-tree-" + node.Id, () => OnTreeClick(cx, node));
        return row;
    }

    private void OnTreeClick(Context<FileManagerState> cx, TreeNode node)
    {
        var willExpand = !node.Expanded;
        var needsLoad = willExpand && !node.Loaded;
        Update(
            (s, c) =>
            {
                node.Expanded = willExpand;
                if (needsLoad)
                {
                    node.Loaded = true;
                }
                c.Notify();
            }
        );
        if (needsLoad && node.Path.Length > 0)
        {
            EnsureChildren(cx, node);
        }
        if (node.Path.Length > 0)
        {
            NavigateTo(cx, node.Path, recordHistory: true);
        }
    }

    private static void EnsureChildren(Context<FileManagerState> cx, TreeNode node)
    {
        var path = node.Path;
        cx.Spawn(
            _ => Task.FromResult(FileSystemService.ListSubdirectories(path)),
            (state, children, context) =>
            {
                node.Children.Clear();
                node.Children.AddRange(children);
                context.Notify();
            }
        );
    }

    private static void BuildTree(FileManagerState state)
    {
        state.TreeRoots.Clear();

        var quick = new TreeNode
        {
            Id = "__quick",
            Label = "快速访问",
            Icon = FileIcons.QuickAccess,
            Expanded = true,
            Loaded = true,
        };
        quick.Children.AddRange(FileSystemService.QuickAccess());

        var computer = new TreeNode
        {
            Id = "__pc",
            Label = "此电脑",
            Icon = "icons/monitor.svg",
            Expanded = true,
            Loaded = true,
        };
        computer.Children.AddRange(FileSystemService.Drives());

        state.TreeRoots.Add(quick);
        state.TreeRoots.Add(computer);
    }

    // -- Content ------------------------------------------------------------

    private Element BuildContentPane(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    ) =>
        ui.Div(BuildBody(ui, state, cx))
            .Flex1()
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
                    .MinH(0)
                    .OnSelect(index => OnEntryActivate(cx, state, index))
                    .RowMenu(
                        ui.ContextMenuItem("打开").OnSelect(row => OpenIndex(cx, state, row)),
                        ui.ContextMenuItem("在文件资源管理器中显示")
                            .OnSelect(row => RevealIndex(state, row)),
                        ui.ContextMenuSeparator(),
                        ui.ContextMenuItem("刷新").OnSelect(_ => Reload(cx, state))
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
        var columns = Math.Max(1, state.IconColumns);
        var rows = (state.Visible.Count + columns - 1) / columns;
        return ui.VirtualList("fm-icons", rows)
            .ItemSize(IconCellHeight)
            .Vertical()
            .Flex1()
            .MinH(0)
            .WFull()
            .RenderItem((ctx, rowIndex) => IconRow(ctx, state, cx, rowIndex, columns));
    }

    private Element IconRow(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx,
        int rowIndex,
        int columns
    )
    {
        var cells = new List<Element>(columns);
        for (var column = 0; column < columns; column++)
        {
            var index = rowIndex * columns + column;
            if (index >= state.Visible.Count)
            {
                break;
            }
            cells.Add(IconCell(ui, state, cx, index));
        }
        return ui.HStack(cells.ToArray()).Gap(0).WFull().ItemsStart().Px(4).Pt(4);
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
        return cell;
    }

    private static string IconColor(FileEntry entry) => entry.IsDirectory ? "amber-500" : "#808080";

    // -- Status bar ---------------------------------------------------------

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

    // -- Commands -----------------------------------------------------------

    private void SetView(Context<FileManagerState> cx, FileView view) =>
        Update(
            (s, c) =>
            {
                s.View = view;
                c.Notify();
            }
        );

    private void SetMode(Context<FileManagerState> cx, FileManagerState state, ThemeMode mode)
    {
        _application.SetTheme(mode, ThemePresets.Palette(state.AccentHex));
        Update(
            (s, c) =>
            {
                s.Mode = mode;
                c.Notify();
            }
        );
    }

    private void SetAccent(
        Context<FileManagerState> cx,
        FileManagerState state,
        string hex,
        string? key
    )
    {
        _application.SetTheme(state.Mode, ThemePresets.Palette(hex));
        Update(
            (s, c) =>
            {
                s.AccentHex = hex;
                if (key is not null)
                {
                    s.AccentKey = key;
                }
                c.Notify();
            }
        );
    }

    private void GoBack(Context<FileManagerState> cx)
    {
        string? target = null;
        Update(
            (s, c) =>
            {
                target = s.Back();
                c.Notify();
            }
        );
        if (target is not null)
        {
            NavigateTo(cx, target, recordHistory: false);
        }
    }

    private void GoForward(Context<FileManagerState> cx)
    {
        string? target = null;
        Update(
            (s, c) =>
            {
                target = s.Forward();
                c.Notify();
            }
        );
        if (target is not null)
        {
            NavigateTo(cx, target, recordHistory: false);
        }
    }

    private void GoUp(Context<FileManagerState> cx, FileManagerState state)
    {
        var parent = FileSystemService.Parent(state.CurrentPath);
        if (parent is not null)
        {
            NavigateTo(cx, parent, recordHistory: true);
        }
    }

    private void Reload(Context<FileManagerState> cx, FileManagerState state) =>
        NavigateTo(cx, state.CurrentPath, recordHistory: false);

    private void NavigateTo(Context<FileManagerState> cx, string path, bool recordHistory)
    {
        if (string.IsNullOrWhiteSpace(path))
        {
            return;
        }
        var target = path.Trim();
        Update(
            (s, c) =>
            {
                s.Loading = true;
                s.Error = null;
                c.Notify();
            }
        );
        cx.Spawn(
            _ => Task.FromResult(FileSystemService.ListDirectory(target)),
            (state, listing, context) =>
            {
                state.Loading = false;
                if (listing.Error is not null && listing.Entries.Count == 0)
                {
                    state.Error = listing.Error;
                    context.Notify();
                    return;
                }
                state.SetListing(listing.Path, listing);
                if (recordHistory)
                {
                    state.PushHistory(listing.Path);
                }
                context.Notify();
            }
        );
    }

    private void OnEntryActivate(
        Context<FileManagerState> cx,
        FileManagerState state,
        int index
    )
    {
        var now = Environment.TickCount64;
        var doubleClick =
            index == state.SelectedIndex
            && index == state.LastClickIndex
            && now - state.LastClickTicks < 450;
        Update(
            (s, c) =>
            {
                s.SelectedIndex = index;
                s.LastClickIndex = index;
                s.LastClickTicks = now;
                c.Notify();
            }
        );
        if (doubleClick)
        {
            OpenIndex(cx, state, index);
        }
    }

    private void OpenIndex(Context<FileManagerState> cx, FileManagerState state, int index)
    {
        if (index < 0 || index >= state.Visible.Count)
        {
            return;
        }
        var entry = state.Visible[index];
        if (entry.IsDirectory)
        {
            NavigateTo(cx, entry.FullPath, recordHistory: true);
        }
        else
        {
            OpenWithShell(entry.FullPath);
        }
    }

    private static void RevealIndex(FileManagerState state, int index)
    {
        if (index < 0 || index >= state.Visible.Count)
        {
            return;
        }
        var path = state.Visible[index].FullPath;
        try
        {
            Process.Start(
                new ProcessStartInfo("explorer.exe", $"/select,\"{path}\"")
                {
                    UseShellExecute = true,
                }
            );
        }
        catch (Exception)
        {
            // Shell launch is best-effort.
        }
    }

    private static void OpenWithShell(string path)
    {
        try
        {
            Process.Start(new ProcessStartInfo(path) { UseShellExecute = true });
        }
        catch (Exception)
        {
            // Shell launch is best-effort.
        }
    }
}
