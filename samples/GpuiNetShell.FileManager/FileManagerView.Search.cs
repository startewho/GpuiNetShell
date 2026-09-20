using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>The title bar: tabs on the left, a small search box and settings on the right.</summary>
internal sealed partial class FileManagerView
{
    private Element BuildTitleBar(RenderContext ui, FileManagerState state) =>
        ui.HStack(
                BuildTabs(ui, state).Flex1().MinW(0),
                ui.Input("fm-titlebar-search")
                    .Placeholder("搜索所有文件")
                    .Value(state.SearchText)
                    .W(240)
                    .MaxW(320)
                    .My(4)
                    .OnChange(text => Update((s, c) => ApplySearch(s, c, text))),
                BuildSettingsButton(ui, state)
            )
            .Gap(8)
            .ItemsCenter()
            .WFull()
            .Px(8);

    /// <summary>
    /// Applies a new query to the active tab. An empty query returns to the
    /// folder listing; otherwise the folder is searched recursively and the
    /// results replace the visible entries once they arrive.
    /// </summary>
    private void ApplySearch(FileManagerState state, Context<FileManagerState> cx, string text)
    {
        var tab = state.ActiveTab;
        tab.SearchText = text;
        tab.SearchSeq++;
        var seq = tab.SearchSeq;

        if (string.IsNullOrWhiteSpace(text))
        {
            tab.SearchResults.Clear();
            state.Recompute(tab);
            cx.Notify();
            return;
        }

        // Show the (now empty) search result set immediately, then fill it in.
        state.Recompute(tab);
        cx.Notify();

        var root = tab.Path;
        cx.Spawn(
            _ => Task.FromResult(FileSystemService.Search(root, text, SearchLimit)),
            (s, results, context) =>
            {
                s.ApplySearchResults(tab, seq, results);
                context.Notify();
            }
        );
    }
}
