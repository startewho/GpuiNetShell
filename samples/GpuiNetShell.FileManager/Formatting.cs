using System.Globalization;

namespace GpuiNetShell.FileManager;

/// <summary>Display formatting for directory entries.</summary>
internal static class Formatting
{
    /// <summary>The size column: empty for a folder, a readable size for a file.</summary>
    public static string Size(FileEntry entry)
    {
        if (entry.IsDirectory)
        {
            return string.Empty;
        }
        double value = entry.Size;
        string unit = "B";
        if (value >= 1024)
        {
            value /= 1024;
            unit = "KB";
        }
        if (value >= 1024)
        {
            value /= 1024;
            unit = "MB";
        }
        if (value >= 1024)
        {
            value /= 1024;
            unit = "GB";
        }
        return value.ToString(value < 10 && unit != "B" ? "0.#" : "0", CultureInfo.CurrentCulture)
            + " " + unit;
    }

    /// <summary>The modified column.</summary>
    public static string Date(DateTime value) =>
        value.ToString("yyyy/MM/dd HH:mm", CultureInfo.CurrentCulture);

    /// <summary>The type column: "文件夹", "PNG 文件", or "文件".</summary>
    public static string TypeName(FileEntry entry)
    {
        if (entry.IsDirectory)
        {
            return "文件夹";
        }
        var extension = System.IO.Path.GetExtension(entry.Name).TrimStart('.');
        return extension.Length == 0 ? "文件" : extension.ToUpperInvariant() + " 文件";
    }
}
