using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class VirtualListPage : GalleryPage
{
    private const int VerticalCount = 100_000;
    private const int HorizontalCount = 1_000;

    // Per-item sizes are computed once and reused.
    private static readonly string VerticalSizes = BuildSizes(
        VerticalCount,
        index => 28 + (index % 6) * 16
    );
    private static readonly string HorizontalSizes = BuildSizes(
        HorizontalCount,
        index => 80 + (index % 5) * 28
    );

    private int _verticalSelected = -1;
    private int _horizontalSelected = -1;
    private int _verticalScrollIndex;
    private long _verticalScrollToken = 1;
    private int _horizontalScrollIndex;
    private long _horizontalScrollToken = 1;
    private string _status = "(no row action yet)";

    public override string Title => "Virtual List";

    public override Element Render(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Virtual List",
                    "Virtualized vertical/horizontal lists with varying sizes, jump commands, selection, and a row right-click menu."
                ),
                ui.HStack(
                        ui.Button("v-top").Label("Top").OnClick(() => JumpVertical(0)),
                        ui
                            .Button("v-mid")
                            .Label("Go to 50,000")
                            .OnClick(() => JumpVertical(50_000)),
                        ui
                            .Button("v-bottom")
                            .Label("Bottom")
                            .OnClick(() => JumpVertical(VerticalCount - 1)),
                        ui.Label($"selected row: {_verticalSelected}")
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.VirtualList("virtual-vertical", VerticalCount)
                    .ItemSizes(() => VerticalSizes)
                    .Vertical()
                    .WFull()
                    .Flex1()
                    .MinH(0)
                    .ScrollTo(_verticalScrollIndex, _verticalScrollToken)
                    .OnSelect(index =>
                    {
                        _verticalSelected = index;
                        Invalidate();
                    })
                    .RowMenu(
                        ui.ContextMenuItem("Select this row")
                            .OnSelect(row =>
                            {
                                _verticalSelected = row;
                                Invalidate();
                            }),
                        ui.ContextMenuSeparator(),
                        ui.ContextMenuItem("Delete row")
                            .OnSelect(row =>
                            {
                                _status = $"delete row {row}";
                                Invalidate();
                            })
                    )
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
                                .ItemsStart()
                                .Bg(
                                    index == _verticalSelected
                                        ? "blue-100"
                                        : index % 2 == 0
                                            ? "gray-100"
                                            : "gray-200"
                                )
                                .BorderB(1)
                                .BorderColor("gray-300")
                                .Rounded(4)
                    ),
                ui.HStack(
                        ui.Button("h-first").Label("First").OnClick(() => JumpHorizontal(0)),
                        ui
                            .Button("h-last")
                            .Label("Last")
                            .OnClick(() => JumpHorizontal(HorizontalCount - 1)),
                        ui.Label($"selected column: {_horizontalSelected}")
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.VirtualList("virtual-horizontal", HorizontalCount)
                    .ItemSizes(() => HorizontalSizes)
                    .Horizontal()
                    .Flex1()
                    .MinH(0)
                    .ScrollTo(_horizontalScrollIndex, _horizontalScrollToken)
                    .OnSelect(index =>
                    {
                        _horizontalSelected = index;
                        Invalidate();
                    })
                    .RenderItem(
                        (ctx, index) =>
                            ctx
                                .Div(
                                    ctx
                                        .VStack(
                                            ctx.Label($"Item {index + 1}").TextColor("white"),
                                            ctx
                                                .Label($"width {80 + (index % 5) * 28}px")
                                                .TextSize(12)
                                                .TextColor("white")
                                        )
                                        .Gap(2)
                                        .ItemsCenter()
                                )
                                .W(80 + (index % 5) * 28)
                                .HFull()
                                .P(6)
                                .ItemsStart()
                                .Bg(
                                    index == _horizontalSelected
                                        ? "amber-500"
                                        : index % 2 == 0
                                            ? "blue-500"
                                            : "violet-500"
                                )
                                .Rounded(8)
                                .ItemsEnd()
                    ),
                ui.Label($"Last row action: {_status}")
            )
            .Gap(12)
            .Flex1()
            .MinH(0);


    private void JumpVertical(int index)
    {
        _verticalScrollIndex = index;
        _verticalScrollToken++;
        Invalidate();
    }

    private void JumpHorizontal(int index)
    {
        _horizontalScrollIndex = index;
        _horizontalScrollToken++;
        Invalidate();
    }

    private static string BuildSizes(int count, Func<int, int> size)
    {
        var builder = new System.Text.StringBuilder(count * 3);
        for (var i = 0; i < count; i++)
        {
            if (i > 0)
            {
                builder.Append('\n');
            }
            builder.Append(size(i).ToString(System.Globalization.CultureInfo.InvariantCulture));
        }
        return builder.ToString();
    }
}
