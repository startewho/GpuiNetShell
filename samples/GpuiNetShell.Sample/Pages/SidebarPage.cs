using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class SidebarPage : GalleryPage<SidebarPage.State>
{
    internal sealed class State
    {
        public string Selected { get; set; } = "Home";
    }

    public override string Title => "Sidebar";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
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
                                        .Selected(state.Selected == "Home")
                                        .OnClick(() => Choose("Home")),
                                    ui.SidebarMenuItem("Components")
                                        .Icon(SidebarIcon.Components)
                                        .Selected(state.Selected == "Components")
                                        .OnClick(() => Choose("Components")),
                                    ui.SidebarMenuItem("Settings")
                                        .Icon(SidebarIcon.Settings)
                                        .Selected(state.Selected == "Settings")
                                        .OnClick(() => Choose("Settings"))
                                )
                        )
                        .H(280),
                    ui.Div(ui.Label($"Selected: {state.Selected}"))
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
        Update((s, c) =>
        {
            s.Selected = name;
            c.Notify();
        });
    }
}
