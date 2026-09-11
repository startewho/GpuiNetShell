//! `StatusBar`, ported from `component-shell`'s `display/status_bar.rs`.
//!
//! A three-region status bar. Ordinary children fill the center; the
//! `left_content` and `right_content` element arguments pin content to each
//! edge.

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, ParentElement as _, Refineable as _, Styled as _};
use gpui_component::status_bar::StatusBar;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone, Copy)]
struct StatusBarPayload;

#[derive(Clone)]
enum StatusBarOp {
    Left(ComponentArgument),
    Right(ComponentArgument),
}

struct StatusBarMaterializer;

impl StatusBarMaterializer {
    fn component(payload: &ComponentPayload) -> Result<StatusBar, String> {
        payload
            .downcast_ref::<StatusBarPayload>()
            .ok_or_else(|| "StatusBar received an incompatible payload".to_string())?;
        Ok(StatusBar::new())
    }
}

impl ComponentMaterializer for StatusBarMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut component = Self::component(request.payload())?;
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<StatusBarOp>().cloned())
            .collect::<Vec<_>>();
        for operation in operations {
            component = match operation {
                StatusBarOp::Left(argument) => component.left(request.resolve_element(&argument)?),
                StatusBarOp::Right(argument) => {
                    component.right(request.resolve_element(&argument)?)
                }
            };
        }
        component.style().refine(&request.take_style());
        component.extend(request.take_children());
        Ok(component.into_any_element())
    }
}

fn element_method(
    name: &'static str,
    documentation: &'static str,
    wrap: fn(ComponentArgument) -> StatusBarOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new("element", ArgumentSchema::Element)],
        move |arguments| match arguments {
            [argument @ ComponentArgument::Element(_)] => {
                Ok(ComponentPayload::new(wrap(argument.clone())))
            }
            _ => Err(format!("StatusBar.{name}(element) expects an element")),
        },
    )
    .with_documentation(documentation)
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("StatusBar", Arc::new(StatusBarMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "StatusBar",
                    Vec::new(),
                    |_| Ok(ComponentPayload::new(StatusBarPayload)),
                )])
                .with_methods(vec![
                    element_method(
                        "left_content",
                        "Appends content to the leading region.",
                        StatusBarOp::Left,
                    ),
                    element_method(
                        "right_content",
                        "Appends content to the trailing region.",
                        StatusBarOp::Right,
                    ),
                ])
                .with_documentation(
                    "A three-region status bar; ordinary children fill the center and the \
                     `left_content`/`right_content` elements pin content to each edge.",
                ),
        )
        .expect("the built-in StatusBar descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_real_status_bar() {
        drop(
            StatusBarMaterializer::component(&ComponentPayload::new(StatusBarPayload))
                .unwrap()
                .into_any_element(),
        );
    }

    #[test]
    fn rejects_an_incompatible_payload() {
        assert_eq!(
            StatusBarMaterializer::component(&ComponentPayload::new(()))
                .err()
                .unwrap(),
            "StatusBar received an incompatible payload"
        );
    }
}
