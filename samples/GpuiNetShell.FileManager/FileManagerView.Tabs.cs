using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>The title bar tab strip: one chip per open folder, plus a new-tab button.</summary>
internal sealed partial class FileManagerView
{
    /// <summary>Fixed chip width, so a whole number of tabs fills the strip.</summary>
    private const int TAB_CHIP_WIDTH = 130;

    /// <summary>Gap between tab chips.</summary>
    private const int TAB_GAP = 4;

    /// <summary>
    /// The tab strip. It shows only whole chips: the page size follows the
    /// measured strip width, so no half tab is ever shown.
    /// </summary>
    private Element BuildTabs(RenderContext ui, FileManagerState state)
    {
        var count = state.Tabs.Count;
        var page = TabPageSize(state);
        var start = Math.Clamp(_tabStart, 0, Math.Max(0, count - 1));
        // Keep the active tab on screen; the arrows move the selection, so the
        // page follows it.
        if (state.ActiveTabIndex < start)
        {
            start = state.ActiveTabIndex;
        }
        else if (state.ActiveTabIndex >= start + page)
        {
            start = state.ActiveTabIndex - page + 1;
        }
        start = Math.Clamp(start, 0, Math.Max(0, count - page));
        _tabStart = start;

        var end = Math.Min(count, start + page);
        var chips = new List<Element>(Math.Max(0, end - start));
        for (var index = start; index < end; index++)
        {
            chips.Add(BuildTab(ui, state, index));
        }

        return ui.Div(
                ui.HStack(chips.ToArray()).Gap(TAB_GAP).ItemsCenter().MinW(0),
                // Absolute so it does not take space; it measures the strip and
                // drives the page size.
                ui.Canvas("fm-tab-measure").Absolute().Full().Measure(MeasureTabStripToken)
            )
            .Relative()
            .Flex1()
            .MinW(0)
            .Flex()
            .ItemsCenter()
            .OverflowHidden();
    }

    /// <summary>How many whole tabs fit in the measured strip width.</summary>
    private int TabPageSize(FileManagerState state)
    {
        if (_tabStripWidth <= 1 || state.Tabs.Count == 0)
        {
            return 1;
        }
        var per = TAB_CHIP_WIDTH + TAB_GAP;
        return Math.Clamp((int)((_tabStripWidth + TAB_GAP) / per), 1, state.Tabs.Count);
    }

    /// <summary>Previous/next tab, one at a time, keeping it in view.</summary>
    private void SelectRelative(int delta)
    {
        var state = Entity.Read();
        var count = state.Tabs.Count;
        if (count == 0)
        {
            return;
        }
        var target = Math.Clamp(state.ActiveTabIndex + delta, 0, count - 1);
        if (target != state.ActiveTabIndex)
        {
            SelectTab(target);
        }
    }

    /// <summary>Scrolls the page so <paramref name="index"/> is fully visible.</summary>
    private void EnsureTabVisible(int index)
    {
        var page = TabPageSize(Entity.Read());
        if (index < _tabStart)
        {
            _tabStart = index;
        }
        else if (index >= _tabStart + page)
        {
            _tabStart = index - page + 1;
        }
        if (_tabStart < 0)
        {
            _tabStart = 0;
        }
        Invalidate();
    }

    /// <summary>A `‹`/`›` tab button; dimmed with no action at an end.</summary>
    private Element TabNavButton(RenderContext ui, string id, string glyph, bool enabled, Action onClick)
    {
        var button = ui.Div(ui.Label(glyph).TextSize(15))
            .Size(24)
            .Rounded(6)
            .Flex()
            .ItemsCenter()
            .JustifyCenter()
            .FlexShrink(0)
            .Occlude();
        if (enabled)
        {
            button.OnClick(id, onClick);
        }
        else
        {
            button.Opacity(0.3);
        }
        return button;
    }

    /// <summary>
    /// The new-tab button. It sits outside the tab strip so a crowded strip can
    /// never clip it away.
    /// </summary>
    private Element BuildAddTab(RenderContext ui)
    {
        var add = ui.Div(ui.Label("+").TextSize(14))
            .H(28)
            .Px(10)
            .Rounded(6)
            .Flex()
            .ItemsCenter()
            .JustifyCenter()
            .FlexShrink(0)
            .Occlude();
        // Mouse-down rather than click: the title bar is inside GPUI's drag
        // region and the tab strip rebuilds often, so the press/release click
        // state is not reliable here.
        add.OnMouseDown(pointer =>
        {
            if (pointer.Button == 0)
            {
                NewTab();
            }
        });
        return add;
    }

    private Element BuildTab(RenderContext ui, FileManagerState state, int index)
    {
        var tab = state.Tabs[index];
        var active = index == state.ActiveTabIndex;

        var title = ui.Label(tab.Title).TextSize(12).LineClamp(1);

        var body = ui.HStack(
                ui.Icon(FileIcons.Folder).Size(ControlSize.Small).Color("amber-500"),
                title
            )
            .Gap(6)
            .ItemsCenter()
            .Flex1()
            .MinW(0);

        var close = ui.Div(ui.Label("×").TextSize(12))
            .Size(18)
            .Rounded(4)
            .Flex()
            .ItemsCenter()
            .JustifyCenter()
            .FlexShrink(0);
        if (state.Tabs.Count > 1)
        {
            close.OnClick("fm-tab-close-" + index, () => CloseTab(index));
        }

        // A fixed width keeps the page size (and thus the number of whole tabs
        // shown) predictable. The active tab uses a neutral highlight, not the
        // theme accent: the accent marks the selected folder in the list instead.
        var chip = ui.HStack(body, close)
            .Gap(4)
            .ItemsCenter()
            .W(TAB_CHIP_WIDTH)
            .FlexShrink(0)
            .OverflowHidden()
            .H(28)
            .Px(8)
            .Rounded(6);
        if (active)
        {
            chip.Bg(NeutralSelection);
        }
        chip.OnClick("fm-tab-" + index, () => SelectTab(index)).Occlude();

        // Right-click menu: duplicate and the usual close variants. Stable keys
        // are unique per tab so an already-open menu's callbacks stay valid.
        return ui.ContextMenu("fm-tab-menu-" + index)
            .Target(chip)
            .Items(
                ui.ContextMenuItem("复制标签页")
                    .OnSelectStable("fm-tab-copy-" + index, () => DuplicateTab(index)),
                ui.ContextMenuItem("关闭标签页")
                    .OnSelectStable("fm-tab-close-menu-" + index, () => CloseTab(index)),
                ui.ContextMenuItem("关闭其他标签页")
                    .OnSelectStable("fm-tab-close-others-" + index, () => CloseOtherTabs(index))
            );
    }

    /// <summary>Opens a new tab at the current tab's folder (a duplicate).</summary>
    private void NewTab()
    {
        Entity.Update(
            (state, cx) =>
            {
                var path = state.CurrentPath;
                if (string.IsNullOrEmpty(path))
                {
                    path = FileSystemService.DefaultPath();
                }
                var tab = state.AddTab();
                tab.Loading = true;
                LoadTab(cx, tab, path, recordHistory: true);
                cx.Notify();
            }
        );
        EnsureTabVisible(Entity.Read().Tabs.Count - 1);
    }

    private void DuplicateTab(int index)
    {
        Entity.Update(
            (state, cx) =>
            {
                var path =
                    index >= 0 && index < state.Tabs.Count
                        ? state.Tabs[index].Path
                        : state.CurrentPath;
                if (string.IsNullOrEmpty(path))
                {
                    path = FileSystemService.DefaultPath();
                }
                var tab = state.AddTab();
                tab.Loading = true;
                LoadTab(cx, tab, path, recordHistory: true);
                cx.Notify();
            }
        );
        EnsureTabVisible(Entity.Read().Tabs.Count - 1);
    }

    private void CloseTab(int index)
    {
        Update(
            (s, c) =>
            {
                s.CloseTab(index);
                c.Notify();
            }
        );
        ClampTabStart();
    }

    private void CloseOtherTabs(int index)
    {
        Update(
            (s, c) =>
            {
                s.CloseOtherTabs(index);
                c.Notify();
            }
        );
        ClampTabStart();
    }

    private void SelectTab(int index)
    {
        Update(
            (s, c) =>
            {
                s.SelectTab(index);
                c.Notify();
            }
        );
        EnsureTabVisible(index);
    }

    /// <summary>Keeps the page start within range after the tab count changes.</summary>
    private void ClampTabStart()
    {
        var count = Entity.Read().Tabs.Count;
        var page = TabPageSize(Entity.Read());
        var maxStart = Math.Max(0, count - page);
        if (_tabStart > maxStart)
        {
            _tabStart = maxStart;
        }
        if (_tabStart < 0)
        {
            _tabStart = 0;
        }
        Invalidate();
    }
}
