//! `Collapsible`, ported from `component-shell`'s `compound/collapsible.rs`.
//!
//! A trigger container with an optional named `content` reveal. Ordinary
//! children are the trigger; `open` and `motion_id` refine the reveal.

use std::sync::Arc;

use gpui::AnyElement;
use gpui_component::collapsible::Collapsible;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone, Copy)]
struct CollapsiblePayload;

#[derive(Clone)]
enum CollapsibleOp {
    Open(bool),
    MotionId(String),
}

struct CollapsibleMaterializer;

impl ComponentMaterializer for CollapsibleMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<CollapsiblePayload>()
            .ok_or_else(|| "Collapsible received an incompatible payload".to_string())?;
        let mut component = Collapsible::new();
        for op in request
            .methods()
            .filter_map(|m| m.payload().downcast_ref::<CollapsibleOp>())
        {
            component = match op {
                CollapsibleOp::Open(value) => component.open(*value),
                CollapsibleOp::MotionId(id) => component.motion_id(id.clone()),
            };
        }
        if let Some(content) = request.take_slot("content") {
            component = component.content(content);
        }
        request.finish(component)
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Collapsible", Arc::new(CollapsibleMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Collapsible",
                    vec![],
                    |_| Ok(ComponentPayload::new(CollapsiblePayload)),
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "open",
                        vec![ArgumentDescriptor::new("open", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(CollapsibleOp::Open(*value)))
                            }
                            _ => Err("Collapsible.open(open) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Controls whether the content slot is revealed."),
                    MethodDescriptor::new(
                        "motion_id",
                        vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(
                                CollapsibleOp::MotionId(value.clone()),
                            )),
                            _ => Err("Collapsible.motion_id(id) expects a string".into()),
                        },
                    )
                    .with_documentation("Adds stable identity for a reversible measured reveal."),
                ])
                .with_documentation(
                    "A trigger container with optional named `content` reveal content.",
                ),
        )
        .expect("the built-in Collapsible descriptor is valid");
}
