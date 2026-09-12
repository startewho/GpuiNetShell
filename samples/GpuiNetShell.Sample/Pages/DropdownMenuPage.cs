using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class DropdownMenuPage : GalleryPage
{
    private string _last = "(none)";

    public override string Title => "DropdownMenu";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "DropdownMenu",
            "A button-triggered popup menu whose items run managed callbacks.",
            ui.DropdownMenu("actions", "Actions")
                .Item("Copy", () => Choose("Copy"))
                .Item("Paste", () => Choose("Paste"))
                .Item("Delete", () => Choose("Delete")),
            ui.Label($"Last action: {_last}")
        );

    private void Choose(string action)
    {
        _last = action;
        Invalidate();
    }
}
