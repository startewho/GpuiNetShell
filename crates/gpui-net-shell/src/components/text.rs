//! `Text`, ported from `component-shell`'s `basic/text.rs`.
//!
//! Plain `gpui-component` text in a styleable wrapper. `Text` takes its content
//! through the constructor and accepts no children.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _};
use gpui_component::text::Text;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest,
};

#[derive(Clone)]
struct TextPayload(String);

struct TextMaterializer;

fn text_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(value)] => Ok(ComponentPayload::new(TextPayload(value.clone()))),
        _ => Err("Text(value) expects one string".into()),
    }
}

impl ComponentMaterializer for TextMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let value = request
            .payload()
            .downcast_ref::<TextPayload>()
            .ok_or_else(|| "Text received an incompatible payload".to_string())?
            .0
            .clone();
        if request.children_len() != 0 {
            return Err(
                "Text does not accept children; pass its content to Text(value)".to_string(),
            );
        }
        request.finish(div().child(Text::from(value)))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Text", Arc::new(TextMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Text",
                    vec![ArgumentDescriptor::new("value", ArgumentSchema::String)],
                    text_payload,
                )])
                .with_methods(Vec::new())
                .with_documentation(
                    "Plain gpui-component Text content in a styleable shell wrapper. \
                     Text accepts no children.",
                ),
        )
        .expect("the built-in Text descriptor is valid");
}
