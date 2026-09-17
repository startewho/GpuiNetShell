namespace GpuiNetShell.FileManager;

/// <summary>
/// Maps a file to one of the icons bundled with the native host
/// (<c>gpui-kit-assets</c> Lucide set). Paths are relative to the asset root.
/// </summary>
internal static class FileIcons
{
    public const string Folder = "icons/folder.svg";
    public const string FolderOpen = "icons/folder-open.svg";
    public const string File = "icons/file.svg";
    public const string Drive = "icons/hard-drive.svg";
    public const string Tree = "icons/folder-tree.svg";
    public const string QuickAccess = "icons/star.svg";

    private static readonly HashSet<string> ImageExtensions = new(StringComparer.OrdinalIgnoreCase)
    {
        ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".webp", ".svg", ".ico", ".tif", ".tiff", ".heic",
    };

    private static readonly HashSet<string> VideoExtensions = new(StringComparer.OrdinalIgnoreCase)
    {
        ".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".mpg", ".mpeg",
    };

    private static readonly HashSet<string> AudioExtensions = new(StringComparer.OrdinalIgnoreCase)
    {
        ".mp3", ".wav", ".flac", ".aac", ".ogg", ".m4a", ".wma", ".opus",
    };

    private static readonly HashSet<string> ArchiveExtensions = new(StringComparer.OrdinalIgnoreCase)
    {
        ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz", ".iso", ".cab",
    };

    private static readonly HashSet<string> CodeExtensions = new(StringComparer.OrdinalIgnoreCase)
    {
        ".cs", ".rs", ".c", ".h", ".cpp", ".hpp", ".js", ".ts", ".tsx", ".jsx", ".py", ".java",
        ".go", ".rb", ".php", ".swift", ".kt", ".sh", ".ps1", ".lua", ".sql", ".html", ".css",
        ".scss", ".json", ".xml", ".yml", ".yaml", ".toml", ".ini", ".gradle",
    };

    private static readonly HashSet<string> TextExtensions = new(StringComparer.OrdinalIgnoreCase)
    {
        ".txt", ".md", ".log", ".csv", ".rtf", ".doc", ".docx", ".pdf", ".xls", ".xlsx", ".ppt",
        ".pptx",
    };

    /// <summary>The icon path for an entry, opening the folder icon when expanded.</summary>
    public static string For(FileEntry entry, bool expanded = false)
    {
        if (entry.IsDirectory)
        {
            return expanded ? FolderOpen : Folder;
        }
        var extension = System.IO.Path.GetExtension(entry.Name);
        if (ImageExtensions.Contains(extension))
        {
            return "icons/file-image.svg";
        }
        if (VideoExtensions.Contains(extension))
        {
            return "icons/film.svg";
        }
        if (AudioExtensions.Contains(extension))
        {
            return "icons/file-music.svg";
        }
        if (ArchiveExtensions.Contains(extension))
        {
            return "icons/file-archive.svg";
        }
        if (CodeExtensions.Contains(extension))
        {
            return "icons/file-code.svg";
        }
        if (TextExtensions.Contains(extension))
        {
            return "icons/file-text.svg";
        }
        return File;
    }
}
