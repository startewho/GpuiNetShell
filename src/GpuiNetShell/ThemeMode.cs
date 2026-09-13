namespace GpuiNetShell;

/// <summary>Which appearance the application uses.</summary>
public enum ThemeMode : uint
{
    /// <summary>Always light.</summary>
    Light = 0,

    /// <summary>Always dark.</summary>
    Dark = 1,

    /// <summary>Follow the operating-system appearance.</summary>
    System = 2,
}

/// <summary>
/// Semantic theme-color names accepted by
/// <see cref="GpuiApplication.SetTheme"/>. Colors are <c>#rrggbb</c> (or
/// <c>#rrggbbaa</c>) literals applied on top of the chosen light/dark theme.
/// </summary>
public static class ThemeColors
{
    public const string Background = "background";
    public const string Foreground = "foreground";
    public const string Primary = "primary";
    public const string PrimaryForeground = "primary_foreground";
    public const string Secondary = "secondary";
    public const string SecondaryForeground = "secondary_foreground";
    public const string Muted = "muted";
    public const string MutedForeground = "muted_foreground";
    public const string Accent = "accent";
    public const string AccentForeground = "accent_foreground";
    public const string Border = "border";
    public const string Input = "input";
    public const string Ring = "ring";
    public const string Popover = "popover";
    public const string PopoverForeground = "popover_foreground";
    public const string Danger = "danger";
    public const string Success = "success";
    public const string Warning = "warning";
    public const string Info = "info";
    public const string Link = "link";
    public const string Selection = "selection";
    public const string Caret = "caret";
    public const string List = "list";
    public const string ListEven = "list_even";
    public const string ListHead = "list_head";
    public const string Sidebar = "sidebar";
    public const string SidebarForeground = "sidebar_foreground";
    public const string SidebarBorder = "sidebar_border";
    public const string Scrollbar = "scrollbar";
    public const string ScrollbarThumb = "scrollbar_thumb";
    public const string TitleBar = "title_bar";
    public const string TitleBarBorder = "title_bar_border";
}
