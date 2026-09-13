//! `EntityHost`: renders a managed entity's subtree as a retained native view.
//!
//! An `EntityHost` node's data is the managed entity id; its `render_entity`
//! callback builds the subtree. The native side keeps one `EntityHostView`
//! entity per id (a keyed state), so `notify_entity` can repaint only that
//! subtree rather than the whole window — the native counterpart of the managed
//! `Context::notify`, scoped to an entity.

use std::sync::Arc;

use gpui::prelude::*;
use gpui::{
    div, AnyElement, App, Context, ElementId, Refineable as _, Render, SharedString, Styled as _,
    Window,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    ElementCallback, MaterializeRequest, MethodDescriptor,
};

/// The retained native view for one managed entity.
pub(crate) struct EntityHostView {
    id: SharedString,
    renderer: Option<ElementCallback>,
}

impl Render for EntityHostView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match &self.renderer {
            Some(callback) => callback
                .build(&[self.id.to_string()], window, cx)
                .unwrap_or_else(|error| {
                    div()
                        .child(format!("Failed to render entity: {error}"))
                        .into_any_element()
                }),
            None => div().into_any_element(),
        }
    }
}

#[derive(Clone)]
struct EntityHostPayload {
    entity_id: String,
}

#[derive(Clone)]
enum EntityHostOp {
    Render(ComponentArgument),
}

struct EntityHostMaterializer;

impl ComponentMaterializer for EntityHostMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<EntityHostPayload>()
            .ok_or_else(|| "EntityHost received an incompatible payload".to_string())?
            .clone();
        let renderer = request
            .methods()
            .find_map(|method| match method.payload().downcast_ref::<EntityHostOp>() {
                Some(EntityHostOp::Render(argument)) => Some(argument.clone()),
                None => None,
            })
            .ok_or_else(|| "EntityHost requires a render_entity callback".to_string())
            .and_then(|argument| request.resolve_element_callback(&argument))?;

        if request.children_len() != 0 {
            return Err("EntityHost does not accept children".to_string());
        }

        let key = entity_key(&payload.entity_id);
        let entity_id = payload.entity_id.clone();
        let entity = request.use_keyed_state(key, move |_window, _cx| EntityHostView {
            id: SharedString::from(entity_id),
            renderer: None,
        });
        let renderer = renderer.clone();
        request.update_entity(&entity, |view, _| {
            view.renderer = Some(renderer);
        });

        let style = request.take_style();
        let mut wrapper = div().size_full().child(entity);
        wrapper.style().refine(&style);
        Ok(wrapper.into_any_element())
    }
}

/// The keyed-state name for one entity id. Shared with the host's
/// `notify_entity` path so both agree on the key.
pub(crate) fn entity_key(entity_id: &str) -> ElementId {
    ElementId::Name(SharedString::from(format!("shell-entity:{entity_id}")))
}

/// Repaints the retained entity subtree `entity_id` in `window`, if present.
///
/// `use_keyed_state` returns the existing entity without running the init
/// closure; a missing key creates a placeholder view that renders nothing until
/// the next full frame materializes it.
pub(crate) fn notify_entity(
    entity_id: u64,
    window: &mut Window,
    cx: &mut App,
) {
    let key = entity_key(&entity_id.to_string());
    let entity = window.use_keyed_state(key, cx, |_window, _cx| EntityHostView {
        id: SharedString::default(),
        renderer: None,
    });
    entity.update(cx, |_view, cx| cx.notify());
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("EntityHost", Arc::new(EntityHostMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "EntityHost",
                    vec![ArgumentDescriptor::new("entity_id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(EntityHostPayload {
                                entity_id: id.clone(),
                            }))
                        }
                        _ => Err("EntityHost expects a non-empty entity id".into()),
                    },
                )])
                .with_methods(vec![MethodDescriptor::new(
                    "render_entity",
                    vec![ArgumentDescriptor::new("callback", ArgumentSchema::Callback)],
                    |arguments| match arguments {
                        [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                            EntityHostOp::Render(ComponentArgument::Callback(*token)),
                        )),
                        _ => Err("EntityHost.render_entity(callback) expects a callback".into()),
                    },
                )
                .with_documentation("Builds the subtree for the entity id.")])
                .with_documentation(
                    "A retained managed entity subtree; `notify_entity` repaints it in place.",
                ),
        )
        .expect("the built-in EntityHost descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_entity_host_registers() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        assert!(frozen
            .descriptors()
            .any(|descriptor| descriptor.name() == "EntityHost"));
    }
}
