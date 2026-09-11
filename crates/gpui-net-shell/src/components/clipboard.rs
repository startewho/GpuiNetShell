//! `Clipboard`, ported from `component-shell`'s `display/clipboard.rs`.
//!
//! A button that copies a configured string to the system clipboard.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _};
use gpui_component::clipboard::Clipboard;

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct ClipboardPayload {
    id: String,
}

#[derive(Clone)]
enum ClipboardOp {
    Value(String),
    Tooltip(String),
}

struct ClipboardMaterializer;

impl ClipboardMaterializer {
    fn component<'a>(
        payload: &ComponentPayload,
        operations: impl IntoIterator<Item = &'a ClipboardOp>,
    ) -> Result<Clipboard, String> {
        let payload = payload
            .downcast_ref::<ClipboardPayload>()
            .ok_or_else(|| "Clipboard received an incompatible payload".to_string())?;
        Ok(operations.into_iter().fold(
            Clipboard::new(payload.id.clone()),
            |component, operation| match operation {
                ClipboardOp::Value(value) => component.value(value.clone()),
                ClipboardOp::Tooltip(text) => component.tooltip(text.clone()),
            },
        ))
    }
}

impl ComponentMaterializer for ClipboardMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("Clipboard does not accept children".to_string());
        }
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ClipboardOp>());
        let component = Self::component(request.payload(), operations)?;
        request.finish(div().child(component))
    }
}

fn string_method(
    name: &'static str,
    documentation: &'static str,
    wrap: fn(String) -> ClipboardOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(wrap(value.clone()))),
            _ => Err(format!("Clipboard.{name}({name}) expects a string")),
        },
    )
    .with_documentation(documentation)
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Clipboard", Arc::new(ClipboardMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Clipboard",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => {
                            Ok(ComponentPayload::new(ClipboardPayload {
                                id: nonempty_id(id, "Clipboard")?,
                            }))
                        }
                        _ => Err("Clipboard(id) expects a string".into()),
                    },
                )])
                .with_methods(vec![
                    string_method(
                        "value",
                        "Sets the text copied when the button is pressed.",
                        ClipboardOp::Value,
                    ),
                    string_method(
                        "tooltip",
                        "Sets the copy button tooltip.",
                        ClipboardOp::Tooltip,
                    ),
                ])
                .with_documentation(
                    "A button that copies a configured string to the system clipboard.",
                ),
        )
        .expect("the built-in Clipboard descriptor is valid");
}
