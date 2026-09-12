using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TooltipPage : GalleryPage
{
    public override string Title => "Tooltip";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Tooltip",
            "A button trigger with a managed text tooltip.",
            ui.HStack(
                    ui.Tooltip("save", "Save", "Saves the current document"),
                    ui.Tooltip("delete", "Delete", "Deletes the selected item")
                )
                .Gap(12)
                .ItemsCenter()
        );
}
