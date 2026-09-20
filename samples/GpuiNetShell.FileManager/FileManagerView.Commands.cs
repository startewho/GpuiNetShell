using System.Diagnostics;
using GpuiNetShell.Entities;

namespace GpuiNetShell.FileManager;

/// <summary>Navigation, tab, and file commands.</summary>
internal sealed partial class FileManagerView
{
    private void SetView(FileView view) =>
        Update(
            (s, c) =>
            {
                s.View = view;
                c.Notify();
            }
        );

    private void SetMode(ThemeMode mode) =>
        Update(
            (s, c) =>
            {
                _application.SetTheme(mode, ThemePresets.Palette(s.AccentHex));
                s.Mode = mode;
                c.Notify();
            }
        );

    private void SetAccent(string hex, string? key) =>
        Update(
            (s, c) =>
            {
                _application.SetTheme(s.Mode, ThemePresets.Palette(hex));
                s.AccentHex = hex;
                if (key is not null)
                {
                    s.AccentKey = key;
                }
                c.Notify();
            }
        );

    private void GoBack(Context<FileManagerState> cx)
    {
        var tab = Entity.Read().ActiveTab;
        var target = tab.Back();
        cx.Notify();
        if (target is not null)
        {
            LoadTab(cx, tab, target, recordHistory: false);
        }
    }

    private void GoForward(Context<FileManagerState> cx)
    {
        var tab = Entity.Read().ActiveTab;
        var target = tab.Forward();
        cx.Notify();
        if (target is not null)
        {
            LoadTab(cx, tab, target, recordHistory: false);
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

    /// <summary>Loads <paramref name="path"/> into the active tab.</summary>
    private void NavigateTo(Context<FileManagerState> cx, string path, bool recordHistory)
    {
        if (string.IsNullOrWhiteSpace(path))
        {
            return;
        }
        var tab = Entity.Read().ActiveTab;
        tab.Loading = true;
        tab.Error = null;
        cx.Notify();
        LoadTab(cx, tab, path.Trim(), recordHistory);
    }

    /// <summary>Lists <paramref name="path"/> off the UI thread into <paramref name="tab"/>.</summary>
    private void LoadTab(
        Context<FileManagerState> cx,
        FileTab tab,
        string path,
        bool recordHistory
    )
    {
        cx.Spawn(
            _ => Task.FromResult(FileSystemService.ListDirectory(path)),
            (state, listing, context) =>
            {
                state.ApplyListing(tab, listing, recordHistory);
                PreloadCrumbs(context, state, tab);
                context.Notify();
                // The tab title in the title bar reflects the new path.
                Invalidate();
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
