//! `Kbd`, ported from `component-shell`'s `controls/text.rs`.
//!
//! A platform-formatted keyboard shortcut keycap. The constructor parses a
//! keystroke string; `appearance` and `outline` refine the keycap.

use std::sync::Arc;

use gpui::{div, AnyElement, Keystroke, ParentElement as _};
use gpui_component::kbd::Kbd;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
enum KbdOp {
    Appearance(bool),
    Outline,
}

struct KbdMaterializer;

impl ComponentMaterializer for KbdMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let stroke = request
            .payload()
            .downcast_ref::<Keystroke>()
            .ok_or_else(|| "Kbd received an incompatible payload".to_string())?;
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<KbdOp>().cloned())
            .collect::<Vec<_>>();
        let mut component = Kbd::new(stroke.clone());
        for operation in operations {
            component = match operation {
                KbdOp::Appearance(value) => component.appearance(value),
                KbdOp::Outline => component.outline(),
            };
        }
        request.finish(div().child(component))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Kbd", Arc::new(KbdMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Kbd",
                    vec![ArgumentDescriptor::new("keystroke", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(value)] => Keystroke::parse(value)
                            .map(ComponentPayload::new)
                            .map_err(|error| format!("invalid Kbd keystroke: {error}")),
                        _ => Err("Kbd expects one keystroke string".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "appearance",
                        vec![ArgumentDescriptor::new(
                            "appearance",
                            ArgumentSchema::Boolean,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(KbdOp::Appearance(*value)))
                            }
                            _ => Err("Kbd.appearance expects one boolean".into()),
                        },
                    )
                    .with_documentation("Controls whether the keystroke uses keycap presentation."),
                    MethodDescriptor::new("outline", Vec::new(), |_| {
                        Ok(ComponentPayload::new(KbdOp::Outline))
                    })
                    .with_documentation("Uses the outlined keycap presentation."),
                ])
                .with_documentation("A platform-formatted keyboard shortcut keycap."),
        )
        .expect("the built-in Kbd descriptor is valid");
}
