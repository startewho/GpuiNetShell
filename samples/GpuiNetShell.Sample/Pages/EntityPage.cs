using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// Demonstrates the managed entity model: a counter rendered as an independent
/// retained subtree, a mirror entity that <c>Observe</c>s it, an event log that
/// <c>Subscribe</c>s to typed events, and a list entity whose selection is
/// published to a detail entity. Only the changed entity's subtree repaints.
/// </summary>
internal sealed class EntityPage : GalleryPage
{
    private sealed class CounterState
    {
        public int Count { get; set; }
    }

    private sealed class MirrorState
    {
        public int Saved { get; set; }
        public int Updates { get; set; }
    }

    private sealed class LogState
    {
        public string Line { get; set; } = "(no events yet)";
    }

    private sealed class ListState
    {
        public int Selected { get; set; }
    }

    private sealed class DetailState
    {
        public string Text { get; set; } = "(nothing picked)";
    }

    private sealed class GlobalCounter
    {
        public int Count { get; init; }
    }

    private sealed record Incremented(int Delta);

    private sealed record ItemPicked(int Index, string Label);

    private static readonly (string Id, string Label)[] Items =
    [
        ("0", "Alpha"),
        ("1", "Beta"),
        ("2", "Gamma"),
    ];

    private Entity<CounterState>? _counter;
    private Entity<MirrorState>? _mirror;
    private Entity<LogState>? _log;
    private Entity<ListState>? _list;
    private Entity<DetailState>? _detail;

    public override string Title => "Entity";

    public override Element Render(ref RenderContext ui)
    {
        var counter = _counter ??= Application.New<CounterState>(_ => new CounterState());

        // Observe: the mirror tracks the counter's state whenever it notifies.
        var mirror =
            _mirror
            ??= Application.New<MirrorState>(cx =>
            {
                cx.Observe(
                    counter,
                    (state, target, context) =>
                    {
                        state.Saved = target.Read().Count;
                        state.Updates++;
                        context.Notify();
                    }
                );
                return new MirrorState();
            });

        // Subscribe: the log receives the counter's typed `Incremented` events.
        var log =
            _log
            ??= Application.New<LogState>(cx =>
            {
                cx.Subscribe<CounterState, Incremented>(
                    counter,
                    (state, _, ev, context) =>
                    {
                        state.Line = $"Incremented by {ev.Delta}";
                        context.Notify();
                    }
                );
                return new LogState();
            });

        var list = _list ??= Application.New<ListState>(_ => new ListState());

        // Subscribe: the detail follows the list's `ItemPicked` events.
        var detail =
            _detail
            ??= Application.New<DetailState>(cx =>
            {
                cx.Subscribe<ListState, ItemPicked>(
                    list,
                    (state, _, ev, context) =>
                    {
                        state.Text = ev.Label;
                        context.Notify();
                    }
                );
                return new DetailState();
            });

        return Page(
            ref ui,
            "Entity",
            "State lives in entities; Observe tracks notify, Subscribe receives typed events, and a notify repaints only that subtree.",
            ui
                .Child(
                    counter,
                    (state, childUi, cx) =>
                        childUi
                            .VStack(
                                childUi.Label($"count = {state.Count}").TextSize(24),
                                childUi
                                    .HStack(
                                        childUi
                                            .Button("entity-dec")
                                            .Label("-1")
                                            .OnClick(() =>
                                                counter.Update(
                                                    (s, c) =>
                                                    {
                                                        s.Count--;
                                                        c.Emit(new Incremented(-1));
                                                        c.Notify();
                                                    }
                                                )
                                            ),
                                        childUi
                                            .Button("entity-inc")
                                            .Label("+1")
                                            .Primary()
                                            .OnClick(() =>
                                                counter.Update(
                                                    (s, c) =>
                                                    {
                                                        s.Count++;
                                                        c.Emit(new Incremented(1));
                                                        c.Notify();
                                                    }
                                                )
                                            ),
                                        childUi
                                            .Button("entity-async")
                                            .Label("async +1")
                                            .OnClick(() =>
                                                cx.Spawn(
                                                    async _ =>
                                                    {
                                                        await Task.Delay(500)
                                                            .ConfigureAwait(false);
                                                        return 1;
                                                    },
                                                    (s, delta, c) =>
                                                    {
                                                        s.Count += delta;
                                                        c.Emit(new Incremented(delta));
                                                        c.Notify();
                                                    }
                                                )
                                            )
                                    )
                                    .Gap(8)
                                    .ItemsCenter()
                            )
                            .Gap(8)
                )
                .P(16),
            Section(ref ui, "Observe", "The mirror observes the counter and re-renders itself."),
            ui
                .Child(
                    mirror,
                    (state, childUi, _) =>
                        childUi.Label($"mirror: {state.Saved} (updates {state.Updates})")
                )
                .P(16),
            Section(ref ui, "Subscribe / Emit", "The log subscribes to the counter's typed events."),
            ui.Child(log, (state, childUi, _) => childUi.Label(state.Line)).P(16),
            Section(ref ui, "List selection", "Picking a row emits a typed event to the detail."),
            ui
                .HStack(
                    ui.Select(
                            "entity-select",
                            () => string.Join('\n', Items.Select(item => $"{item.Id}\t{item.Label}")),
                            selected =>
                            {
                                var index = int.Parse(
                                    selected,
                                    System.Globalization.CultureInfo.InvariantCulture
                                );
                                list.Update(
                                    (s, c) =>
                                    {
                                        s.Selected = index;
                                        c.Emit(new ItemPicked(index, Items[index].Label));
                                        c.Notify();
                                    }
                                );
                            }
                        )
                        .Placeholder("Pick one")
                        .W(200),
                    ui.Child(detail, (state, childUi, _) => childUi.Label($"picked: {state.Text}"))
                )
                .Gap(12)
                .ItemsCenter(),
            Section(
                ref ui,
                "Spawn / global",
                "\"async +1\" computes off-thread and applies on the UI thread; the global counter is process-wide."
            ),
            ui
                .HStack(
                    ui.Label($"global = {Application.Global<GlobalCounter>()?.Count ?? 0}"),
                    ui.Button("entity-global")
                        .Label("+ global")
                        .OnClick(() =>
                            Application.SetGlobal(
                                new GlobalCounter
                                {
                                    Count =
                                        (Application.Global<GlobalCounter>()?.Count ?? 0) + 1,
                                }
                            )
                        )
                )
                .Gap(8)
                .ItemsCenter()
        );
    }
}
