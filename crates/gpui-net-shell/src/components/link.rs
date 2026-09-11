//! `Link`, ported from `component-shell`'s `controls/text.rs`.
//!
//! An external-resource link. Shell disabled and on_click are honored; `href`
//! sets the external URL opened when activated.

use std::sync::Arc;

use gpui::AnyElement;
use gpui_component::link::Link;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct StringPayload(String);

#[derive(Clone)]
enum LinkOp {
    Href(String),
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

struct LinkMaterializer;

impl ComponentMaterializer for LinkMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<StringPayload>()
            .ok_or_else(|| "Link received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<LinkOp>().cloned())
            .collect::<Vec<_>>();
        let mut component = Link::new(id).disabled(request.disabled());
        for operation in operations {
            match operation {
                LinkOp::Href(value) => component = component.href(value),
            }
        }
        if let Some(token) = request.on_click() {
            let host = request.host().clone();
            component = component.on_click(move |_event, _window, cx| {
                if let Some(click) = host.callbacks.click {
                    // SAFETY: managed callback; it copies anything it keeps.
                    unsafe {
                        let _ = click(host.session_id, token);
                    }
                }
                (host.invalidate)(cx);
            });
        }
        request.finish(component)
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Link", Arc::new(LinkMaterializer))
                .with_constructors(vec![string_constructor("Link", "id")])
                .with_methods(vec![MethodDescriptor::new(
                    "href",
                    vec![ArgumentDescriptor::new("href", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(value)] => {
                            Ok(ComponentPayload::new(LinkOp::Href(value.to_owned())))
                        }
                        _ => Err("Link.href expects one URL string".into()),
                    },
                )
                .with_documentation("Sets the external URL opened when activated.")])
                .with_documentation(
                    "An external-resource link. Shell disabled, children, style, and on_click \
                     are honored.",
                ),
        )
        .expect("the built-in Link descriptor is valid");
}
