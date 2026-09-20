using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>
/// The address bar: a Windows Explorer style breadcrumb built from C# elements.
/// Each segment navigates when clicked, and a chevron after it is a native
/// dropdown menu listing that folder's subfolders. The menu is anchored to the
/// chevron, so it opens right where it was clicked.
/// </summary>
internal sealed partial class FileManagerView
{
    /// <summary>How many subfolders a breadcrumb dropdown lists.</summary>
    private const int CrumbMenuLimit = 200;

    private Element BuildAddressBar(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    )
    {
        var segments = PathSegments(state.CurrentPath);
        var children = new List<Element>(segments.Count * 2);
        for (var index = 0; index < segments.Count; index++)
        {
            var (label, path) = segments[index];
            var crumb = ui.Div(ui.Label(label).TextSize(12)).Px(6).Py(4).Rounded(4);
            crumb.OnClick("fm-crumb-nav-" + index, () => NavigateTo(cx, path, true));
            children.Add(crumb);

            if (index < segments.Count - 1)
            {
                children.Add(BuildCrumbMenu(ui, state, cx, path, index));
            }
        }

        return ui.HStack(children.ToArray()).Gap(1).ItemsCenter().H(32);
    }

    private Element BuildCrumbMenu(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx,
        string path,
        int index
    )
    {
        var menu = ui.DropdownMenu("fm-crumb-dd-" + index, "›").Ghost().H(28).Px(2);
        if (state.CrumbChildren.TryGetValue(path, out var children))
        {
            var count = Math.Min(children.Count, CrumbMenuLimit);
            for (var i = 0; i < count; i++)
            {
                var child = children[i];
                menu.ItemStable(
                    "fm-crumb:" + child.Path,
                    child.Label,
                    () => NavigateTo(cx, child.Path, true)
                );
            }
        }
        return menu;
    }

    /// <summary>
    /// Loads the subfolders of every breadcrumb segment for the current path, so
    /// a chevron dropdown has items ready when it opens. Marked loaded up front
    /// so a repaint does not spawn the same listing twice.
    /// </summary>
    private static void PreloadCrumbs(
        Context<FileManagerState> cx,
        FileManagerState state,
        FileTab tab
    )
    {
        foreach (var (_, path) in PathSegments(tab.Path))
        {
            if (!state.CrumbLoaded.Add(path))
            {
                continue;
            }
            cx.Spawn(
                _ => Task.FromResult(FileSystemService.ListSubdirectories(path)),
                (s, children, context) =>
                {
                    s.CrumbChildren[path] = [.. children];
                    context.Notify();
                }
            );
        }
    }

    /// <summary>Splits a path into navigable segments: `C:`, `Users`, `Name`, …</summary>
    private static List<(string Label, string Path)> PathSegments(string path)
    {
        var segments = new List<(string Label, string Path)>();
        if (string.IsNullOrWhiteSpace(path))
        {
            return segments;
        }

        var root = Path.GetPathRoot(path);
        if (string.IsNullOrEmpty(root))
        {
            segments.Add((path, path));
            return segments;
        }

        var trimmedRoot = root.TrimEnd(
            Path.DirectorySeparatorChar,
            Path.AltDirectorySeparatorChar
        );
        segments.Add((trimmedRoot.Length > 0 ? trimmedRoot : root, root));

        var rest = path.Length > root.Length ? path[root.Length..] : string.Empty;
        var current = root;
        foreach (
            var part in rest.Split(
                [Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar],
                StringSplitOptions.RemoveEmptyEntries
            )
        )
        {
            current = Path.Combine(current, part);
            segments.Add((part, current));
        }
        return segments;
    }
}
