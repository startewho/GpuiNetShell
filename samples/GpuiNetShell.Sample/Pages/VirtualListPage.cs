using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class VirtualListPage : GalleryPage<VirtualListPage.State>
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

    internal sealed class State
    {
        public int VerticalSelected { get; set; } = -1;
        public int HorizontalSelected { get; set; } = -1;
        public int VerticalScrollIndex { get; set; }
        public long VerticalScrollToken { get; set; } = 1;
        public int HorizontalScrollIndex { get; set; }
        public long HorizontalScrollToken { get; set; } = 1;
        public string Status { get; set; } = "(no row action yet)";
    }

    public override string Title => "Virtual List";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Virtual List",
                    "Virtualized vertical/horizontal lists with varying sizes, jump commands, selection, and a row right-click menu."
                ),
                ui.HStack(
                        ui.Button("v-top")
                            .Label("Top")
                            .OnClick(() => JumpVertical(0)),
                        ui.Button("v-mid")
                            .Label("Go to 50,000")
                            .OnClick(() => JumpVertical(50_000)),
                        ui.Button("v-bottom")
                            .Label("Bottom")
                            .OnClick(() => JumpVertical(VerticalCount - 1)),
                        ui.Label($"selected row: {state.VerticalSelected}")
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.VirtualList("virtual-vertical", VerticalCount)
                    .ItemSizes(() => VerticalSizes)
                    .Vertical()
                    .WFull()
                    .Flex1()
                    .MinH(0)
                    .ScrollTo(state.VerticalScrollIndex, state.VerticalScrollToken)
                    .OnSelect(index =>
                        Update(
                            (s, c) =>
                            {
                                s.VerticalSelected = index;
                                c.Notify();
                            }
                        )
                    )
                    .RowMenu(
                        ui.ContextMenuItem("Select this row")
                            .OnSelect(row =>
                                Update(
                                    (s, c) =>
                                    {
                                        s.VerticalSelected = row;
                                        c.Notify();
                                    }
                                )
                            ),
                        ui.ContextMenuSeparator(),
                        ui.ContextMenuItem("Delete row")
                            .OnSelect(row =>
                                Update(
                                    (s, c) =>
                                    {
                                        s.Status = $"delete row {row}";
                                        c.Notify();
                                    }
                                )
                            )
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
                                    index == state.VerticalSelected
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
                        ui.Button("h-first")
                            .Label("First")
                            .OnClick(() => JumpHorizontal(0)),
                        ui.Button("h-last")
                            .Label("Last")
                            .OnClick(() => JumpHorizontal(HorizontalCount - 1)),
                        ui.Label($"selected column: {state.HorizontalSelected}")
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.VirtualList("virtual-horizontal", HorizontalCount)
                    .ItemSizes(() => HorizontalSizes)
                    .Horizontal()
                    .Flex1()
                    .MinH(0)
                    .ScrollTo(state.HorizontalScrollIndex, state.HorizontalScrollToken)
                    .OnSelect(index =>
                        Update(
                            (s, c) =>
                            {
                                s.HorizontalSelected = index;
                                c.Notify();
                            }
                        )
                    )
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
                                    index == state.HorizontalSelected
                                        ? "amber-500"
                                        : index % 2 == 0
                                            ? "blue-500"
                                            : "violet-500"
                                )
                                .Rounded(8)
                                .ItemsEnd()
                    ),
                ui.Label($"Last row action: {state.Status}")
            )
            .Gap(12)
            .Flex1()
            .MinH(0);

    private void JumpVertical(int index) =>
        Update(
            (s, c) =>
            {
                s.VerticalScrollIndex = index;
                s.VerticalScrollToken++;
                c.Notify();
            }
        );

    private void JumpHorizontal(int index) =>
        Update(
            (s, c) =>
            {
                s.HorizontalScrollIndex = index;
                s.HorizontalScrollToken++;
                c.Notify();
            }
        );

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
