using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// A parent entity and a child entity, showing the three parent/child channels:
/// <list type="bullet">
/// <item>value down — the parent passes its value into the child's render (props);</item>
/// <item>child observes parent — a parent change re-renders the child's own state;</item>
/// <item>event up — the parent subscribes to the child's typed events.</item>
/// </list>
/// Only the entity that notifies repaints; the child is a retained subtree.
/// </summary>
internal sealed class ParentChildPage : GalleryPage
{
    private sealed class ParentState
    {
        public int Value { get; set; } = 3;

        /// <summary>Recent child events, newest first.</summary>
        public List<string> Log { get; } = [];
    }

    private sealed class ChildState
    {
        public int Observed { get; set; }
        public int Clicks { get; set; }
    }

    private sealed record ChildClicked(int Clicks);

    private Entity<ParentState>? _parent;
    private Entity<ChildState>? _child;
    private bool _wired;

    public override string Title => "Parent / Child";

    public override Element Render(ref RenderContext ui)
    {
        var parent = _parent ??= Application.New<ParentState>(_ => new ParentState());

        // Child observes the parent: a parent notify mirrors the value into the
        // child's state and re-renders only the child subtree.
        var child =
            _child
            ??= Application.New<ChildState>(cx =>
            {
                cx.Observe(
                    parent,
                    (state, target, context) =>
                    {
                        state.Observed = target.Read().Value;
                        context.Notify();
                    }
                );
                return new ChildState();
            });

        // Parent subscribes to the child's typed events. This needs the parent's
        // context, so it is wired once on the first render.
        if (!_wired)
        {
            _wired = true;
            parent.Update(
                (_, cx) =>
                {
                    cx.Subscribe<ChildState, ChildClicked>(
                        child,
                        (state, _, ev, context) =>
                        {
                            state.Log.Insert(0, $"child clicked x{ev.Clicks}");
                            if (state.Log.Count > 4)
                            {
                                state.Log.RemoveAt(state.Log.Count - 1);
                            }
                            context.Notify();
                        }
                    );
                }
            );
        }

        return Page(
            ref ui,
            "Parent / Child",
            "A parent entity passes a value down to a child (props); the child observes the parent; the parent subscribes to the child's typed events.",
            ui
                .Child(
                    parent,
                    (state, parentUi, _) =>
                        parentUi
                            .VStack(
                                parentUi.Label("父组件 Parent").FontSemibold().TextSize(16),
                                parentUi.Label($"value = {state.Value}").TextSize(22),
                                parentUi
                                    .HStack(
                                        parentUi
                                            .Button("pc-dec")
                                            .Label("-1")
                                            .OnClick(() =>
                                                parent.Update(
                                                    (s, c) =>
                                                    {
                                                        s.Value--;
                                                        c.Notify();
                                                    }
                                                )
                                            ),
                                        parentUi
                                            .Button("pc-inc")
                                            .Label("+1")
                                            .Primary()
                                            .OnClick(() =>
                                                parent.Update(
                                                    (s, c) =>
                                                    {
                                                        s.Value++;
                                                        c.Notify();
                                                    }
                                                )
                                            )
                                    )
                                    .Gap(8)
                                    .ItemsCenter(),
                                parentUi
                                    .Label("父组件监听到的子组件事件：")
                                    .FontSemibold()
                                    .TextSize(12),
                                parentUi.VStack(
                                    state.Log.Count == 0
                                        ? new Element[]
                                        {
                                            parentUi.Label("(还没有事件)").TextSize(12),
                                        }
                                        : state
                                            .Log.Select(line =>
                                                parentUi.Label(line).TextSize(12)
                                            )
                                            .ToArray()
                                ),
                                Section(
                                    ref parentUi,
                                    "子组件 Child",
                                    "props 拿到父组件的值，同时 observe 父组件；按钮向父组件 emit 事件。"
                                ),
                                parentUi
                                    .Child(
                                        child,
                                        (childState, childUi, _) =>
                                            childUi
                                                .VStack(
                                                    childUi
                                                        .Label($"props from parent = {state.Value}")
                                                        .TextSize(16),
                                                    childUi.Label(
                                                        $"observed from parent = {childState.Observed}"
                                                    ),
                                                    childUi.Label(
                                                        $"child clicks = {childState.Clicks}"
                                                    ),
                                                    childUi
                                                        .Button("pc-child")
                                                        .Label("emit event to parent")
                                                        .OnClick(() =>
                                                            child.Update(
                                                                (s, c) =>
                                                                {
                                                                    s.Clicks++;
                                                                    c.Emit(
                                                                        new ChildClicked(s.Clicks)
                                                                    );
                                                                    c.Notify();
                                                                }
                                                            )
                                                        )
                                                )
                                                .Gap(6)
                                    )
                                    .P(12)
                            )
                            .Gap(10)
                )
                .P(16)
        );
    }
}
