using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class DropdownMenuPage : GalleryPage<DropdownMenuPage.State>
{
    internal sealed class State
    {
        public string Last { get; set; } = "(none)";
    }

    public override string Title => "DropdownMenu";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "DropdownMenu",
            "A button-triggered popup menu whose items run managed callbacks.",
            ui.DropdownMenu("actions", "Actions")
                .Item("Copy", () => Choose("Copy"))
                .Item("Paste", () => Choose("Paste"))
                .Item("Delete", () => Choose("Delete")),
            ui.Label($"Last action: {state.Last}")
        );

    private void Choose(string action) =>
        Update((s, c) =>
        {
            s.Last = action;
            c.Notify();
        });
}
