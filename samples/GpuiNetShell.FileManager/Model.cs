namespace GpuiNetShell.FileManager;

/// <summary>How the right pane presents the current directory.</summary>
internal enum FileView
{
    Details = 0,
    LargeIcons = 1,
}

/// <summary>One entry in the current directory, read once per listing.</summary>
internal sealed record FileEntry(
    string Name,
    string FullPath,
    bool IsDirectory,
    long Size,
    DateTime Modified
);

/// <summary>The outcome of listing a directory: its entries or a message.</summary>
internal sealed record DirectoryListing(
    string Path,
    IReadOnlyList<FileEntry> Entries,
    string? Error
);

/// <summary>The text read for a preview: content, an error, or a truncation flag.</summary>
internal sealed record TextPreview(string? Text, string? Error, bool Truncated);

/// <summary>
/// One row of the navigation tree. A node with an empty <see cref="Path"/> is a
/// synthetic group (Quick access, This PC); a real folder navigates when clicked.
/// Children are loaded lazily the first time the node expands.
/// </summary>
internal sealed class TreeNode
{
    public required string Id { get; init; }

    public required string Label { get; init; }

    public string Path { get; init; } = string.Empty;

    public string Icon { get; init; } = FileIcons.Folder;

    public bool Expanded { get; set; }

    public bool Loaded { get; set; }

    public List<TreeNode> Children { get; } = [];
}
