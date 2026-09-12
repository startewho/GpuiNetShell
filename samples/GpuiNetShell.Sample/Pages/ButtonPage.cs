using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ButtonPage : GalleryPage
{
    private int _count;

    public override string Title => "Button";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Button",
            "Variants, sizes, and the loading/disabled states.",
            ui.HStack(
                    ui.Button("primary").Label("Primary").Primary(),
                    ui.Button("secondary").Label("Secondary").Secondary(),
                    ui.Button("danger").Label("Danger").Danger(),
                    ui.Button("success").Label("Success").Success(),
                    ui.Button("ghost").Label("Ghost").Ghost(),
                    ui.Button("link").Label("Link").Link()
                )
                .Gap(8)
                .ItemsCenter(),
            ui.HStack(
                    ui.Button("small").Label("Small").Size(ButtonSize.Small),
                    ui.Button("medium").Label("Medium").Size(ButtonSize.Medium),
                    ui.Button("large").Label("Large").Size(ButtonSize.Large),
                    ui.Button("loading").Label("Loading").Loading(),
                    ui.Button("disabled").Label("Disabled").Disabled()
                )
                .Gap(8)
                .ItemsCenter(),
            ui.Button("counter")
                .Label($"Clicked {_count} times")
                .OnClick(() =>
                {
                    _count++;
                    Invalidate();
                })
        );
}
