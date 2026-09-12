using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TagPage : GalleryPage
{
    public override string Title => "Tag";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Tag",
            "Compact semantic status tags that render ordinary children.",
            ui.HStack(
                    ui.Tag().Variant(TagVariant.Primary).Add(ui.Text("Primary")),
                    ui.Tag().Variant(TagVariant.Secondary).Add(ui.Text("Secondary")),
                    ui.Tag().Variant(TagVariant.Danger).Add(ui.Text("Danger")),
                    ui.Tag().Variant(TagVariant.Success).Add(ui.Text("Success")),
                    ui.Tag().Variant(TagVariant.Warning).Add(ui.Text("Warning")),
                    ui.Tag().Variant(TagVariant.Info).Add(ui.Text("Info"))
                )
                .Gap(8)
                .ItemsCenter(),
            ui.HStack(
                    ui.Tag().Outline().Add(ui.Text("Outline")),
                    ui.Tag().RoundedFull().Add(ui.Text("Rounded")),
                    ui.Tag().Size(ControlSize.Small).Add(ui.Text("Small"))
                )
                .Gap(8)
                .ItemsCenter()
        );
}
