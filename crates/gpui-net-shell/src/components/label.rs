//! `Label`, ported from `component-shell`'s `controls/text.rs`.
//!
//! A `gpui-component` label with optional secondary text, masking, and
//! highlights. Shell disabled, selected, children, style, and on_click are
//! honored.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _};
use gpui_component::label::Label;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct StringPayload(String);

#[derive(Clone)]
enum LabelOp {
    Secondary(String),
    Masked(bool),
    Highlights(String),
}

fn string_constructor(export: &'static str, argument: &'static str) -> ConstructorDescriptor {
    ConstructorDescriptor::new(
        export,
        vec![ArgumentDescriptor::new(argument, ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(value)] if !value.is_empty() => {
                Ok(ComponentPayload::new(StringPayload(value.to_owned())))
            }
            [ComponentArgument::String(_)] => Err(format!("{export} {argument} must not be empty")),
            _ => Err(format!("{export} expects one string {argument}")),
        },
    )
}

fn label_string_method(
    name: &'static str,
    docs: &'static str,
    make: fn(String) -> LabelOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(make(value.to_owned()))),
            _ => Err(format!("Label.{name} expects one string")),
        },
    )
    .with_documentation(docs)
}

struct LabelMaterializer;

impl ComponentMaterializer for LabelMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let text = request
            .payload()
            .downcast_ref::<StringPayload>()
            .ok_or_else(|| "Label received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<LabelOp>().cloned())
            .collect::<Vec<_>>();
        let mut component = Label::new(text);
        for operation in operations {
            component = match operation {
                LabelOp::Secondary(value) => component.secondary(value),
                LabelOp::Masked(value) => component.masked(value),
                LabelOp::Highlights(value) => component.highlights(value),
            };
        }
        request.finish(div().child(component))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Label", Arc::new(LabelMaterializer))
                .with_constructors(vec![string_constructor("Label", "text")])
                .with_methods(vec![
                    label_string_method(
                        "secondary",
                        "Adds muted secondary text.",
                        LabelOp::Secondary,
                    ),
                    MethodDescriptor::new(
                        "masked",
                        vec![ArgumentDescriptor::new("masked", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(LabelOp::Masked(*value)))
                            }
                            _ => Err("Label.masked expects one boolean".into()),
                        },
                    )
                    .with_documentation("Controls whether the main text is masked."),
                    label_string_method(
                        "highlights",
                        "Highlights matching text fragments.",
                        LabelOp::Highlights,
                    ),
                ])
                .with_documentation(
                    "A text label with optional secondary, masking, and highlight presentation.",
                ),
        )
        .expect("the built-in Label descriptor is valid");
}
