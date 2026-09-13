using System.Globalization;
using System.Text;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class VirtualListPage : GalleryPage
{
    // Per-item sizes are computed once and reused; the native list asks the
    // managed side for the whole size list.
    private static readonly string VerticalSizes = BuildSizes(
        100_000,
        index => 28 + (index % 6) * 16
    );
    private static readonly string HorizontalSizes = BuildSizes(1_000, index => 80 + (index % 5) * 28);

    public override string Title => "Virtual List";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Virtual List",
            "Vertical and horizontal virtualized lists with varying item sizes and a scrollbar.",
            ui.Label("Vertical (100,000 items, varying heights)").FontSemibold(),
            ui.VirtualList("virtual-vertical", 100_000)
                .ItemSizes(() => VerticalSizes)
                .Vertical()
                .H(0.5)
                .RenderItem(
                    (ctx, index) =>
                        ctx
                            .Div(
                                ctx
                                    .HStack(
                                        ctx.Label($"Row {index + 1}").FontMedium(),
                                        ctx
                                            .Label($"height {28 + (index % 6) * 16}px")
                                            .TextSize(12)
                                            .TextColor("gray-500")
                                    )
                                    .Gap(8)
                                    .ItemsCenter()
                            )
                            .WFull()
                            .H(28 + (index % 6) * 16)
                            .P(6)
                            .Bg(index % 2 == 0 ? "gray-100" : "gray-200")
                ),
            ui.Label("Horizontal (1,000 items, varying widths)").FontSemibold(),
            ui.VirtualList("virtual-horizontal", 1_000)
                .ItemSizes(() => HorizontalSizes)
                .Horizontal()
                .H(0.5)
                .RenderItem(
                    (ctx, index) =>
                        ctx
                            .Div(ctx.Label($"Item {index + 1}").TextColor("white"))
                            .W(80 + (index % 5) * 28)
                            .H(100)
                            .P(6)
                            .Bg(index % 2 == 0 ? "blue-500" : "violet-500")
                            .Rounded(8)
                            .ItemsEnd()
                )
        );

    private static string BuildSizes(int count, Func<int, int> size)
    {
        var builder = new StringBuilder(count * 3);
        for (var i = 0; i < count; i++)
        {
            if (i > 0)
            {
                builder.Append('\n');
            }
            builder.Append(size(i).ToString(CultureInfo.InvariantCulture));
        }
        return builder.ToString();
    }
}
