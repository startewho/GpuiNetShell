using System.Diagnostics;
using GpuiNetShell.Elements;
using GpuiNetShell.Events;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Tests;

/// <summary>
/// A per-frame managed allocation guard. <see cref="RenderContext"/> and
/// <see cref="RenderArena"/> are the steady-state hot path: every window frame
/// rebuilds the arena, so allocations here are paid at frame rate. The test
/// prints the number and asserts a ceiling so a regression fails the build.
/// </summary>
public sealed class AllocationBaselineTests
{
    private readonly ITestOutputHelper _output;

    public AllocationBaselineTests(ITestOutputHelper output) => _output = output;

    [Fact]
    public void PerFrameManagedAllocationsStayWithinBudget()
    {
        using var arena = new RenderArena();
        var events = new EventRegistry();
        var ui = new RenderContext(arena, events, static () => { });

        const int iterations = 200;

        // Warm up the JIT and grow the arena/event capacities so the measured
        // loop reflects steady state, not first-use growth.
        for (var i = 0; i < 20; i++)
        {
            Frame(arena, events, ui, (ulong)(0x1000 + i));
        }

        GC.Collect();
        var before = GC.GetAllocatedBytesForCurrentThread();
        var stopwatch = Stopwatch.StartNew();
        for (var i = 0; i < iterations; i++)
        {
            Frame(arena, events, ui, (ulong)(0x2000 + i));
        }
        stopwatch.Stop();
        var allocated = GC.GetAllocatedBytesForCurrentThread() - before;
        var perFrame = allocated / iterations;

        _output.WriteLine(
            $"{perFrame:N0} B/frame over {iterations} frames "
                + $"({allocated:N0} B total, {stopwatch.Elapsed.TotalMilliseconds:N1} ms)"
        );

        // A regression guard, not a target: the measured steady state is well
        // under this. Raise it deliberately, never to silence a leak.
        Assert.InRange(perFrame, 1, 40_000);
    }

    /// <summary>One representative frame: 100 rows of text + button, all styled.</summary>
    private static void Frame(RenderArena arena, EventRegistry events, RenderContext ui, ulong generation)
    {
        arena.Reset();
        events.BeginGeneration(generation);
        ui.BeginRender();

        var rows = new Element[100];
        for (var i = 0; i < rows.Length; i++)
        {
            rows[i] = ui
                .Div(
                    ui.Text($"row {i}"),
                    ui.Button($"b{i}").Label("Go").OnClick(static () => { })
                )
                .FlexRow()
                .ItemsCenter()
                .Gap(8)
                .P(4);
        }

        _ = ui.Div(rows).FlexColumn().P(16).Bg("#ffffff").Full();

        ui.EndRender();
        _ = arena.Publish();
        events.Retire(generation);
    }
}
