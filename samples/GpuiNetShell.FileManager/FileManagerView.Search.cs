using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>The title bar: tabs on the left, a small search box and settings on the right.</summary>
internal sealed partial class FileManagerView
{
    private Element BuildTitleBar(RenderContext ui, FileManagerState state) =>
        ui.HStack(
                // `<` at the far left and `>` at the far right of the tab strip;
                // either moves the selection one tab (the strip paginates so
                // only whole tabs are shown).
                TabNavButton(
                    ui,
                    "fm-tabs-prev",
                    "‹",
                    state.ActiveTabIndex > 0,
                    () => SelectRelative(-1)
                ),
                BuildTabs(ui, state),
                TabNavButton(
                    ui,
                    "fm-tabs-next",
                    "›",
                    state.ActiveTabIndex < state.Tabs.Count - 1,
                    () => SelectRelative(1)
                ),
                BuildAddTab(ui),
                ui.Div(
                        ui.Input("fm-titlebar-search")
                            .Placeholder("搜索所有文件")
                            .Value(state.SearchText)
                            .W(200)
                            .MaxW(320)
                            .Size(ControlSize.Small)
                            .OnChange(text => Update((s, c) => ApplySearch(s, c, text)))
                            .OnFocus(() => _searchFocused = true)
                            .OnBlur(() => _searchFocused = false)
                    )
                    .FlexShrink(0)
                    .Occlude(),
                ui.Div(BuildSettingsButton(ui, state)).FlexShrink(0).Occlude()
            )
            .Gap(6)
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
