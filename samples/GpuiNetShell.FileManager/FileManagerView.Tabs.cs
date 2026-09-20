using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>The title bar tab strip: one chip per open folder, plus a new-tab button.</summary>
internal sealed partial class FileManagerView
{
    /// <summary>How far a `<`/`>` click scrolls the tab strip.</summary>
    private const int TAB_SCROLL_STEP = 180;

    private Element BuildTabs(RenderContext ui, FileManagerState state)
    {
        var chips = new List<Element>(state.Tabs.Count);
        for (var index = 0; index < state.Tabs.Count; index++)
        {
            chips.Add(BuildTab(ui, state, index));
        }

        // A horizontal scroll area clips the chips when they exceed the strip and
        // lets the `<`/`>` buttons reach the hidden ones.
        var strip = ui.Scroll("fm-tabs").Axis(ScrollAxis.Horizontal);
        if (_tabScrollNudge != 0)
        {
            strip.ScrollBy(_tabScrollNudge);
            _tabScrollNudge = 0;
        }
        strip.Add(chips.ToArray());
        return strip.FlexShrink(1).MinW(0);
    }

    /// <summary>Left/right buttons that scroll the tab strip.</summary>
    private Element BuildTabNav(RenderContext ui, FileManagerState state)
    {
        if (state.Tabs.Count <= 1)
        {
            return ui.Div();
        }
        return ui.HStack(
                NavButton(ui, "fm-tabs-left", "‹", () => NudgeTabs(-TAB_SCROLL_STEP)),
                NavButton(ui, "fm-tabs-right", "›", () => NudgeTabs(TAB_SCROLL_STEP))
            )
            .Gap(2)
            .ItemsCenter()
            .FlexShrink(0);
    }

    private static Element NavButton(RenderContext ui, string id, string glyph, Action onClick)
    {
        var button = ui.Div(ui.Label(glyph).TextSize(14))
            .Size(24)
            .Rounded(6)
            .Flex()
            .ItemsCenter()
            .JustifyCenter()
            .FlexShrink(0)
            .Occlude();
        button.OnClick(id, onClick);
        return button;
    }

    /// <summary>Requests a one-frame horizontal nudge of the tab strip.</summary>
    private void NudgeTabs(int delta)
    {
        _tabScrollNudge += delta;
        Invalidate();
    }

    /// <summary>
    /// The new-tab button. It sits outside the tab strip so a crowded strip can
    /// never clip it away.
    /// </summary>
    private Element BuildAddTab(RenderContext ui)
    {
        var add = ui.Div(ui.Label("+").TextSize(14))
            .H(28)
            .Px(12)
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

        // `OverflowHidden` lets a chip shrink below its text so many tabs never
        // push the new-tab button, the search box, or the window controls out.
        // The active tab uses a neutral highlight, not the theme accent: the
        // accent marks the selected folder in the list instead.
        var chip = ui.HStack(body, close)
            .Gap(4)
            .ItemsCenter()
            .MinW(0)
            .MaxW(168)
            .FlexShrink(1)
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
        Invalidate();
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
        Invalidate();
    }

    private void CloseTab(int index) =>
        Update(
            (s, c) =>
            {
                s.CloseTab(index);
                c.Notify();
            }
        );

    private void CloseOtherTabs(int index) =>
        Update(
            (s, c) =>
            {
                s.CloseOtherTabs(index);
                c.Notify();
            }
        );

    private void SelectTab(int index) =>
        Update(
            (s, c) =>
            {
                s.SelectTab(index);
                c.Notify();
            }
        );
}
