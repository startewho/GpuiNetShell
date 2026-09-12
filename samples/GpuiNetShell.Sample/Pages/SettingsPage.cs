using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SettingsPage : GalleryPage
{
    public override string Title => "Settings";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Settings",
            "A typed settings hierarchy: Settings → SettingPage → SettingGroup → SettingItem.",
            ui.Settings("prefs")
                .Size(ControlSize.Medium)
                .Add(
                    ui.SettingPage("General")
                        .Description("Application-wide preferences.")
                        .DefaultOpen()
                        .Add(
                            ui.SettingGroup()
                                .Title("Appearance")
                                .Description("How the application looks.")
                                .Add(
                                    ui.SettingItem("Theme")
                                        .Description("Choose the color theme.")
                                        .Keywords("theme", "color", "dark", "light")
                                        .Content(
                                            ui.Select("theme", () => "light\tLight\ndark\tDark", _ => { })
                                        ),
                                    ui.SettingItem("Compact mode")
                                        .Layout(SettingLayout.Horizontal)
                                        .Content(ui.Slider("density").Min(0).Max(1).Value(0))
                                )
                        ),
                    ui.SettingPage("Advanced")
                        .Description("Power-user options.")
                        .Add(
                            ui.SettingGroup()
                                .Title("Diagnostics")
                                .Add(
                                    ui.SettingItem("Telemetry")
                                        .Description("Send anonymous usage data.")
                                        .Content(ui.Slider("telemetry").Min(0).Max(1).Value(1))
                                )
                        )
                )
                .Full()
                .H(360)
        );
}
