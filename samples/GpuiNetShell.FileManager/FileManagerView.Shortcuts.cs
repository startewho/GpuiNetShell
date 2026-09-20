using GpuiNetShell.Entities;
using GpuiNetShell.Events;

namespace GpuiNetShell.FileManager;

/// <summary>Windows Explorer style keyboard shortcuts, delivered as window input.</summary>
internal sealed partial class FileManagerView
{
    private void HandleShortcut(InputEvent input)
    {
        if (input.Kind != InputEventKind.KeyDown)
        {
            return;
        }

        var ctrl = (input.Modifiers & InputModifiers.Control) != 0;
        var alt = (input.Modifiers & InputModifiers.Alt) != 0;
        var shift = (input.Modifiers & InputModifiers.Shift) != 0;
        var key = input.Key;

        if (ctrl && key == "t")
        {
            NewTab();
        }
        else if (ctrl && key == "w")
        {
            CloseTab(Entity.Read().ActiveTabIndex);
        }
        else if (ctrl && key == "tab")
        {
            CycleTab(shift ? -1 : 1);
        }
        else if (ctrl && key.Length == 1 && key[0] >= '1' && key[0] <= '9')
        {
            SelectTabByNumber(key[0] - '1');
        }
        else if (alt && key == "left")
        {
            StepActiveTab(tab => tab.Back());
        }
        else if (alt && key == "right")
        {
            StepActiveTab(tab => tab.Forward());
        }
        else if (alt && key == "up")
        {
            GoUpCommand();
        }
        else if (key == "f5")
        {
            ReloadCommand();
        }
    }

    /// <summary>Moves the active tab by <paramref name="delta"/>, wrapping around.</summary>
    private void CycleTab(int delta) =>
        Update(
            (s, c) =>
            {
                if (s.Tabs.Count == 0)
                {
                    return;
                }
                var count = s.Tabs.Count;
                s.SelectTab((((s.ActiveTabIndex + delta) % count) + count) % count);
                c.Notify();
            }
        );

    private void SelectTabByNumber(int index)
    {
        if (index < Entity.Read().Tabs.Count)
        {
            SelectTab(index);
        }
    }

    /// <summary>Runs a per-tab history step (back/forward) on the active tab.</summary>
    private void StepActiveTab(Func<FileTab, string?> step)
    {
        Entity.Update(
            (state, cx) =>
            {
                var tab = state.ActiveTab;
                var target = step(tab);
                if (target is not null)
                {
                    tab.Loading = true;
                    LoadTab(cx, tab, target, recordHistory: false);
                }
                cx.Notify();
            }
        );
        Invalidate();
    }

    private void GoUpCommand()
    {
        Entity.Update(
            (state, cx) =>
            {
                var tab = state.ActiveTab;
                var parent = FileSystemService.Parent(tab.Path);
                if (parent is not null)
                {
                    tab.Loading = true;
                    LoadTab(cx, tab, parent, recordHistory: true);
                }
                cx.Notify();
            }
        );
        Invalidate();
    }

    private void ReloadCommand()
    {
        Entity.Update(
            (state, cx) =>
            {
                var tab = state.ActiveTab;
                tab.Loading = true;
                LoadTab(cx, tab, tab.Path, recordHistory: false);
                cx.Notify();
            }
        );
        Invalidate();
    }
}
