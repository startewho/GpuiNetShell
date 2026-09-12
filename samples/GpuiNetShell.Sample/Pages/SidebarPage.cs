using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SidebarPage : GalleryPage
{
    private string _selected = "Home";

    public override string Title => "Sidebar";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Sidebar",
            "An application sidebar with a typed navigation menu, header, and footer.",
            ui.HStack(
                    ui.Sidebar("side")
                        .Collapsible(SidebarCollapsibleKind.Icon)
                        .Header(ui.Label("Workspace").TextSize(14).FontSemibold())
                        .Footer(ui.Label("v0.1.0").TextSize(12))
                        .Add(
                            ui.SidebarMenu()
                                .Add(
                                    ui.SidebarMenuItem("Home")
                                        .Icon(SidebarIcon.Home)
                                        .Selected(_selected == "Home")
                                        .OnClick(() => Choose("Home")),
                                    ui.SidebarMenuItem("Components")
                                        .Icon(SidebarIcon.Components)
                                        .Selected(_selected == "Components")
                                        .OnClick(() => Choose("Components")),
                                    ui.SidebarMenuItem("Settings")
                                        .Icon(SidebarIcon.Settings)
                                        .Selected(_selected == "Settings")
                                        .OnClick(() => Choose("Settings"))
                                )
                        )
                        .H(280),
                    ui.Div(ui.Label($"Selected: {_selected}"))
                        .P(16)
                        .Rounded(8)
                        .Border(1)
                        .BorderColor("#e5e7eb")
                )
                .Gap(16)
                .ItemsStart()
        );

    private void Choose(string name)
    {
        _selected = name;
        Invalidate();
    }
}
