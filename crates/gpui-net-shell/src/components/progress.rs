//! `Progress`: a determinate progress bar wrapping `gpui-component`'s
//! `Progress`.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _, SharedString, Styled as _};
use gpui_component::progress::Progress;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct IdPayload(String);

#[derive(Clone)]
enum ProgressOp {
    Value(f32),
    Loading(bool),
}

fn id_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(id)] => Ok(ComponentPayload::new(IdPayload(id.clone()))),
        _ => Err("Progress(id) expects one string".into()),
    }
}

fn value_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "value",
        vec![ArgumentDescriptor::new("value", ArgumentSchema::Number)],
        |args| {
            let value = args
                .first()
                .and_then(ComponentArgument::as_f64)
                .unwrap_or(0.0) as f32;
            Ok(ComponentPayload::new(ProgressOp::Value(value)))
        },
    )
    .with_documentation("Sets the completion percentage, from 0 to 100.")
}

fn loading_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "loading",
        vec![ArgumentDescriptor::new("loading", ArgumentSchema::Boolean)],
        |args| {
            let value = args
                .first()
                .map(ComponentArgument::is_truthy)
                .unwrap_or(true);
            Ok(ComponentPayload::new(ProgressOp::Loading(value)))
        },
    )
    .with_documentation("Shows the indeterminate loading animation.")
}

struct ProgressMaterializer;

impl ComponentMaterializer for ProgressMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Progress received an incompatible payload".to_string())?
            .0
            .clone();
        let mut progress = Progress::new(SharedString::from(id));
        for method in request.methods() {
            match method.payload().downcast_ref::<ProgressOp>() {
                Some(ProgressOp::Value(value)) => progress = progress.value(*value),
                Some(ProgressOp::Loading(loading)) => progress = progress.loading(*loading),
                None => {}
            }
        }
        request.finish(div().w_full().child(progress))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Progress", Arc::new(ProgressMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Progress",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    id_payload,
                )])
                .with_methods(vec![value_method(), loading_method()])
                .with_documentation("A determinate progress bar."),
        )
        .expect("the built-in Progress descriptor is valid");
}
