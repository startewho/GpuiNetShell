//! `Motion`, `Presence`, and `Reveal`: gpui-kit motion animation for managed code.
//!
//! `Motion` interpolates opacity, translation, and size toward targets supplied
//! each render, using a timed transition, a spring, or a keyframe track.
//! `Presence` keeps a child mounted through its exit animation. `Reveal` is a
//! measured, clipped vertical reveal driven by animated progress. All three
//! sample `gpui_base::motion`, which requests frames while active, so managed
//! code is not involved per frame.
//!
//! Semantic durations/easings/springs come from the theme motion tokens, so
//! managed code can name `fast`/`normal`/`slow`, `enter`/`exit`/`move`, and
//! `control`/`move` instead of hard-coding values.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    div, px, AnyElement, App, ElementId, IntoElement as _, ParentElement as _, SharedString,
    SpringTarget, Styled as _, Window,
};
use gpui_base::motion::{
    animate_keyframes, spring as spring_animate, transition, Easing, Interpolate, IterationCount,
    Keyframe, Keyframes, MotionReveal, PlaybackDirection, Presence, SignedDuration, Spring, Timing,
    Transition,
};
use gpui_component::ActiveTheme as _;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct MotionPayload(String);

#[derive(Clone)]
struct PresencePayload(String);

#[derive(Clone)]
struct RevealPayload(String);

#[derive(Clone, Copy, Default, PartialEq)]
enum MotionKind {
    #[default]
    Transition,
    Spring,
    Keyframes,
}

#[derive(Clone)]
struct MotionPolicy {
    kind: MotionKind,
    /// When false, the animation is paused: targets are adopted immediately and
    /// no frames are requested.
    active: bool,
    duration_ms: f32,
    duration_token: Option<String>,
    easing: String,
    delay_ms: f32,
    response_ms: f32,
    damping: f32,
    travel: bool,
    spring_token: Option<String>,
    keyframes: Option<String>,
    iterations: f32,
    direction: String,
}

impl Default for MotionPolicy {
    fn default() -> Self {
        Self {
            kind: MotionKind::Transition,
            active: true,
            duration_ms: 200.0,
            duration_token: None,
            easing: "ease_out".to_string(),
            delay_ms: 0.0,
            response_ms: 180.0,
            damping: 1.0,
            travel: true,
            spring_token: None,
            keyframes: None,
            iterations: 1.0,
            direction: "normal".to_string(),
        }
    }
}

#[derive(Clone)]
enum MotionOp {
    Kind(String),
    Active(bool),
    Duration(f32),
    DurationToken(String),
    Easing(String),
    Delay(f32),
    SpringResponse(f32),
    SpringDamping(f32),
    SpringTravel(bool),
    SpringToken(String),
    Keyframes(String),
    Iterations(f32),
    Direction(String),
    // Targets read by `Motion`.
    Opacity(f32),
    TranslateX(f32),
    TranslateY(f32),
    Width(f32),
    Height(f32),
    // Presence / Reveal state.
    Present(bool),
    FadeFrom(f32),
    FadeTo(f32),
    SlideFrom(f32),
    SlideTo(f32),
}

/// The sampled targets of one `Motion` render.
struct MotionTargets {
    opacity: Option<f32>,
    translate_x: Option<f32>,
    translate_y: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
}

fn parse_kind(name: &str) -> MotionKind {
    match name {
        "spring" => MotionKind::Spring,
        "keyframes" => MotionKind::Keyframes,
        _ => MotionKind::Transition,
    }
}

fn parse_direction(name: &str) -> PlaybackDirection {
    match name {
        "reverse" => PlaybackDirection::Reverse,
        "alternate" => PlaybackDirection::Alternate,
        "alternate_reverse" => PlaybackDirection::AlternateReverse,
        _ => PlaybackDirection::Normal,
    }
}

fn parse_easing(name: &str, tokens: &gpui_component::theme::MotionTokens) -> Easing {
    match name {
        "linear" => Easing::Linear,
        "ease" => Easing::Ease,
        "ease_in" => Easing::EaseIn,
        "ease_in_out" => Easing::EaseInOut,
        "enter" => tokens.easing_enter.clone(),
        "exit" => tokens.easing_exit.clone(),
        "move" => tokens.easing_move.clone(),
        _ => Easing::EaseOut,
    }
}

fn resolve_duration(
    policy: &MotionPolicy,
    tokens: &gpui_component::theme::MotionTokens,
) -> Duration {
    match policy.duration_token.as_deref() {
        Some("instant") => tokens.duration_instant,
        Some("fast") => tokens.duration_fast,
        Some("normal") => tokens.duration_normal,
        Some("slow") => tokens.duration_slow,
        _ => Duration::from_millis(policy.duration_ms.max(0.0) as u64),
    }
}

fn resolve_spring(policy: &MotionPolicy, tokens: &gpui_component::theme::MotionTokens) -> Spring {
    match policy.spring_token.as_deref() {
        Some("control") => tokens.spring_control,
        Some("move") => tokens.spring_move,
        _ => Spring::new(Duration::from_millis(policy.response_ms.max(0.0) as u64))
            .with_damping(policy.damping)
            .with_travel(policy.travel),
    }
}

fn delay_duration(policy: &MotionPolicy) -> Duration {
    Duration::from_millis(policy.delay_ms.max(0.0) as u64)
}

fn parse_keyframes(dsl: &str) -> Result<Keyframes<f32>, String> {
    let mut frames = Vec::new();
    for part in dsl.split(';').filter(|part| !part.trim().is_empty()) {
        let mut fields = part.split(':');
        let offset = fields
            .next()
            .and_then(|value| value.trim().parse::<f32>().ok())
            .ok_or_else(|| format!("invalid keyframe `{part}`"))?;
        let value = fields
            .next()
            .and_then(|value| value.trim().parse::<f32>().ok())
            .ok_or_else(|| format!("invalid keyframe `{part}`"))?;
        let easing = fields
            .next()
            .map(|name| match name.trim() {
                "linear" => Easing::Linear,
                "ease" => Easing::Ease,
                "ease_in" => Easing::EaseIn,
                "ease_in_out" => Easing::EaseInOut,
                _ => Easing::EaseOut,
            })
            .unwrap_or(Easing::Linear);
        frames.push(Keyframe::new(offset, value).ease(easing));
    }
    Keyframes::try_new(frames).map_err(|error| format!("{error:?}"))
}

fn sample_transition<T: Interpolate + PartialEq + 'static>(
    key: &SharedString,
    channel: &'static str,
    target: T,
    policy: &Transition,
    window: &mut Window,
    cx: &mut App,
) -> T {
    transition((key.clone(), channel), target, policy.clone(), window, cx)
}

fn sample_spring<T: SpringTarget + 'static>(
    key: &SharedString,
    channel: &'static str,
    target: T,
    policy: &Spring,
    window: &mut Window,
    cx: &mut App,
) -> T::Output {
    spring_animate((key.clone(), channel), target, *policy, window, cx)
}

struct MotionMaterializer;

impl ComponentMaterializer for MotionMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<MotionPayload>()
            .ok_or_else(|| "Motion received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<MotionOp>().cloned())
            .collect::<Vec<_>>();
        let policy = read_policy(&operations);
        let targets = read_targets(&operations);
        let key = SharedString::from(id);

        let sampled = if !policy.active {
            // Paused: adopt the targets (and keyframes hold at full opacity)
            // without requesting any frames.
            let hold = policy.kind == MotionKind::Keyframes;
            MotionTargets {
                opacity: if hold { Some(1.0) } else { targets.opacity },
                translate_x: if hold { None } else { targets.translate_x },
                translate_y: if hold { None } else { targets.translate_y },
                width: if hold { None } else { targets.width },
                height: if hold { None } else { targets.height },
            }
        } else {
            request.with_window_app(|window, cx| -> Result<_, String> {
                let tokens = cx.theme().motion_tokens();
                let easing = parse_easing(&policy.easing, tokens);
                let duration = resolve_duration(&policy, tokens);
                Ok(match policy.kind {
                    MotionKind::Transition => {
                        let transition = Transition::new(duration)
                            .easing(easing.clone())
                            .delay(SignedDuration::from(delay_duration(&policy)));
                        MotionTargets {
                            opacity: targets.opacity.map(|t| {
                                sample_transition(&key, "opacity", t, &transition, window, cx)
                            }),
                            translate_x: targets
                                .translate_x
                                .map(|t| sample_transition(&key, "x", t, &transition, window, cx)),
                            translate_y: targets
                                .translate_y
                                .map(|t| sample_transition(&key, "y", t, &transition, window, cx)),
                            width: targets
                                .width
                                .map(|t| sample_transition(&key, "w", t, &transition, window, cx)),
                            height: targets
                                .height
                                .map(|t| sample_transition(&key, "h", t, &transition, window, cx)),
                        }
                    }
                    MotionKind::Spring => {
                        let spring = resolve_spring(&policy, tokens);
                        MotionTargets {
                            opacity: targets
                                .opacity
                                .map(|t| sample_spring(&key, "opacity", t, &spring, window, cx)),
                            translate_x: targets
                                .translate_x
                                .map(|t| sample_spring(&key, "x", t, &spring, window, cx)),
                            translate_y: targets
                                .translate_y
                                .map(|t| sample_spring(&key, "y", t, &spring, window, cx)),
                            width: targets
                                .width
                                .map(|t| sample_spring(&key, "w", t, &spring, window, cx)),
                            height: targets
                                .height
                                .map(|t| sample_spring(&key, "h", t, &spring, window, cx)),
                        }
                    }
                    MotionKind::Keyframes => {
                        let frames = parse_keyframes(policy.keyframes.as_deref().unwrap_or(""))?;
                        let timing = Timing::new(duration)
                            .delay(SignedDuration::from(delay_duration(&policy)))
                            .iterations(if policy.iterations <= 0.0 {
                                IterationCount::Infinite
                            } else {
                                IterationCount::Finite(policy.iterations.max(1.0) as u64)
                            })
                            .direction(parse_direction(&policy.direction))
                            .ease(easing);
                        let value =
                            animate_keyframes((key.clone(), "kf"), &frames, timing, window, cx)
                                .value;
                        MotionTargets {
                            opacity: Some(value),
                            translate_x: None,
                            translate_y: None,
                            width: None,
                            height: None,
                        }
                    }
                })
            })?
        };

        let children = request.take_children();
        let mut wrapper = div();
        if sampled.translate_x.is_some() || sampled.translate_y.is_some() {
            wrapper = wrapper.relative();
        }
        if let Some(value) = sampled.opacity {
            wrapper = wrapper.opacity(value);
        }
        if let Some(value) = sampled.translate_x {
            wrapper = wrapper.left(px(value));
        }
        if let Some(value) = sampled.translate_y {
            wrapper = wrapper.top(px(value));
        }
        if let Some(value) = sampled.width {
            wrapper = wrapper.w(px(value));
        }
        if let Some(value) = sampled.height {
            wrapper = wrapper.h(px(value));
        }
        wrapper = wrapper.children(children);
        Ok(wrapper.into_any_element())
    }
}

struct PresenceMaterializer;

impl ComponentMaterializer for PresenceMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<PresencePayload>()
            .ok_or_else(|| "Presence received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<MotionOp>().cloned())
            .collect::<Vec<_>>();
        let policy = read_policy(&operations);
        let presence = read_presence(&operations);
        let present = presence.present.unwrap_or(false);
        let (fade, slide) = (presence.fade, presence.slide);

        let key = ElementId::Name(SharedString::from(id.clone()));
        let sample = request.with_window_app(|window, cx| {
            let tokens = cx.theme().motion_tokens();
            let easing = parse_easing(&policy.easing, tokens);
            let duration = resolve_duration(&policy, tokens);
            let transition = Transition::new(duration)
                .easing(easing)
                .delay(SignedDuration::from(delay_duration(&policy)));
            Presence::new(key, present)
                .transition(transition)
                .sample(window, cx)
        });
        if !sample.should_render() {
            return Ok(div().into_any_element());
        }
        let progress = sample.progress;
        let opacity = fade.0 + (fade.1 - fade.0) * progress;
        let offset = slide.0 + (slide.1 - slide.0) * progress;
        let children = request.take_children();
        let mut wrapper = div().relative().opacity(opacity);
        if offset != 0.0 {
            wrapper = wrapper.top(px(offset));
        }
        wrapper = wrapper.children(children);
        Ok(wrapper.into_any_element())
    }
}

struct RevealMaterializer;

impl ComponentMaterializer for RevealMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<RevealPayload>()
            .ok_or_else(|| "Reveal received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<MotionOp>().cloned())
            .collect::<Vec<_>>();
        let policy = read_policy(&operations);
        let open = operations
            .iter()
            .any(|op| matches!(op, MotionOp::Present(true)));

        let key = SharedString::from(id);
        let progress = if !policy.active {
            if open {
                1.0_f32
            } else {
                0.0_f32
            }
        } else {
            request.with_window_app(|window, cx| {
                let tokens = cx.theme().motion_tokens();
                let easing = parse_easing(&policy.easing, tokens);
                let duration = resolve_duration(&policy, tokens);
                let target = if open { 1.0_f32 } else { 0.0_f32 };
                match policy.kind {
                    MotionKind::Spring => {
                        let spring = resolve_spring(&policy, tokens);
                        sample_spring(&key, "reveal", target, &spring, window, cx)
                    }
                    _ => {
                        let transition = Transition::new(duration)
                            .easing(easing)
                            .delay(SignedDuration::from(delay_duration(&policy)));
                        sample_transition(&key, "reveal", target, &transition, window, cx)
                    }
                }
            })
        };
        let children = request.take_children();
        let content = div().children(children).into_any_element();
        let reveal = MotionReveal::new(key, progress, content);
        Ok(reveal.into_any_element())
    }
}

fn read_policy(operations: &[MotionOp]) -> MotionPolicy {
    let mut policy = MotionPolicy::default();
    for op in operations {
        match op {
            MotionOp::Kind(name) => policy.kind = parse_kind(name),
            MotionOp::Active(value) => policy.active = *value,
            MotionOp::Duration(value) => policy.duration_ms = *value,
            MotionOp::DurationToken(name) => policy.duration_token = Some(name.clone()),
            MotionOp::Easing(name) => policy.easing = name.clone(),
            MotionOp::Delay(value) => policy.delay_ms = *value,
            MotionOp::SpringResponse(value) => policy.response_ms = *value,
            MotionOp::SpringDamping(value) => policy.damping = *value,
            MotionOp::SpringTravel(value) => policy.travel = *value,
            MotionOp::SpringToken(name) => policy.spring_token = Some(name.clone()),
            MotionOp::Keyframes(dsl) => policy.keyframes = Some(dsl.clone()),
            MotionOp::Iterations(value) => policy.iterations = *value,
            MotionOp::Direction(name) => policy.direction = name.clone(),
            _ => {}
        }
    }
    policy
}

fn read_targets(operations: &[MotionOp]) -> MotionTargets {
    let mut targets = MotionTargets {
        opacity: None,
        translate_x: None,
        translate_y: None,
        width: None,
        height: None,
    };
    for op in operations {
        match op {
            MotionOp::Opacity(value) => targets.opacity = Some(*value),
            MotionOp::TranslateX(value) => targets.translate_x = Some(*value),
            MotionOp::TranslateY(value) => targets.translate_y = Some(*value),
            MotionOp::Width(value) => targets.width = Some(*value),
            MotionOp::Height(value) => targets.height = Some(*value),
            _ => {}
        }
    }
    targets
}

struct PresenceFields {
    present: Option<bool>,
    fade: (f32, f32),
    slide: (f32, f32),
}

fn read_presence(operations: &[MotionOp]) -> PresenceFields {
    let mut fields = PresenceFields {
        present: None,
        fade: (0.0, 1.0),
        slide: (8.0, 0.0),
    };
    for op in operations {
        match op {
            MotionOp::Present(value) => fields.present = Some(*value),
            MotionOp::FadeFrom(value) => fields.fade.0 = *value,
            MotionOp::FadeTo(value) => fields.fade.1 = *value,
            MotionOp::SlideFrom(value) => fields.slide.0 = *value,
            MotionOp::SlideTo(value) => fields.slide.1 = *value,
            _ => {}
        }
    }
    fields
}

fn number_method(name: &'static str, make: fn(f32) -> MotionOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new("value", ArgumentSchema::Number)],
        move |arguments| match arguments {
            [ComponentArgument::Number(value)] if value.is_finite() => {
                Ok(ComponentPayload::new(make(*value as f32)))
            }
            _ => Err(format!("{name}(value) expects a number")),
        },
    )
    .with_documentation("Sets an animation value.")
}

fn boolean_method(name: &'static str, make: fn(bool) -> MotionOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new("value", ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(make(*value))),
            [ComponentArgument::Number(value)] => Ok(ComponentPayload::new(make(*value != 0.0))),
            _ => Err(format!("{name}(value) expects a boolean")),
        },
    )
    .with_documentation("Sets an animation flag.")
}

fn enum_method(
    name: &'static str,
    values: &'static [&'static str],
    make: fn(String) -> MotionOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(
            "value",
            ArgumentSchema::Enum(values),
        )],
        move |arguments| match arguments {
            [value] => value
                .as_str()
                .map(|text| ComponentPayload::new(make(text.to_string())))
                .ok_or_else(|| format!("{name}(value) expects a string")),
            _ => Err(format!("{name}(value) expects a string")),
        },
    )
    .with_documentation("Sets an animation option.")
}

fn string_method(name: &'static str, make: fn(String) -> MotionOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new("value", ArgumentSchema::String)],
        move |arguments| match arguments {
            [value] => value
                .as_str()
                .map(|text| ComponentPayload::new(make(text.to_string())))
                .ok_or_else(|| format!("{name}(value) expects a string")),
            _ => Err(format!("{name}(value) expects a string")),
        },
    )
    .with_documentation("Sets an animation value.")
}

fn motion_policy_methods() -> Vec<MethodDescriptor> {
    vec![
        enum_method(
            "kind",
            &["transition", "spring", "keyframes"],
            MotionOp::Kind,
        ),
        boolean_method("active", MotionOp::Active),
        number_method("duration_ms", MotionOp::Duration),
        enum_method(
            "duration_token",
            &["instant", "fast", "normal", "slow"],
            MotionOp::DurationToken,
        ),
        enum_method(
            "easing",
            &[
                "linear",
                "ease",
                "ease_in",
                "ease_out",
                "ease_in_out",
                "enter",
                "exit",
                "move",
            ],
            MotionOp::Easing,
        ),
        number_method("delay_ms", MotionOp::Delay),
        number_method("spring_response_ms", MotionOp::SpringResponse),
        number_method("spring_damping", MotionOp::SpringDamping),
        boolean_method("spring_travel", MotionOp::SpringTravel),
        enum_method("spring_token", &["control", "move"], MotionOp::SpringToken),
        string_method("keyframes", MotionOp::Keyframes),
        number_method("iterations", MotionOp::Iterations),
        enum_method(
            "direction",
            &["normal", "reverse", "alternate", "alternate_reverse"],
            MotionOp::Direction,
        ),
    ]
}

fn motion_target_methods() -> Vec<MethodDescriptor> {
    vec![
        number_method("opacity", MotionOp::Opacity),
        number_method("translate_x", MotionOp::TranslateX),
        number_method("translate_y", MotionOp::TranslateY),
        number_method("width", MotionOp::Width),
        number_method("height", MotionOp::Height),
    ]
}

fn id_constructor(
    export: &'static str,
    make: fn(String) -> ComponentPayload,
) -> ConstructorDescriptor {
    ConstructorDescriptor::new(
        export,
        vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(id)] if !id.trim().is_empty() => Ok(make(id.clone())),
            _ => Err(format!("{export} expects a non-empty id")),
        },
    )
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    let mut motion_methods = motion_policy_methods();
    motion_methods.extend(motion_target_methods());
    registry
        .register(
            ComponentDescriptor::new("Motion", Arc::new(MotionMaterializer))
                .with_constructors(vec![id_constructor("Motion", |id| {
                    ComponentPayload::new(MotionPayload(id))
                })])
                .with_methods(motion_methods)
                .with_documentation(
                    "A container that animates opacity/translation/size toward targets.",
                ),
        )
        .expect("the Motion descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Presence", Arc::new(PresenceMaterializer))
                .with_constructors(vec![id_constructor("Presence", |id| {
                    ComponentPayload::new(PresencePayload(id))
                })])
                .with_methods(vec![
                    boolean_method("present", MotionOp::Present),
                    number_method("duration_ms", MotionOp::Duration),
                    enum_method(
                        "easing",
                        &[
                            "linear",
                            "ease",
                            "ease_in",
                            "ease_out",
                            "ease_in_out",
                            "enter",
                            "exit",
                            "move",
                        ],
                        MotionOp::Easing,
                    ),
                    number_method("delay_ms", MotionOp::Delay),
                    number_method("fade_from", MotionOp::FadeFrom),
                    number_method("fade_to", MotionOp::FadeTo),
                    number_method("slide_from", MotionOp::SlideFrom),
                    number_method("slide_to", MotionOp::SlideTo),
                ])
                .with_documentation("A container that stays mounted through its exit animation."),
        )
        .expect("the Presence descriptor is valid");

    let mut reveal_methods = motion_policy_methods();
    reveal_methods.push(boolean_method("open", MotionOp::Present));
    registry
        .register(
            ComponentDescriptor::new("Reveal", Arc::new(RevealMaterializer))
                .with_constructors(vec![id_constructor("Reveal", |id| {
                    ComponentPayload::new(RevealPayload(id))
                })])
                .with_methods(reveal_methods)
                .with_documentation(
                    "A measured, clipped vertical reveal driven by animated progress.",
                ),
        )
        .expect("the Reveal descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_presence_and_reveal_register() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        for name in ["Motion", "Presence", "Reveal"] {
            assert!(frozen
                .descriptors()
                .any(|descriptor| descriptor.name() == name));
        }
    }

    #[test]
    fn keyframe_dsl_parses() {
        let frames = parse_keyframes("0:0.2:linear;0.5:1:ease_out;1:0.2").unwrap();
        assert_eq!(frames.len(), 3);
        assert!(parse_keyframes("nonsense").is_err());
    }

    #[test]
    fn presence_declares_only_transition_methods() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        let presence = frozen
            .descriptors()
            .find(|descriptor| descriptor.name() == "Presence")
            .unwrap();
        assert!(presence.method("present").is_some());
        assert!(presence.method("kind").is_none());
    }
}
