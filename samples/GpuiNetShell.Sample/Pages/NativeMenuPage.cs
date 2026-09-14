using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class NativeMenuPage : GalleryPage<NativeMenuPage.State>
{
    internal sealed class State
    {
        public string Status { get; set; } = "(none)";
    }

    public override string Title => "Native Menu";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Native Menu",
            "A real button that shows an operating-system popup menu.",
            ui.NativeMenuTrigger("native-menu", "Open native menu")
                .Add(
                    ui.NativeMenuItem("New")
                        .OnSelect(() =>
                        {
                            Update((s, c) => { s.Status = "new"; c.Notify(); });
                        }),
                    ui.NativeMenuItem("Open")
                        .OnSelect(() =>
                        {
                            Update((s, c) => { s.Status = "open"; c.Notify(); });
                        }),
                    ui.NativeMenuSeparator(),
                    ui.NativeMenuItem("Disabled").Disabled(),
                    ui.NativeMenuItem("Checked").Checked()
                ),
            ui.Label($"Last selection: {state.Status}")
        );
}
