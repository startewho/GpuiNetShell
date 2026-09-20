using GpuiNetShell;

namespace GpuiNetShell.FileManager;

/// <summary>
/// The whole file manager state. It lives in an <c>Entity&lt;T&gt;</c> so a change
/// repaints only the file manager subtree; every mutation ends in
/// <c>Context.Notify()</c>.
/// </summary>
/// <remarks>
/// Per-folder state (path, history, listing, selection, search) lives on
/// <see cref="FileTab"/>; everything shared across tabs (sort, view, theme, the
/// tree, breadcrumb caches) lives here. The read-only convenience properties
/// below reflect the active tab so the render code stays unchanged.
/// </remarks>
internal sealed class FileManagerState
{
    public List<FileTab> Tabs { get; } = [];

    public int ActiveTabIndex { get; set; }

    public FileTab ActiveTab => Tabs[Math.Clamp(ActiveTabIndex, 0, Tabs.Count - 1)];

    // -- Shared settings ----------------------------------------------------

    public FileView View { get; set; } = FileView.Details;

    public string SortKey { get; set; } = "name";

    public bool SortAscending { get; set; } = true;

    public int IconColumns { get; set; } = 6;

    public ThemeMode Mode { get; set; } = ThemeMode.System;

    public string AccentKey { get; set; } = ThemePresets.Default.Key;

    public string AccentHex { get; set; } = ThemePresets.Default.Hex;

    /// <summary>Whether the settings popover is open.</summary>
    public bool SettingsOpen { get; set; }

    /// <summary>Whether the second toolbar row (view options) is shown.</summary>
    public bool ShowToolbar { get; set; }

    public List<TreeNode> TreeRoots { get; } = [];

    /// <summary>Subfolders of a breadcrumb, loaded lazily by the view.</summary>
    public Dictionary<string, List<TreeNode>> CrumbChildren { get; } =
        new(StringComparer.OrdinalIgnoreCase);

    /// <summary>Breadcrumb paths whose subfolders have been loaded.</summary>
    public HashSet<string> CrumbLoaded { get; } = new(StringComparer.OrdinalIgnoreCase);

    public int LastClickIndex { get; set; } = -1;

    public long LastClickTicks { get; set; }

    // -- Active tab accessors (rendering) -----------------------------------

    public string CurrentPath => Tabs.Count > 0 ? ActiveTab.Path : string.Empty;

    public List<FileEntry> Visible => ActiveTab.Visible;

    public string SearchText => ActiveTab.SearchText;

    public bool CanGoBack => Tabs.Count > 0 && ActiveTab.CanGoBack;

    public bool CanGoForward => Tabs.Count > 0 && ActiveTab.CanGoForward;

    public int SelectedIndex
    {
        get => ActiveTab.SelectedIndex;
        set => ActiveTab.SelectedIndex = value;
    }

    public bool Loading
    {
        get => ActiveTab.Loading;
        set => ActiveTab.Loading = value;
    }

    public string? Error
    {
        get => ActiveTab.Error;
        set => ActiveTab.Error = value;
    }

    // -- Tabs ---------------------------------------------------------------

    /// <summary>Adds a tab, makes it active, and returns it.</summary>
    public FileTab AddTab()
    {
        var tab = new FileTab();
        Tabs.Add(tab);
        ActiveTabIndex = Tabs.Count - 1;
        return tab;
    }

    /// <summary>Closes a tab unless it is the last one, keeping a sensible active tab.</summary>
    public void CloseTab(int index)
    {
        if (Tabs.Count <= 1 || index < 0 || index >= Tabs.Count)
        {
            return;
        }
        Tabs.RemoveAt(index);
        if (ActiveTabIndex >= Tabs.Count)
        {
            ActiveTabIndex = Tabs.Count - 1;
        }
        else if (index < ActiveTabIndex)
        {
            ActiveTabIndex--;
        }
    }

    /// <summary>Closes every tab except <paramref name="index"/>, which becomes active.</summary>
    public void CloseOtherTabs(int index)
    {
        if (Tabs.Count <= 1 || index < 0 || index >= Tabs.Count)
        {
            return;
        }
        var keep = Tabs[index];
        Tabs.Clear();
        Tabs.Add(keep);
        ActiveTabIndex = 0;
    }

    /// <summary>Makes <paramref name="index"/> active and recomputes its visible list.</summary>
    public void SelectTab(int index)
    {
        if (index < 0 || index >= Tabs.Count)
        {
            return;
        }
        ActiveTabIndex = index;
        Recompute(ActiveTab);
    }

    // -- Listing and search -------------------------------------------------

    /// <summary>Installs a listing for <paramref name="tab"/> and recomputes it.</summary>
    public void ApplyListing(FileTab tab, DirectoryListing listing, bool recordHistory)
    {
        tab.Loading = false;
        if (listing.Error is not null && listing.Entries.Count == 0)
        {
            tab.Error = listing.Error;
            return;
        }
        tab.SetListing(listing.Path, listing);
        if (recordHistory)
        {
            tab.PushHistory(listing.Path);
        }
        Recompute(tab);
    }

    /// <summary>Installs a search result set for <paramref name="tab"/>.</summary>
    public void ApplySearchResults(FileTab tab, int seq, IReadOnlyList<FileEntry> results)
    {
        tab.SetSearchResults(seq, results);
        Recompute(tab);
    }

    /// <summary>Rebuilds <paramref name="tab"/>'s visible list with the shared sort.</summary>
    public void Recompute(FileTab tab) => tab.Recompute(SortKey, SortAscending);

    /// <summary>Toggles the sort key, or flips direction when it is active, and recomputes.</summary>
    public void SortBy(string key)
    {
        if (SortKey == key)
        {
            SortAscending = !SortAscending;
        }
        else
        {
            SortKey = key;
            SortAscending = true;
        }
        Recompute(ActiveTab);
    }
}
