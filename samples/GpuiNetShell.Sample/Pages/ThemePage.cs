using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ThemePage : GalleryPage
{
    private static readonly Dictionary<string, string> CustomPalette = new()
    {
        [ThemeColors.Background] = "#101418",
        [ThemeColors.Foreground] = "#e6e6e6",
        [ThemeColors.Primary] = "#7c3aed",
        [ThemeColors.PrimaryForeground] = "#ffffff",
        [ThemeColors.Secondary] = "#1f2937",
        [ThemeColors.Muted] = "#111827",
        [ThemeColors.MutedForeground] = "#9ca3af",
        [ThemeColors.Border] = "#30363d",
        [ThemeColors.Input] = "#1f2937",
        [ThemeColors.Popover] = "#161b22",
        [ThemeColors.Sidebar] = "#161b22",
        [ThemeColors.SidebarForeground] = "#e6e6e6",
        [ThemeColors.SidebarBorder] = "#30363d",
        [ThemeColors.TitleBar] = "#161b22",
        [ThemeColors.TitleBarBorder] = "#30363d",
    };

    private ThemeMode _mode = ThemeMode.Light;
    private bool _custom;

    public override string Title => "Theme";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Theme",
            "Switch between light, dark, the system appearance, or a custom palette.",
            ui.HStack(
                    ui.Button("theme-light")
                        .Label("Light")
                        .OnClick(() => Apply(ThemeMode.Light, custom: false)),
                    ui.Button("theme-dark")
                        .Label("Dark")
                        .OnClick(() => Apply(ThemeMode.Dark, custom: false)),
                    ui.Button("theme-system")
                        .Label("System")
                        .OnClick(() => Apply(ThemeMode.System, custom: false)),
                    ui.Button("theme-custom")
                        .Label("Custom")
                        .Primary()
                        .OnClick(() => Apply(ThemeMode.Dark, custom: true))
                )
                .Gap(12)
                .ItemsCenter(),
            ui.Label($"Mode: {_mode}, custom palette: {_custom}")
        );

    private void Apply(ThemeMode mode, bool custom)
    {
        _mode = mode;
        _custom = custom;
        Application.SetTheme(mode, custom ? CustomPalette : null);
        Invalidate();
    }
}
