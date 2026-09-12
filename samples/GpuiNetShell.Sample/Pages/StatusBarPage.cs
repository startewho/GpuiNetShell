using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class StatusBarPage : GalleryPage
{
    public override string Title => "StatusBar";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "StatusBar",
            "A three-region status bar with pinned edge content.",
            ui.StatusBar()
                .LeftContent(ui.Label("Ready"))
                .RightContent(ui.Label("v0.1.0"))
                .Add(ui.Text("Three-region status bar"))
        );
}
