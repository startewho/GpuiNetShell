//! `Breadcrumb`, ported from `component-shell`'s `display/breadcrumb.rs`.
//!
//! A navigation trail built from an ordered list of labels. The labels arrive as
//! one newline-separated string; the component rejects ordinary children.

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, Refineable as _, Styled as _};
use gpui_component::breadcrumb::Breadcrumb;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest,
};

#[derive(Clone)]
struct BreadcrumbPayload(Vec<String>);

struct BreadcrumbMaterializer;

impl BreadcrumbMaterializer {
    fn component(payload: &ComponentPayload) -> Result<Breadcrumb, String> {
        let payload = payload
            .downcast_ref::<BreadcrumbPayload>()
            .ok_or_else(|| "Breadcrumb received an incompatible payload".to_string())?;
        Ok(Breadcrumb::new().children(payload.0.clone()))
    }
}

impl ComponentMaterializer for BreadcrumbMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("Breadcrumb does not accept children".to_string());
        }
        let mut component = Self::component(request.payload())?;
        component.style().refine(&request.take_style());
        Ok(component.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Breadcrumb", Arc::new(BreadcrumbMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Breadcrumb",
                    vec![ArgumentDescriptor::new("labels", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(labels)] => {
                            let labels = labels.split('\n').map(str::to_owned).collect();
                            Ok(ComponentPayload::new(BreadcrumbPayload(labels)))
                        }
                        _ => Err("Breadcrumb(labels) expects a newline-separated string".into()),
                    },
                )])
                .with_methods(Vec::new())
                .with_documentation(
                    "A navigation trail built from an ordered, newline-separated list of labels.",
                ),
        )
        .expect("the built-in Breadcrumb descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_real_breadcrumb() {
        drop(
            BreadcrumbMaterializer::component(&ComponentPayload::new(BreadcrumbPayload(vec![
                "Home".into(),
                "Settings".into(),
            ])))
            .unwrap()
            .into_any_element(),
        );
    }

    #[test]
    fn rejects_an_incompatible_payload() {
        assert_eq!(
            BreadcrumbMaterializer::component(&ComponentPayload::new(()))
                .err()
                .unwrap(),
            "Breadcrumb received an incompatible payload"
        );
    }
}
