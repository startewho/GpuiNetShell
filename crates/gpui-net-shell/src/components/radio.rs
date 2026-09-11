//! `Radio`: one controlled option, matching `gpui-component`'s `Radio`.
//!
//! Like `Checkbox`, it holds nothing — the managed host says which option is
//! checked and re-reads its state every render. A radio cannot deselect itself,
//! so the callback only fires for a newly chosen option.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _, SharedString};
use gpui_component::radio::Radio;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct IdPayload(String);

#[derive(Clone)]
struct LabelOp(String);

#[derive(Clone)]
struct CheckedOp(bool);

fn id_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(id)] => Ok(ComponentPayload::new(IdPayload(id.clone()))),
        _ => Err("Radio(id) expects one string".into()),
    }
}

fn label_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "label",
        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
        |args| match args {
            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(LabelOp(value.clone()))),
            _ => Err("Radio.label expects one string".into()),
        },
    )
    .with_documentation("Sets the option label.")
}

fn checked_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "checked",
        vec![ArgumentDescriptor::new("checked", ArgumentSchema::Boolean)],
        |args| {
            let value = args
                .first()
                .map(ComponentArgument::is_truthy)
                .unwrap_or(true);
            Ok(ComponentPayload::new(CheckedOp(value)))
        },
    )
    .with_documentation("Sets the controlled checked state.")
}

struct RadioMaterializer;

impl ComponentMaterializer for RadioMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Radio received an incompatible payload".to_string())?
            .0
            .clone();

        let mut label = None;
        let mut checked = false;
        for method in request.methods() {
            match method.name() {
                "label" => {
                    if let Some(op) = method.payload().downcast_ref::<LabelOp>() {
                        label = Some(op.0.clone());
                    }
                }
                "checked" => {
                    if let Some(op) = method.payload().downcast_ref::<CheckedOp>() {
                        checked = op.0;
                    }
                }
                _ => {}
            }
        }

        let mut radio = Radio::new(SharedString::from(id))
            .checked(checked)
            .disabled(request.disabled());
        if let Some(label) = label {
            radio = radio.label(SharedString::from(label));
        }

        if let Some(token) = request.on_click() {
            let host = request.host().clone();
            radio = radio.on_change(move |_checked, _window, cx| {
                if let Some(click) = host.callbacks.click {
                    // SAFETY: the managed callback copies anything it keeps.
                    unsafe {
                        let _ = click(host.session_id, token);
                    }
                }
                (host.invalidate)(cx);
            });
        }

        request.finish(div().child(radio))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Radio", Arc::new(RadioMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Radio",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    id_payload,
                )])
                .with_methods(vec![label_method(), checked_method()])
                .with_documentation("A controlled radio option."),
        )
        .expect("the built-in Radio descriptor is valid");
}
