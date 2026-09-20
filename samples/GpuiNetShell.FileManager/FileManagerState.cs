using GpuiNetShell;

namespace GpuiNetShell.FileManager;

/// <summary>
/// The whole file manager state. It lives in an <c>Entity&lt;T&gt;</c> so a change
/// repaints only the file manager subtree; every mutation ends in
/// <c>Context.Notify()</c>.
/// </summary>
internal sealed class FileManagerState
{
    private readonly List<string> _history = [];
    private int _historyIndex = -1;

    public string CurrentPath { get; set; } = string.Empty;

    /// <summary>The address bar text; edits are committed with the Go button.</summary>
    public string AddressText { get; set; } = string.Empty;

    public string SearchText { get; set; } = string.Empty;

    /// <summary>Bumped on every search so a stale result set is ignored.</summary>
    public int SearchSeq { get; set; }

    /// <summary>Recursive search results for <see cref="SearchText"/>.</summary>
    public List<FileEntry> SearchResults { get; } = [];

    public List<FileEntry> Entries { get; } = [];

    /// <summary>Entries after the search filter and sort; what the list renders.</summary>
    public List<FileEntry> Visible { get; private set; } = [];

    /// <summary>The breadcrumb whose folder dropdown is open, if any.</summary>
    public string? OpenCrumb { get; set; }

    /// <summary>Subfolders of a breadcrumb, loaded lazily when its dropdown opens.</summary>
    public Dictionary<string, List<TreeNode>> CrumbChildren { get; } =
        new(StringComparer.OrdinalIgnoreCase);

    /// <summary>Breadcrumb paths whose subfolders have been loaded.</summary>
    public HashSet<string> CrumbLoaded { get; } = new(StringComparer.OrdinalIgnoreCase);

    public FileView View { get; set; } = FileView.Details;

    public string SortKey { get; set; } = "name";

    public bool SortAscending { get; set; } = true;

    public int SelectedIndex { get; set; } = -1;

    public bool Loading { get; set; }

    public string? Error { get; set; }

    public ThemeMode Mode { get; set; } = ThemeMode.System;

    public string AccentKey { get; set; } = ThemePresets.Default.Key;

    public string AccentHex { get; set; } = ThemePresets.Default.Hex;

    public bool ThemeOpen { get; set; }

    /// <summary>Icons per row in the large-icon view.</summary>
    public int IconColumns { get; set; } = 6;

    public List<TreeNode> TreeRoots { get; } = [];

    public int LastClickIndex { get; set; } = -1;

    public long LastClickTicks { get; set; }

    public bool CanGoBack => _historyIndex > 0;

    public bool CanGoForward => _historyIndex >= 0 && _historyIndex < _history.Count - 1;

    /// <summary>Installs a directory listing and recomputes the visible entries.</summary>
    public void SetListing(string path, DirectoryListing listing)
    {
        CurrentPath = path;
        AddressText = path;
        Entries.Clear();
        Entries.AddRange(listing.Entries);
        SearchText = string.Empty;
        SearchResults.Clear();
        SelectedIndex = -1;
        Error = listing.Error;
        Recompute();
    }

    /// <summary>Installs a recursive search result set, ignoring stale searches.</summary>
    public void SetSearchResults(int seq, IReadOnlyList<FileEntry> results)
    {
        if (seq != SearchSeq)
        {
            return;
        }
        SearchResults.Clear();
        SearchResults.AddRange(results);
        Recompute();
    }

    /// <summary>Rebuilds <see cref="Visible"/> from the filter and sort settings.</summary>
    public void Recompute()
    {
        // While searching, the visible list is the recursive search result set;
        // otherwise it is the current directory's listing.
        var source = SearchText.Length > 0 ? SearchResults : Entries;
        var filtered = new List<FileEntry>(source);

        Comparison<FileEntry> comparison = SortKey switch
        {
            "modified" => (a, b) => a.Modified.CompareTo(b.Modified),
            "type" => (a, b) =>
                string.Compare(Formatting.TypeName(a), Formatting.TypeName(b), StringComparison.OrdinalIgnoreCase),
            "size" => (a, b) => a.Size.CompareTo(b.Size),
            _ => (a, b) => string.Compare(a.Name, b.Name, StringComparison.OrdinalIgnoreCase),
        };
        filtered.Sort(
            (a, b) =>
            {
                if (a.IsDirectory != b.IsDirectory)
                {
                    return a.IsDirectory ? -1 : 1;
                }
                var result = comparison(a, b);
                return SortAscending ? result : -result;
            }
        );
        Visible = filtered;

        if (SelectedIndex >= Visible.Count)
        {
            SelectedIndex = -1;
        }
    }

    /// <summary>Toggles the sort key, or flips the direction when it is already active.</summary>
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
        Recompute();
    }

    public void PushHistory(string path)
    {
        if (
            _historyIndex >= 0
            && _historyIndex < _history.Count
            && string.Equals(_history[_historyIndex], path, StringComparison.OrdinalIgnoreCase)
        )
        {
            return;
        }
        if (_historyIndex < _history.Count - 1)
        {
            _history.RemoveRange(_historyIndex + 1, _history.Count - _historyIndex - 1);
        }
        _history.Add(path);
        _historyIndex = _history.Count - 1;
    }

    /// <summary>The previous path and the new history position, or null at the start.</summary>
    public string? Back()
    {
        if (!CanGoBack)
        {
            return null;
        }
        _historyIndex--;
        return _history[_historyIndex];
    }

    /// <summary>The next path and the new history position, or null at the end.</summary>
    public string? Forward()
    {
        if (!CanGoForward)
        {
            return null;
        }
        _historyIndex++;
        return _history[_historyIndex];
    }
}
