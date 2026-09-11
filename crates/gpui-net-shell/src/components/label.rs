//! `Label`: a styled label wrapping `gpui-component`'s `Label`.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _, SharedString};
use gpui_component::label::Label;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest,
};

#[derive(Clone)]
struct TextPayload(String);

struct LabelMaterializer;

fn label_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(value)] => Ok(ComponentPayload::new(TextPayload(value.clone()))),
        _ => Err("Label(value) expects one string".into()),
    }
}

impl ComponentMaterializer for LabelMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let value = request
            .payload()
            .downcast_ref::<TextPayload>()
            .ok_or_else(|| "Label received an incompatible payload".to_string())?
            .0
            .clone();
        request.finish(div().child(Label::new(SharedString::from(value))))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Label", Arc::new(LabelMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Label",
                    vec![ArgumentDescriptor::new("value", ArgumentSchema::String)],
                    label_payload,
                )])
                .with_methods(Vec::new())
                .with_documentation("A styled text label."),
        )
        .expect("the built-in Label descriptor is valid");
}
