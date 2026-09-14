//! `Motion` and `Presence`: gpui-kit motion animation exposed to managed code.
//!
//! `Motion` interpolates opacity, translation, and size toward targets supplied
//! each render; `Presence` keeps a child mounted through its exit animation.
//! Both sample the `gpui_base::motion` primitives, which request animation
//! frames while active, so managed code is not involved per frame.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    div, px, AnyElement, App, ElementId, IntoElement as _, ParentElement as _, SharedString,
    Styled as _, Window,
};
use gpui_base::motion::{transition, Easing, Interpolate, Presence, Transition};

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
enum MotionOp {
    Duration(f32),
    /// The easing name; parsed at materialization (kept `Send` for the payload).
    Easing(String),
    Opacity(f32),
    TranslateX(f32),
    TranslateY(f32),
    Width(f32),
    Height(f32),
}

#[derive(Clone)]
enum PresenceOp {
    Present(bool),
    Duration(f32),
    /// The easing name; parsed at materialization (kept `Send` for the payload).
    Easing(String),
    FadeFrom(f32),
    FadeTo(f32),
    SlideFrom(f32),
    SlideTo(f32),
}

fn easing_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "easing",
        vec![ArgumentDescriptor::new(
            "easing",
            ArgumentSchema::Enum(&["linear", "ease", "ease_in", "ease_out", "ease_in_out"]),
        )],
        |arguments| match arguments {
            [value] => value
                .as_str()
                .map(|name| ComponentPayload::new(MotionOp::Easing(name.to_string())))
                .ok_or_else(|| "easing(name) expects a string".to_string()),
            _ => Err("easing(name) expects a string".into()),
        },
    )
    .with_documentation("Sets the easing curve: linear, ease, ease_in, ease_out, ease_in_out.")
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
    .with_documentation("Sets an animation target.")
}

fn parse_easing(name: &str) -> Easing {
    match name {
        "linear" => Easing::Linear,
        "ease" => Easing::Ease,
        "ease_in" => Easing::EaseIn,
        "ease_out" => Easing::EaseOut,
        "ease_in_out" => Easing::EaseInOut,
        _ => Easing::EaseOut,
    }
}

fn sample<T: Interpolate + PartialEq + 'static>(
    key: &SharedString,
    channel: &'static str,
    target: T,
    policy: &Transition,
    window: &mut Window,
    cx: &mut App,
) -> T {
    transition((key.clone(), channel), target, policy.clone(), window, cx)
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

        let (mut duration_ms, mut easing) = (200.0_f32, Easing::EaseOut);
        let (mut opacity, mut translate_x, mut translate_y) = (None, None, None);
        let (mut width, mut height) = (None, None);
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<MotionOp>().cloned())
        {
            match op {
                MotionOp::Duration(value) => duration_ms = value,
                MotionOp::Easing(name) => easing = parse_easing(&name),
                MotionOp::Opacity(value) => opacity = Some(value),
                MotionOp::TranslateX(value) => translate_x = Some(value),
                MotionOp::TranslateY(value) => translate_y = Some(value),
                MotionOp::Width(value) => width = Some(value),
                MotionOp::Height(value) => height = Some(value),
            }
        }

        let key = SharedString::from(id);
        let policy =
            Transition::new(Duration::from_millis(duration_ms.max(0.0) as u64)).easing(easing);
        let sampled = request.with_window_app(|window, cx| {
            (
                opacity.map(|t| sample(&key, "opacity", t, &policy, window, cx)),
                translate_x.map(|t| sample(&key, "x", t, &policy, window, cx)),
                translate_y.map(|t| sample(&key, "y", t, &policy, window, cx)),
                width.map(|t| sample(&key, "w", t, &policy, window, cx)),
                height.map(|t| sample(&key, "h", t, &policy, window, cx)),
            )
        });
        let (opacity, translate_x, translate_y, width, height) = sampled;

        let children = request.take_children();
        let mut wrapper = div();
        if translate_x.is_some() || translate_y.is_some() {
            wrapper = wrapper.relative();
        }
        if let Some(value) = opacity {
            wrapper = wrapper.opacity(value);
        }
        if let Some(value) = translate_x {
            wrapper = wrapper.left(px(value));
        }
        if let Some(value) = translate_y {
            wrapper = wrapper.top(px(value));
        }
        if let Some(value) = width {
            wrapper = wrapper.w(px(value));
        }
        if let Some(value) = height {
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

        let (mut present, mut duration_ms, mut easing) = (false, 200.0_f32, Easing::EaseInOut);
        let mut fade = (0.0_f32, 1.0_f32);
        let mut slide = (8.0_f32, 0.0_f32);
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<PresenceOp>().cloned())
        {
            match op {
                PresenceOp::Present(value) => present = value,
                PresenceOp::Duration(value) => duration_ms = value,
                PresenceOp::Easing(name) => easing = parse_easing(&name),
                PresenceOp::FadeFrom(value) => fade.0 = value,
                PresenceOp::FadeTo(value) => fade.1 = value,
                PresenceOp::SlideFrom(value) => slide.0 = value,
                PresenceOp::SlideTo(value) => slide.1 = value,
            }
        }

        let key = ElementId::Name(SharedString::from(id.clone()));
        let policy =
            Transition::new(Duration::from_millis(duration_ms.max(0.0) as u64)).easing(easing);
        let sample = request.with_window_app(|window, cx| {
            Presence::new(key, present)
                .transition(policy)
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

fn presence_number(name: &'static str, make: fn(f32) -> PresenceOp) -> MethodDescriptor {
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
    .with_documentation("Sets an animation target.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Motion", Arc::new(MotionMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Motion",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(MotionPayload(id.clone())))
                        }
                        _ => Err("Motion expects a non-empty id".into()),
                    },
                )])
                .with_methods(vec![
                    number_method("duration_ms", MotionOp::Duration),
                    easing_method(),
                    number_method("opacity", MotionOp::Opacity),
                    number_method("translate_x", MotionOp::TranslateX),
                    number_method("translate_y", MotionOp::TranslateY),
                    number_method("width", MotionOp::Width),
                    number_method("height", MotionOp::Height),
                ])
                .with_documentation(
                    "A container that interpolates opacity/translation/size to targets.",
                ),
        )
        .expect("the Motion descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Presence", Arc::new(PresenceMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Presence",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(PresencePayload(id.clone())))
                        }
                        _ => Err("Presence expects a non-empty id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "present",
                        vec![ArgumentDescriptor::new("present", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(PresenceOp::Present(*value)))
                            }
                            [ComponentArgument::Number(value)] => {
                                Ok(ComponentPayload::new(PresenceOp::Present(*value != 0.0)))
                            }
                            _ => Err("present(flag) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Whether the content is present (false animates out)."),
                    presence_number("duration_ms", PresenceOp::Duration),
                    MethodDescriptor::new(
                        "easing",
                        vec![ArgumentDescriptor::new(
                            "easing",
                            ArgumentSchema::Enum(&[
                                "linear",
                                "ease",
                                "ease_in",
                                "ease_out",
                                "ease_in_out",
                            ]),
                        )],
                        |arguments| match arguments {
                            [value] => value
                                .as_str()
                                .map(|name| {
                                    ComponentPayload::new(PresenceOp::Easing(name.to_string()))
                                })
                                .ok_or_else(|| "easing(name) expects a string".to_string()),
                            _ => Err("easing(name) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the easing curve."),
                    presence_number("fade_from", PresenceOp::FadeFrom),
                    presence_number("fade_to", PresenceOp::FadeTo),
                    presence_number("slide_from", PresenceOp::SlideFrom),
                    presence_number("slide_to", PresenceOp::SlideTo),
                ])
                .with_documentation("A container that stays mounted through its exit animation."),
        )
        .expect("the Presence descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_and_presence_register() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        assert!(frozen
            .descriptors()
            .any(|descriptor| descriptor.name() == "Motion"));
        assert!(frozen
            .descriptors()
            .any(|descriptor| descriptor.name() == "Presence"));
    }
}
