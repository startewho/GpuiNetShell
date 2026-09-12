using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class DropdownButtonPage : GalleryPage
{
    private string _last = "(none)";

    public override string Title => "DropdownButton";

    public override Element Render(ref RenderContext ui) =>
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
            ui.Label($"Last action: {_last}")
        );

    private void Choose(string action)
    {
        _last = action;
        Invalidate();
    }
}
