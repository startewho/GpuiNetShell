using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class CollapsiblePage : GalleryPage<CollapsiblePage.State>
{
    internal sealed class State
    {
        public bool Open { get; set; } = true;
    }

    public override string Title => "Collapsible";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Collapsible",
            "A trigger with a revealable content slot.",
            ui.Collapsible()
                .Open(state.Open)
                .Add(
                    ui.Button("collapse-toggle")
                        .Label(state.Open ? "Hide details" : "Show details")
                        .OnClick(() =>
                        {
                            Update((s, c) =>
                            {
                                s.Open = !s.Open;
                                c.Notify();
                            });
                        })
                )
                .Content(ui.Text("Revealed content in the named `content` slot."))
        );
}
