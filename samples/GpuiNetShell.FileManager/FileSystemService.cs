namespace GpuiNetShell.FileManager;

/// <summary>
/// Reads the filesystem for the UI. Every method is synchronous and safe to run
/// on a thread-pool thread; the view marshals results back to the UI thread.
/// </summary>
internal static class FileSystemService
{
    /// <summary>A folder with more than this many entries is truncated.</summary>
    public const int MaxEntries = 4000;

    /// <summary>Lists one directory; on failure returns the message, not an exception.</summary>
    public static DirectoryListing ListDirectory(string path)
    {
        try
        {
            var info = new DirectoryInfo(path);
            if (!info.Exists)
            {
                return new DirectoryListing(path, [], $"找不到路径：{path}");
            }
            var entries = new List<FileEntry>();
            var truncated = false;
            try
            {
                foreach (var item in info.EnumerateFileSystemInfos())
                {
                    if (entries.Count >= MaxEntries)
                    {
                        truncated = true;
                        break;
                    }
                    var entry = ToEntry(item);
                    if (entry is not null)
                    {
                        entries.Add(entry);
                    }
                }
            }
            catch (Exception exception) when (IsExpected(exception))
            {
                if (entries.Count == 0)
                {
                    return new DirectoryListing(path, [], exception.Message);
                }
            }
            var error = truncated ? $"仅显示前 {MaxEntries} 个项目。" : null;
            return new DirectoryListing(info.FullName, entries, error);
        }
        catch (Exception exception) when (IsExpected(exception))
        {
            return new DirectoryListing(path, [], exception.Message);
        }
    }

    /// <summary>The immediate subdirectories of a folder, for tree expansion.</summary>
    public static IReadOnlyList<TreeNode> ListSubdirectories(string path)
    {
        var nodes = new List<TreeNode>();
        try
        {
            var info = new DirectoryInfo(path);
            if (!info.Exists)
            {
                return nodes;
            }
            foreach (var child in info.EnumerateDirectories())
            {
                nodes.Add(
                    new TreeNode
                    {
                        Id = child.FullName,
                        Label = child.Name,
                        Path = child.FullName,
                    }
                );
            }
        }
        catch (Exception exception) when (IsExpected(exception))
        {
            // An unreadable folder simply has no visible children.
        }
        return nodes;
    }

    /// <summary>
    /// Recursively searches <paramref name="root"/> for entries whose name
    /// contains <paramref name="query"/>, breadth-first, up to
    /// <paramref name="maxResults"/> matches. Unreadable folders are skipped.
    /// </summary>
    public static IReadOnlyList<FileEntry> Search(string root, string query, int maxResults)
    {
        var results = new List<FileEntry>();
        if (string.IsNullOrWhiteSpace(query) || string.IsNullOrWhiteSpace(root))
        {
            return results;
        }

        var queue = new Queue<string>();
        queue.Enqueue(root);
        while (queue.Count > 0 && results.Count < maxResults)
        {
            var directory = queue.Dequeue();
            DirectoryInfo info;
            try
            {
                info = new DirectoryInfo(directory);
                if (!info.Exists)
                {
                    continue;
                }
            }
            catch (Exception exception) when (IsExpected(exception))
            {
                continue;
            }

            try
            {
                foreach (var item in info.EnumerateFileSystemInfos())
                {
                    if (results.Count >= maxResults)
                    {
                        break;
                    }
                    var entry = ToEntry(item);
                    if (entry is null)
                    {
                        continue;
                    }
                    if (entry.Name.Contains(query, StringComparison.OrdinalIgnoreCase))
                    {
                        results.Add(entry);
                    }
                    if (entry.IsDirectory)
                    {
                        queue.Enqueue(entry.FullPath);
                    }
                }
            }
            catch (Exception exception) when (IsExpected(exception))
            {
                // Skip a folder we cannot enumerate and keep searching.
            }
        }
        return results;
    }

    /// <summary>Desktop, documents, downloads, and the other user folders that exist.</summary>
    public static IReadOnlyList<TreeNode> QuickAccess()
    {
        (Environment.SpecialFolder Folder, string Label)[] candidates =
        [
            (Environment.SpecialFolder.Desktop, "桌面"),
            (Environment.SpecialFolder.MyDocuments, "文档"),
            (Environment.SpecialFolder.UserProfile, "主目录"),
            (Environment.SpecialFolder.MyPictures, "图片"),
            (Environment.SpecialFolder.MyMusic, "音乐"),
            (Environment.SpecialFolder.MyVideos, "视频"),
        ];

        var nodes = new List<TreeNode>();
        foreach (var (folder, label) in candidates)
        {
            var path = Environment.GetFolderPath(folder);
            if (!string.IsNullOrEmpty(path) && Directory.Exists(path))
            {
                nodes.Add(new TreeNode { Id = path, Label = label, Path = path });
            }
        }

        var downloads = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            "Downloads"
        );
        if (Directory.Exists(downloads))
        {
            nodes.Add(new TreeNode { Id = downloads, Label = "下载", Path = downloads });
        }
        return nodes;
    }

    /// <summary>The ready drives on this machine.</summary>
    public static IReadOnlyList<TreeNode> Drives()
    {
        var nodes = new List<TreeNode>();
        foreach (var drive in DriveInfo.GetDrives())
        {
            string? label = null;
            try
            {
                if (drive.IsReady)
                {
                    label = string.IsNullOrEmpty(drive.VolumeLabel)
                        ? null
                        : drive.VolumeLabel;
                }
            }
            catch (Exception exception) when (IsExpected(exception))
            {
                label = null;
            }
            var name = label is null ? drive.Name : $"{label} ({drive.Name.TrimEnd('\\')})";
            nodes.Add(
                new TreeNode
                {
                    Id = drive.Name,
                    Label = name,
                    Path = drive.Name,
                    Icon = FileIcons.Drive,
                }
            );
        }
        return nodes;
    }

    /// <summary>The best directory to open on startup.</summary>
    public static string DefaultPath()
    {
        var profile = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        return string.IsNullOrEmpty(profile) ? Path.GetPathRoot(Environment.SystemDirectory) ?? "C:\\" : profile;
    }

    /// <summary>The parent directory, or the same path at a drive root.</summary>
    public static string? Parent(string path)
    {
        var info = new DirectoryInfo(path);
        return info.Parent?.FullName;
    }

    private static FileEntry? ToEntry(FileSystemInfo item)
    {
        try
        {
            var isDirectory = (item.Attributes & FileAttributes.Directory) != 0;
            var size = isDirectory ? 0 : (item as FileInfo)?.Length ?? 0;
            return new FileEntry(item.Name, item.FullName, isDirectory, size, item.LastWriteTime);
        }
        catch (Exception exception) when (IsExpected(exception))
        {
            return null;
        }
    }

    private static bool IsExpected(Exception exception) =>
        exception
            is UnauthorizedAccessException
                or IOException
                or ArgumentException
                or NotSupportedException
                or System.Security.SecurityException;
}
