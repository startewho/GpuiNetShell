using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// Demonstrates the motion system: a <c>Motion</c> container that transitions
/// opacity and offset to new targets, and a <c>Presence</c> container that stays
/// mounted while it animates out. Native drives the frames; managed only uploads
/// the targets on state change.
/// </summary>
[GpuiCallbacks]
internal sealed partial class AnimationPage : GalleryPage<AnimationPage.State>
{
    internal sealed class State
    {
        public bool Open { get; set; } = true;
    }

    public override string Title => "Animation";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Animation",
            "gpui-kit motion: opacity/offset transitions and presence. Native interpolates; managed only uploads targets.",
            ui.HStack(
                    ui.Button("animation-toggle")
                        .Label(state.Open ? "Collapse" : "Expand")
                        .Primary()
                        .OnClick(() =>
                            Update(
                                (s, c) =>
                                {
                                    s.Open = !s.Open;
                                    c.Notify();
                                }
                            )
                        )
                )
                .Gap(8)
                .ItemsCenter(),
            ui.Motion("panel", MotionSpec.Transition(TimeSpan.FromMilliseconds(220), Easing.Out))
                .Opacity(state.Open ? 1.0 : 0.0)
                .TranslateY(state.Open ? 0.0 : 12.0)
                .WFull()
                .Add(
                    ui.Div(
                            ui.Label("Animated content").FontSemibold(),
                            ui
                                .Text("Opacity and offset transition to the new target.")
                                .TextColor("gray-600")
                        )
                        .Gap(4)
                        .P(12)
                        .Rounded(8)
                        .Bg("#eef2ff")
                ),
            Section(ref ui, "Presence", "The notice stays mounted while it animates out."),
            ui.Presence(
                    "notice",
                    state.Open,
                    MotionSpec.Transition(TimeSpan.FromMilliseconds(200), Easing.InOut)
                )
                .Fade(0, 1)
                .SlideY(8, 0)
                .Add(
                    ui.Div(ui.Text("Presence content")).P(12).Rounded(8).Bg("#ecfdf5")
                )
        );
}
