using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>The title bar tab strip: one chip per open folder, plus a new-tab button.</summary>
internal sealed partial class FileManagerView
{
    private Element BuildTabs(RenderContext ui, FileManagerState state)
    {
        var chips = new List<Element>(state.Tabs.Count + 1);
        for (var index = 0; index < state.Tabs.Count; index++)
        {
            chips.Add(BuildTab(ui, state, index));
        }

        var add = ui.Div(ui.Label("+").TextSize(14)).Size(28).Rounded(6).ItemsCenter().JustifyCenter();
        add.OnClick("fm-tab-new", NewTab);
        chips.Add(add);

        return ui.HStack(chips.ToArray()).Gap(4).ItemsCenter();
    }

    private Element BuildTab(RenderContext ui, FileManagerState state, int index)
    {
        var tab = state.Tabs[index];
        var active = index == state.ActiveTabIndex;

        var title = ui.Label(tab.Title).TextSize(12).LineClamp(1);
        if (active)
        {
            title.TextColor("#ffffff");
        }

        var body = ui.HStack(
                ui.Icon(FileIcons.Folder)
                    .Size(ControlSize.Small)
                    .Color(active ? "#ffffff" : "amber-500"),
                title
            )
            .Gap(6)
            .ItemsCenter()
            .Flex1()
            .MinW(0);

        var close = ui.Div(ui.Label("×").TextSize(12))
            .Size(18)
            .Rounded(4)
            .ItemsCenter()
            .JustifyCenter();
        if (state.Tabs.Count > 1)
        {
            close.OnClick("fm-tab-close-" + index, () => CloseTab(index));
        }

        var row = ui.HStack(body, close).Gap(4).ItemsCenter().W(168).H(28).Px(8).Rounded(6);
        if (active)
        {
            row.Bg(state.AccentHex);
        }
        row.OnClick("fm-tab-" + index, () => SelectTab(index));
        return row;
    }

    private void NewTab()
    {
        Entity.Update(
            (state, cx) =>
            {
                var tab = state.AddTab();
                tab.Loading = true;
                LoadTab(cx, tab, FileSystemService.DefaultPath(), recordHistory: true);
                cx.Notify();
            }
        );
    }

    private void CloseTab(int index) =>
        Update(
            (s, c) =>
            {
                s.CloseTab(index);
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
