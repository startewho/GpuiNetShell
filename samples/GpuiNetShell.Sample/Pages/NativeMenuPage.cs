using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class NativeMenuPage : GalleryPage
{
    private string _status = "(none)";

    public override string Title => "Native Menu";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Native Menu",
            "A real button that shows an operating-system popup menu.",
            ui.NativeMenuTrigger("native-menu", "Open native menu")
                .Add(
                    ui.NativeMenuItem("New")
                        .OnSelect(() =>
                        {
                            _status = "new";
                            Invalidate();
                        }),
                    ui.NativeMenuItem("Open")
                        .OnSelect(() =>
                        {
                            _status = "open";
                            Invalidate();
                        }),
                    ui.NativeMenuSeparator(),
                    ui.NativeMenuItem("Disabled").Disabled(),
                    ui.NativeMenuItem("Checked").Checked()
                ),
            ui.Label($"Last selection: {_status}")
        );
}
