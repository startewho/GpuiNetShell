//! `Badge`: a count/dot badge wrapping `gpui-component`'s `Badge`.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _};
use gpui_component::badge::Badge;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct CountPayload(u32);

#[derive(Clone)]
struct DotOp;

fn badge_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::Number(count)] if *count >= 0.0 => {
            Ok(ComponentPayload::new(CountPayload(*count as u32)))
        }
        [ComponentArgument::String(count)] => count
            .parse::<u32>()
            .map(CountPayload)
            .map(ComponentPayload::new)
            .map_err(|_| format!("Badge(count) expects an integer, got `{count}`")),
        _ => Err("Badge(count) expects one number".into()),
    }
}

fn dot_method() -> MethodDescriptor {
    MethodDescriptor::new("dot", Vec::new(), |_| Ok(ComponentPayload::new(DotOp)))
        .with_documentation("Renders the badge as a dot rather than a count.")
}

struct BadgeMaterializer;

impl ComponentMaterializer for BadgeMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let count = request
            .payload()
            .downcast_ref::<CountPayload>()
            .ok_or_else(|| "Badge received an incompatible payload".to_string())?
            .0;
        let mut badge = Badge::new().count(count as usize);
        for method in request.methods() {
            if method.payload().downcast_ref::<DotOp>().is_some() {
                badge = badge.dot();
            }
        }
        request.finish(div().child(badge))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Badge", Arc::new(BadgeMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Badge",
                    vec![ArgumentDescriptor::new("count", ArgumentSchema::Number)],
                    badge_payload,
                )])
                .with_methods(vec![dot_method()])
                .with_documentation("A small count or status badge."),
        )
        .expect("the built-in Badge descriptor is valid");
}
