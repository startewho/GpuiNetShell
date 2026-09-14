using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TabBarPage : GalleryPage<TabBarPage.State>
{
    internal sealed class State
    {
        public int Index { get; set; }
    }

    public override string Title => "TabBar";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "TabBar",
            "A typed tab list accepting only Tab children.",
            ui.TabBar("tabs")
                .SelectedIndex(state.Index)
                .Variant(TabVariantKind.Pill)
                .OnChange(index =>
                {
                    Update((s, c) =>
                    {
                        s.Index = index;
                        c.Notify();
                    });
                })
                .Add(
                    ui.Tab().Label("Overview"),
                    ui.Tab().Label("Activity"),
                    ui.Tab().Label("Settings"),
                    ui.Tab().Label("Archived").Disabled()
                ),
            ui.Label($"Selected tab index: {state.Index}")
        );
}
