using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class CollapsiblePage : GalleryPage
{
    private bool _open = true;

    public override string Title => "Collapsible";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Collapsible",
            "A trigger with a revealable content slot.",
            ui.Collapsible()
                .Open(_open)
                .Add(
                    ui.Button("collapse-toggle")
                        .Label(_open ? "Hide details" : "Show details")
                        .OnClick(() =>
                        {
                            _open = !_open;
                            Invalidate();
                        })
                )
                .Content(ui.Text("Revealed content in the named `content` slot."))
        );
}
