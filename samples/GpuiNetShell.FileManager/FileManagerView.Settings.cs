using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>
/// The settings popover opened from the title bar: theme mode, accent color,
/// and whether the optional command row is shown.
/// </summary>
internal sealed partial class FileManagerView
{
    private Element BuildSettingsButton(RenderContext ui, FileManagerState state) =>
        ui.Popover("fm-settings", "⚙")
            .Open(state.SettingsOpen)
            .CardAnchor(PopoverAnchor.BottomRight)
            .OverlayClosable()
            .OnOpenChange(open =>
                Update(
                    (s, c) =>
                    {
                        s.SettingsOpen = open;
                        c.Notify();
                    }
                )
            )
            .Content(BuildSettingsPanel(ui, state));

    private Element BuildSettingsPanel(RenderContext ui, FileManagerState state)
    {
        var swatches = new List<Element>(ThemePresets.Accents.Count);
        foreach (var accent in ThemePresets.Accents)
        {
            var swatch = ui.Div().Size(24).Rounded(12).Bg(accent.Hex);
            if (accent.Key == state.AccentKey)
            {
                swatch.Border(2).BorderColor("gray-700");
            }
            var selected = accent;
            swatch.OnClick(
                "fm-accent-" + selected.Key,
                () => SetAccent(selected.Hex, selected.Key)
            );
            swatches.Add(swatch);
        }

        return ui.VStack(
                ui.Label("主题模式").FontSemibold().TextSize(12),
                ui.HStack(
                        ModeButton(ui, state, "浅色", ThemeMode.Light),
                        ModeButton(ui, state, "深色", ThemeMode.Dark),
                        ModeButton(ui, state, "跟随系统", ThemeMode.System)
                    )
                    .Gap(6),
                ui.Label("强调色").FontSemibold().TextSize(12),
                ui.HStack(swatches.ToArray()).Gap(8).ItemsCenter(),
                ui.ColorPicker("fm-accent")
                    .Label("自定义强调色")
                    .OnChange(hex => SetAccent(hex, key: null))
            )
            .Gap(10)
            .P(12)
            .W(320);
    }

    private Element ModeButton(
        RenderContext ui,
        FileManagerState state,
        string label,
        ThemeMode mode
    )
    {
        var id = "fm-mode-" + mode;
        return ui.Button(id).Label(label).Selected(state.Mode == mode).OnClick(() => SetMode(mode));
    }
}
