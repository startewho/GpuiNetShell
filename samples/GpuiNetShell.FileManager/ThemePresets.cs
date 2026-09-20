using GpuiNetShell;

namespace GpuiNetShell.FileManager;

/// <summary>One Windows 11 style accent color.</summary>
internal sealed record AccentPreset(string Key, string Label, string Hex);

/// <summary>Accent colors and the semantic theme overrides they produce.</summary>
internal static class ThemePresets
{
    public static IReadOnlyList<AccentPreset> Accents { get; } =
    [
        new("blue", "蓝", "#0078D4"),
        new("purple", "紫", "#8764B8"),
        new("teal", "青", "#00B7C3"),
        new("green", "绿", "#107C10"),
        new("orange", "橙", "#CA5010"),
        new("pink", "粉", "#E3008C"),
        new("red", "红", "#D13438"),
    ];

    public static AccentPreset Default => Accents[0];

    public static AccentPreset Find(string key) =>
        Accents.FirstOrDefault(accent => accent.Key == key) ?? Default;

    /// <summary>
    /// The theme color overrides that paint the primary, focus ring, selection,
    /// and links with <paramref name="accentHex"/>. The title bar is deliberately
    /// left neutral, so the accent marks only the selected folder in the file
    /// list rather than the whole tab strip.
    /// </summary>
    public static IReadOnlyDictionary<string, string> Palette(string accentHex) =>
        new Dictionary<string, string>(StringComparer.Ordinal)
        {
            [ThemeColors.Primary] = accentHex,
            [ThemeColors.PrimaryForeground] = "#ffffff",
            [ThemeColors.Ring] = accentHex,
            [ThemeColors.Selection] = accentHex,
            [ThemeColors.Link] = accentHex,
        };
}
