using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// Demonstrates the motion system: themed transitions, springs, a measured
/// reveal, staggered entries, and a looping keyframe track. Native drives the
/// frames; managed only uploads targets and semantic tokens on state change.
/// </summary>
[GpuiCallbacks]
internal sealed partial class AnimationPage : GalleryPage<AnimationPage.State>
{
    internal sealed class State
    {
        public bool Open { get; set; } = true;
        public bool Pulsing { get; set; }
    }

    internal static AnimationPage? Current { get; private set; }

    public AnimationPage() => Current = this;

    /// <summary>Dev affordance: flip the pulse animation.</summary>
    internal void SetPulsing(bool value) =>
        Update(
            (s, c) =>
            {
                s.Pulsing = value;
                c.Notify();
            }
        );

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
            "gpui-kit motion: themed transitions, springs, reveal, stagger, and keyframes. Only targets cross to native; frames stay native.",
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
                ),
            ui.Button("animation-pulse")
                .Label(state.Pulsing ? "Stop pulse" : "Start pulse")
                .OnClick(() =>
                    Update(
                        (s, c) =>
                        {
                            s.Pulsing = !s.Pulsing;
                            c.Notify();
                        }
                    )
                ),
            Section(ref ui, "Transition (themed)", "Duration and easing come from theme tokens."),
            ui.Motion("panel", MotionSpec.Themed(DurationToken.Normal, Easing.Enter))
                .Opacity(state.Open ? 1.0 : 0.0)
                .TranslateY(state.Open ? 0.0 : 12.0)
                .WFull()
                .Add(
                    ui.Div(ui.Text("Opacity and offset transition to the new target."))
                        .P(12)
                        .Rounded(8)
                        .Bg("#eef2ff")
                ),
            Section(ref ui, "Spring", "A retargetable, velocity-preserving spring."),
            ui.Motion("spring-thumb", MotionSpec.Spring(SpringToken.Move))
                .TranslateX(state.Open ? 120.0 : 0.0)
                .Add(
                    ui.Div()
                        .W(48)
                        .H(24)
                        .Rounded(12)
                        .Bg("#6366f1")
                ),
            Section(ref ui, "Reveal", "A measured, clipped height reveal driven by a spring."),
            ui.Reveal("reveal", state.Open, MotionSpec.Spring(SpringToken.Control))
                .Add(
                    ui.Div(ui.Text("Revealed by an animated measured height."))
                        .P(12)
                        .Rounded(8)
                        .Bg("#fef9c3")
                ),
            Section(ref ui, "Stagger", "Each entry delays a little after the previous."),
            ui.HStack(
                    ui.Motion(
                            "stagger-0",
                            MotionSpec.Themed(DurationToken.Fast, Easing.Exit)
                                .WithDelay(TimeSpan.FromMilliseconds(0))
                        )
                        .Opacity(state.Open ? 1.0 : 0.0)
                        .TranslateY(state.Open ? 0.0 : 8.0)
                        .Add(ui.Div(ui.Text("one")).P(8).Rounded(6).Bg("#dcfce7")),
                    ui.Motion(
                            "stagger-1",
                            MotionSpec.Themed(DurationToken.Fast, Easing.Exit)
                                .WithDelay(TimeSpan.FromMilliseconds(80))
                        )
                        .Opacity(state.Open ? 1.0 : 0.0)
                        .TranslateY(state.Open ? 0.0 : 8.0)
                        .Add(ui.Div(ui.Text("two")).P(8).Rounded(6).Bg("#dcfce7")),
                    ui.Motion(
                            "stagger-2",
                            MotionSpec.Themed(DurationToken.Fast, Easing.Exit)
                                .WithDelay(TimeSpan.FromMilliseconds(160))
                        )
                        .Opacity(state.Open ? 1.0 : 0.0)
                        .TranslateY(state.Open ? 0.0 : 8.0)
                        .Add(ui.Div(ui.Text("three")).P(8).Rounded(6).Bg("#dcfce7"))
                )
                .Gap(8),
            Section(ref ui, "Keyframes", "A looping keyframe track drives opacity while enabled."),
            ui.Motion(
                    "pulse",
                    MotionSpec
                        .Keyframes(
                            TimeSpan.FromMilliseconds(1200),
                            Easing.InOut,
                            new Keyframe(0.0, 0.3),
                            new Keyframe(0.5, 1.0, Easing.Out),
                            new Keyframe(1.0, 0.3)
                        )
                        .Loop()
                )
                .Active(state.Pulsing)
                .Add(ui.Label("pulsing").TextSize(20))
        );
}
