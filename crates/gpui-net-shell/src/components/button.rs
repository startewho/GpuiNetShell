//! `Button`: the control binding ported from `component-shell`'s `action.rs`.
//!
//! Identity is the constructor payload; label, tooltip, loading, size, compact
//! and the visual variants are registered methods; disabled/selected/on_click
//! are the shell's behavior and arrive on the request.

use std::sync::Arc;

use gpui::AnyElement;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::{Disableable as _, Selectable as _, Sizable as _, Size};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
pub(crate) struct IdPayload(String);

#[derive(Clone)]
enum ButtonOp {
    Size(Size),
    Label(String),
    Tooltip(String),
    Loading(bool),
    Outline,
    Primary,
    Secondary,
    Danger,
    Success,
    Warning,
    Ghost,
    Link,
    Compact,
}

fn validate_id(export: &str, id: &str) -> Result<String, String> {
    if id.is_empty() {
        Err(format!("{export} id must not be empty"))
    } else {
        Ok(id.to_owned())
    }
}

fn id_constructor(export: &'static str) -> ConstructorDescriptor {
    ConstructorDescriptor::new(
        export,
        vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(id)] => {
                validate_id(export, id).map(|id| ComponentPayload::new(IdPayload(id)))
            }
            _ => Err(format!("{export}(id) expects one string")),
        },
    )
}

fn button_string(
    name: &'static str,
    docs: &'static str,
    make: fn(String) -> ButtonOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::String)],
        move |args| match args {
            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(make(value.clone()))),
            _ => Err(format!("Button.{name} expects one string")),
        },
    )
    .with_documentation(docs)
}

fn button_loading() -> MethodDescriptor {
    MethodDescriptor::new(
        "loading",
        vec![ArgumentDescriptor::new("loading", ArgumentSchema::Boolean)],
        |args| {
            let value = args
                .first()
                .map(ComponentArgument::is_truthy)
                .unwrap_or(true);
            Ok(ComponentPayload::new(ButtonOp::Loading(value)))
        },
    )
    .with_documentation("Sets the loading presentation.")
}

fn button_size() -> MethodDescriptor {
    MethodDescriptor::new(
        "size",
        vec![ArgumentDescriptor::new("size", ArgumentSchema::Number)],
        |args| {
            let value = args
                .first()
                .and_then(ComponentArgument::as_f64)
                .unwrap_or(2.0) as u64;
            let size = match value {
                0 => Size::XSmall,
                1 => Size::Small,
                3 => Size::Large,
                _ => Size::Medium,
            };
            Ok(ComponentPayload::new(ButtonOp::Size(size)))
        },
    )
    .with_documentation("Sets the semantic control size.")
}

fn variant_method(name: &'static str, op: ButtonOp, docs: &'static str) -> MethodDescriptor {
    MethodDescriptor::new(name, Vec::new(), move |_| {
        Ok(ComponentPayload::new(op.clone()))
    })
    .with_documentation(docs)
}

struct ButtonMaterializer;

impl ComponentMaterializer for ButtonMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Button received an incompatible payload".to_string())?;

        let mut component = Button::new(id.0.clone())
            .disabled(request.disabled())
            .selected(request.selected());

        for method in request.methods() {
            let Some(op) = method.payload().downcast_ref::<ButtonOp>() else {
                continue;
            };
            component = match op {
                ButtonOp::Size(size) => component.with_size(*size),
                ButtonOp::Label(label) => component.label(label.clone()),
                ButtonOp::Tooltip(tooltip) => component.tooltip(tooltip.clone()),
                ButtonOp::Loading(loading) => component.loading(*loading),
                ButtonOp::Outline => component.outline(),
                ButtonOp::Primary => component.primary(),
                ButtonOp::Secondary => component.secondary(),
                ButtonOp::Danger => component.danger(),
                ButtonOp::Success => component.success(),
                ButtonOp::Warning => component.warning(),
                ButtonOp::Ghost => component.ghost(),
                ButtonOp::Link => component.link(),
                ButtonOp::Compact => component.compact(),
            };
        }

        if let Some(token) = request.on_click() {
            let host = request.host().clone();
            component = component.on_click(move |_event, _window, cx| {
                if let Some(click) = host.callbacks.click {
                    // SAFETY: managed callback; it copies anything it keeps.
                    unsafe {
                        let _ = click(host.session_id, token);
                    }
                }
                (host.invalidate)(cx);
            });
        }

        request.finish(component)
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Button", Arc::new(ButtonMaterializer))
                .with_constructors(vec![id_constructor("Button")])
                .with_methods(vec![
                    button_string("label", "Sets the visible button label.", ButtonOp::Label),
                    button_string("tooltip", "Sets concise hover help.", ButtonOp::Tooltip),
                    button_loading(),
                    button_size(),
                    variant_method(
                        "outline",
                        ButtonOp::Outline,
                        "Uses the outline presentation.",
                    ),
                    variant_method(
                        "primary",
                        ButtonOp::Primary,
                        "Uses the primary action variant.",
                    ),
                    variant_method(
                        "secondary",
                        ButtonOp::Secondary,
                        "Uses the secondary variant.",
                    ),
                    variant_method(
                        "danger",
                        ButtonOp::Danger,
                        "Uses the destructive-action variant.",
                    ),
                    variant_method("success", ButtonOp::Success, "Uses the success variant."),
                    variant_method("warning", ButtonOp::Warning, "Uses the warning variant."),
                    variant_method("ghost", ButtonOp::Ghost, "Uses the quiet ghost variant."),
                    variant_method("link", ButtonOp::Link, "Uses the link-like visual variant."),
                    MethodDescriptor::new("compact", Vec::new(), |_| {
                        Ok(ComponentPayload::new(ButtonOp::Compact))
                    })
                    .with_documentation("Uses compact internal spacing."),
                ])
                .with_documentation(
                    "A stateless command button. Shell disabled, selected, children, style, \
                     and on_click are honored.",
                ),
        )
        .expect("the built-in Button descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_rejects_an_empty_id() {
        assert_eq!(
            validate_id("Button", ""),
            Err("Button id must not be empty".into())
        );
        assert_eq!(validate_id("Button", "save"), Ok("save".into()));
    }

    #[test]
    fn constructors_and_methods_record_their_payloads() {
        let frozen = crate::components::catalog();
        let descriptor = frozen
            .descriptors()
            .find(|descriptor| descriptor.name() == "Button")
            .expect("Button is registered");

        let payload = descriptor.constructors()[0]
            .payload(&[ComponentArgument::String("save".into())])
            .unwrap();
        assert_eq!(payload.downcast_ref::<IdPayload>().unwrap().0, "save");

        let label = descriptor
            .method("label")
            .unwrap()
            .record(&[ComponentArgument::String("Save".into())])
            .unwrap();
        assert!(label.downcast_ref::<ButtonOp>().is_some());
    }
}
