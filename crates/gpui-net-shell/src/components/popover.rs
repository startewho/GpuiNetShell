//! `Popover`: an anchored surface with a `trigger` and a `content` slot.
//!
//! The two parts are named slots rather than ordinary children, matching the
//! shell's own arrangement: the trigger is what is on screen while the surface
//! is closed, and the content is painted above the window when it opens.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _, SharedString};
use gpui_base::Popover;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct IdPayload(String);

#[derive(Clone)]
struct DefaultOpenOp(bool);

#[derive(Clone)]
struct OverlayClosableOp(bool);

fn id_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(id)] => Ok(ComponentPayload::new(IdPayload(id.clone()))),
        _ => Err("Popover(id) expects one string".into()),
    }
}

fn bool_method(
    name: &'static str,
    docs: &'static str,
    make: fn(bool) -> DefaultOpenOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |args| {
            let value = args
                .first()
                .map(ComponentArgument::is_truthy)
                .unwrap_or(true);
            Ok(ComponentPayload::new(make(value)))
        },
    )
    .with_documentation(docs)
}

fn overlay_closable_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "overlay_closable",
        vec![ArgumentDescriptor::new(
            "overlay_closable",
            ArgumentSchema::Boolean,
        )],
        |args| {
            let value = args
                .first()
                .map(ComponentArgument::is_truthy)
                .unwrap_or(true);
            Ok(ComponentPayload::new(OverlayClosableOp(value)))
        },
    )
    .with_documentation("Sets whether pressing outside closes the surface.")
}

struct PopoverMaterializer;

impl ComponentMaterializer for PopoverMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Popover received an incompatible payload".to_string())?
            .0
            .clone();

        let mut default_open = false;
        let mut overlay_closable = true;
        for method in request.methods() {
            match method.name() {
                "default_open" => {
                    if let Some(op) = method.payload().downcast_ref::<DefaultOpenOp>() {
                        default_open = op.0;
                    }
                }
                "overlay_closable" => {
                    if let Some(op) = method.payload().downcast_ref::<OverlayClosableOp>() {
                        overlay_closable = op.0;
                    }
                }
                _ => {}
            }
        }

        let trigger = request.take_slot("trigger");
        let content = request.take_slot("content");

        let mut popover = Popover::new(SharedString::from(id))
            .default_open(default_open)
            .overlay_closable(overlay_closable);
        if let Some(trigger) = trigger {
            // `trigger` needs a `Selectable`, which a materialized `AnyElement`
            // is not; `trigger_with` is the entry point for exactly this.
            popover = popover.trigger_with(move |_, _, _| trigger);
        }
        if let Some(content) = content {
            popover = popover.content(move |_, _, _| content);
        }

        request.finish(div().child(popover))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Popover", Arc::new(PopoverMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Popover",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    id_payload,
                )])
                .with_methods(vec![
                    bool_method(
                        "default_open",
                        "Starts the surface open when it is uncontrolled.",
                        DefaultOpenOp,
                    ),
                    overlay_closable_method(),
                ])
                .with_documentation(
                    "An anchored surface. `trigger` and `content` are named slots; pressing \
                     outside closes it.",
                ),
        )
        .expect("the built-in Popover descriptor is valid");
}
