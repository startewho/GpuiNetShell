using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class DropdownButtonPage : GalleryPage<DropdownButtonPage.State>
{
    internal sealed class State
    {
        public string Last { get; set; } = "(none)";
    }

    public override string Title => "DropdownButton";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "DropdownButton",
            "A split dropdown button with a labeled action half and menu items.",
            ui.HStack(
                    ui.DropdownButton("primary", "Run")
                        .Variant(DropdownVariant.Primary)
                        .MenuItem("Run once", () => Choose("Run once"))
                        .MenuItem("Run all", () => Choose("Run all"))
                        .OnClick(() => Choose("Run")),
                    ui.DropdownButton("ghost", "More").Variant(DropdownVariant.Ghost).MenuItem("Duplicate", () => Choose("Duplicate"))
                )
                .Gap(12)
                .ItemsCenter(),
            ui.Label($"Last action: {state.Last}")
        );

    private void Choose(string action) =>
        Update((s, c) =>
        {
            s.Last = action;
            c.Notify();
        });
}
