using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TabBarPage : GalleryPage
{
    private int _index;

    public override string Title => "TabBar";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "TabBar",
            "A typed tab list accepting only Tab children.",
            ui.TabBar("tabs")
                .SelectedIndex(_index)
                .Variant(TabVariantKind.Pill)
                .OnChange(index =>
                {
                    _index = index;
                    Invalidate();
                })
                .Add(
                    ui.Tab().Label("Overview"),
                    ui.Tab().Label("Activity"),
                    ui.Tab().Label("Settings"),
                    ui.Tab().Label("Archived").Disabled()
                ),
            ui.Label($"Selected tab index: {_index}")
        );
}
