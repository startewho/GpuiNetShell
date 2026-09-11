//! `Tooltip`, ported from `component-shell`'s `lifecycle/tooltip.rs`.
//!
//! A button trigger with a managed text tooltip. Identity, trigger label, and
//! tooltip text all arrive through the constructor.

use std::sync::Arc;

use gpui::AnyElement;
use gpui_component::button::Button;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest,
};

#[derive(Clone)]
struct TooltipPayload {
    id: String,
    label: String,
    text: String,
}

struct TooltipMaterializer;

impl ComponentMaterializer for TooltipMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<TooltipPayload>()
            .ok_or_else(|| "Tooltip received an incompatible payload".to_string())?
            .clone();
        request.finish(
            Button::new(payload.id)
                .label(payload.label)
                .tooltip(payload.text),
        )
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Tooltip", Arc::new(TooltipMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Tooltip",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("label", ArgumentSchema::String),
                        ArgumentDescriptor::new("text", ArgumentSchema::String),
                    ],
                    |arguments| match arguments {
                        [
                            ComponentArgument::String(id),
                            ComponentArgument::String(label),
                            ComponentArgument::String(text),
                        ] if !id.trim().is_empty()
                            && !label.trim().is_empty()
                            && !text.trim().is_empty() =>
                        {
                            Ok(ComponentPayload::new(TooltipPayload {
                                id: id.clone(),
                                label: label.clone(),
                                text: text.clone(),
                            }))
                        }
                        [
                            ComponentArgument::String(_),
                            ComponentArgument::String(_),
                            ComponentArgument::String(_),
                        ] => Err("Tooltip id, label, and text must not be empty".into()),
                        _ => Err("Tooltip(id, label, text) expects three strings".into()),
                    },
                )])
                .with_methods(Vec::new())
                .with_documentation(
                    "A button trigger with a managed text tooltip.",
                ),
        )
        .expect("the built-in Tooltip descriptor is valid");
}
