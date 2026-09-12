using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SelectPage : GalleryPage
{
    private string _selected = "";

    public override string Title => "Select";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Select",
            "A retained single-value select backed by a row snapshot.",
            ui.Select("theme", Options, value =>
            {
                _selected = value;
                Invalidate();
            })
                .Placeholder("Pick a theme")
                .RenderRow(
                    (ctx, fields) =>
                        ctx
                            .HStack(
                                ctx.Label(fields.ElementAtOrDefault(1) ?? ""),
                                ctx.Label(fields.ElementAtOrDefault(0) ?? "").TextSize(12)
                            )
                            .Gap(8)
                            .ItemsCenter()
                ),
            ui.Label($"Selected: {_selected}")
        );

    private static string Options() => "light\tLight\ndark\tDark\nsystem\tSystem";
}
