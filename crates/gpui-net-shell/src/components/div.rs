//! `Div`: a plain container with the shared style surface and ordinary children.
//!
//! A managed `Div` may subscribe to a subset of GPUI's element events (click,
//! mouse, hover, scroll, key). Only the subscribed events are bound, so an
//! unsubscribed event costs nothing: no listener and no ABI traffic. Events
//! that GPUI keys by element id (click, aux-click, hover) require a stable
//! `element_id`, which the managed side supplies and which must not change
//! while the element is rendered.

use std::sync::Arc;

use gpui::{
    div, AnyElement, InteractiveElement as _, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _,
};

use crate::element_events;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
enum DivOp {
    /// The stable element id GPUI keys element-event state by.
    ElementId(String),
}

struct DivMaterializer;

impl ComponentMaterializer for DivMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let style = request.take_style();
        let children = request.take_children();
        let element_id = request.methods().find_map(|method| {
            method
                .payload()
                .downcast_ref::<DivOp>()
                .map(|DivOp::ElementId(id)| id.clone())
        });
        let events = request.events().clone();

        let mut element = div();
        element.style().refine(&style);
        element.extend(children);

        if events.is_empty() {
            return Ok(element.into_any_element());
        }

        let host = request.host().clone();
        if events.needs_element_id() {
            let id = element_id.filter(|id| !id.is_empty()).ok_or_else(|| {
                "a Div click, aux-click, or hover handler needs a stable element id".to_string()
            })?;
            let stateful = element.id(SharedString::from(format!("shell-div:{id}")));
            return Ok(element_events::bind_stateful(stateful, &events, &host).into_any_element());
        }
        Ok(element_events::bind_stateless(element, &events, &host).into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Div", Arc::new(DivMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new("Div", Vec::new(), |_| {
                    Ok(ComponentPayload::new(()))
                })])
                .with_methods(vec![MethodDescriptor::new(
                    "element_id",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => {
                            Ok(ComponentPayload::new(DivOp::ElementId(id.clone())))
                        }
                        _ => Err("Div.element_id(id) expects a string".into()),
                    },
                )
                .with_documentation(
                    "Stable identity GPUI keys a Div's element-event state by across frames.",
                )])
                .with_documentation("A plain container; styling and children are the shell's."),
        )
        .expect("the built-in Div descriptor is valid");
}
