//! `Div`: a plain container with the shared style surface and ordinary children.

use std::sync::Arc;

use gpui::{div, AnyElement};

use crate::registry::{
    ComponentDescriptor, ComponentMaterializer, ComponentPayload, ComponentRegistry,
    ConstructorDescriptor, MaterializeRequest,
};

struct DivMaterializer;

impl ComponentMaterializer for DivMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request.finish(div())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Div", Arc::new(DivMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new("Div", Vec::new(), |_| {
                    Ok(ComponentPayload::new(()))
                })])
                .with_methods(Vec::new())
                .with_documentation("A plain container; styling and children are the shell's."),
        )
        .expect("the built-in Div descriptor is valid");
}
