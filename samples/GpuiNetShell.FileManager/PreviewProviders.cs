using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>Everything a preview provider needs to render one entry.</summary>
internal sealed record PreviewContext(
    FileEntry Entry,
    string? Text,
    bool Truncated,
    bool Loading,
    string? Error
);

/// <summary>
/// Renders a preview for some files. Providers are tried in order; a new kind
/// (video, animated gif, pdf, …) is added by appending to
/// <see cref="PreviewProviders.All"/> and does not touch the pane.
/// </summary>
internal interface IPreviewProvider
{
    /// <summary>Whether this provider can preview the entry.</summary>
    bool CanPreview(FileEntry entry);

    /// <summary>Whether the entry's text must be read before <see cref="Render"/>.</summary>
    bool NeedsText { get; }

    /// <summary>Builds the preview body.</summary>
    Element Render(RenderContext ui, PreviewContext context);
}

/// <summary>The providers, in priority order.</summary>
internal static class PreviewProviders
{
    private static readonly IPreviewProvider[] All =
    [
        new ImagePreviewProvider(),
        new TextPreviewProvider(),
    ];

    /// <summary>The first provider that can preview <paramref name="entry"/>, if any.</summary>
    public static IPreviewProvider? Find(FileEntry entry) =>
        Array.Find(All, provider => provider.CanPreview(entry));
}

/// <summary>Shows an image file via the native image element (filesystem paths load directly).</summary>
internal sealed class ImagePreviewProvider : IPreviewProvider
{
    private static readonly HashSet<string> Extensions = new(StringComparer.OrdinalIgnoreCase)
    {
        ".png",
        ".jpg",
        ".jpeg",
        ".gif",
        ".bmp",
        ".webp",
        ".svg",
        ".ico",
        ".tif",
        ".tiff",
    };

    public bool NeedsText => false;

    public bool CanPreview(FileEntry entry) =>
        !entry.IsDirectory && Extensions.Contains(Path.GetExtension(entry.Name));

    public Element Render(RenderContext ui, PreviewContext context) =>
        ui.Image(context.Entry.FullPath).Fit("contain").WFull().Flex1().MinH(0);
}

/// <summary>Shows a text file in a scrollable read-only view.</summary>
internal sealed class TextPreviewProvider : IPreviewProvider
{
    private static readonly HashSet<string> Extensions = new(StringComparer.OrdinalIgnoreCase)
    {
        ".txt",
        ".md",
        ".markdown",
        ".json",
        ".xml",
        ".yaml",
        ".yml",
        ".toml",
        ".ini",
        ".cfg",
        ".conf",
        ".log",
        ".csv",
        ".tsv",
        ".cs",
        ".rs",
        ".js",
        ".mjs",
        ".cjs",
        ".ts",
        ".tsx",
        ".jsx",
        ".py",
        ".java",
        ".kt",
        ".c",
        ".h",
        ".cc",
        ".cpp",
        ".hpp",
        ".go",
        ".rb",
        ".php",
        ".swift",
        ".sh",
        ".bash",
        ".ps1",
        ".bat",
        ".cmd",
        ".sql",
        ".html",
        ".htm",
        ".css",
        ".scss",
        ".less",
        ".vue",
        ".svelte",
        ".gradle",
        ".props",
        ".targets",
        ".csproj",
        ".sln",
        ".gitignore",
        ".editorconfig",
        ".env",
        ".lock",
    };

    private static readonly HashSet<string> Names = new(StringComparer.OrdinalIgnoreCase)
    {
        "makefile",
        "dockerfile",
        "license",
        "readme",
        "changelog",
        "cargo.lock",
    };

    public bool NeedsText => true;

    public bool CanPreview(FileEntry entry)
    {
        if (entry.IsDirectory)
        {
            return false;
        }
        return Extensions.Contains(Path.GetExtension(entry.Name)) || Names.Contains(entry.Name);
    }

    public Element Render(RenderContext ui, PreviewContext context) =>
        ui.Scroll("fm-preview-text")
            .Axis(ScrollAxis.Vertical)
            .Add(ui.Text(context.Text ?? string.Empty).TextSize(12).WFull())
            .Flex1()
            .MinH(0)
            .WFull();
}
