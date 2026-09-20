namespace GpuiNetShell.FileManager;

/// <summary>
/// One open folder: its path, back/forward history, listing, selection, and
/// search state. Each tab is independent; a change repaints the active one.
/// </summary>
internal sealed class FileTab
{
    private readonly List<string> _history = [];
    private int _historyIndex = -1;

    public string Path { get; set; } = string.Empty;

    public List<FileEntry> Entries { get; } = [];

    /// <summary>Entries after search and sort; what the list renders.</summary>
    public List<FileEntry> Visible { get; private set; } = [];

    public List<FileEntry> SearchResults { get; } = [];

    public string SearchText { get; set; } = string.Empty;

    /// <summary>Bumped on every search so a stale result set is ignored.</summary>
    public int SearchSeq { get; set; }

    public int SelectedIndex { get; set; } = -1;

    public bool Loading { get; set; }

    public string? Error { get; set; }

    public bool CanGoBack => _historyIndex > 0;

    public bool CanGoForward => _historyIndex >= 0 && _historyIndex < _history.Count - 1;

    /// <summary>The folder name shown on the tab.</summary>
    public string Title
    {
        get
        {
            if (string.IsNullOrEmpty(Path))
            {
                return "新建标签";
            }
            var trimmed = Path.TrimEnd(
                System.IO.Path.DirectorySeparatorChar,
                System.IO.Path.AltDirectorySeparatorChar
            );
            var name = System.IO.Path.GetFileName(trimmed);
            return string.IsNullOrEmpty(name) ? Path : name;
        }
    }

    /// <summary>Installs a listing and clears the per-tab search state.</summary>
    public void SetListing(string path, DirectoryListing listing)
    {
        Path = path;
        Entries.Clear();
        Entries.AddRange(listing.Entries);
        SearchText = string.Empty;
        SearchResults.Clear();
        SelectedIndex = -1;
        Error = listing.Error;
        Visible = [];
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
    }

    /// <summary>Rebuilds <see cref="Visible"/> from the search filter and sort.</summary>
    public void Recompute(string sortKey, bool ascending)
    {
        var source = SearchText.Length > 0 ? SearchResults : Entries;
        var filtered = new List<FileEntry>(source);

        Comparison<FileEntry> comparison = sortKey switch
        {
            "modified" => (a, b) => a.Modified.CompareTo(b.Modified),
            "type" => (a, b) =>
                string.Compare(
                    Formatting.TypeName(a),
                    Formatting.TypeName(b),
                    StringComparison.OrdinalIgnoreCase
                ),
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
                return ascending ? result : -result;
            }
        );
        Visible = filtered;

        if (SelectedIndex >= Visible.Count)
        {
            SelectedIndex = -1;
        }
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
