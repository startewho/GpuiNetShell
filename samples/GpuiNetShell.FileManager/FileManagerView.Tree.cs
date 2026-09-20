using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>The navigation tree pane.</summary>
internal sealed partial class FileManagerView
{
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
}
