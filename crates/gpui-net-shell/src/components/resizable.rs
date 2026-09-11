//! `Resizable`: a row or column of draggable panels.
//!
//! The shell reads its children as typed panels; here the sizes come from a
//! plain `sizes` string (`"200,*,200"`, where `*` is the flexible panel) and the
//! ordinary children are wrapped in `ResizablePanel`s in order. That keeps the
//! component declarative without a typed-child seam.

use std::sync::Arc;

use gpui::{div, px, AnyElement, ParentElement as _, SharedString, Styled as _};
use gpui_base::{h_resizable, resizable_panel, v_resizable};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct IdPayload(String);

#[derive(Clone)]
struct AxisOp(bool);

#[derive(Clone)]
struct SizesOp(Vec<Option<f32>>);

fn id_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(id)] => Ok(ComponentPayload::new(IdPayload(id.clone()))),
        _ => Err("Resizable(id) expects one string".into()),
    }
}

fn axis_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "axis",
        vec![ArgumentDescriptor::new("axis", ArgumentSchema::Number)],
        |args| {
            let vertical = args
                .first()
                .and_then(ComponentArgument::as_f64)
                .unwrap_or(0.0)
                >= 0.5;
            Ok(ComponentPayload::new(AxisOp(vertical)))
        },
    )
    .with_documentation("Sets the axis: 0 is a row, 1 is a column.")
}

fn sizes_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "sizes",
        vec![ArgumentDescriptor::new("sizes", ArgumentSchema::String)],
        |args| {
            let value = args
                .first()
                .and_then(ComponentArgument::as_str)
                .ok_or("Resizable.sizes expects a string")?;
            let sizes = value
                .split(',')
                .map(|part| part.trim().parse::<f32>().ok())
                .collect();
            Ok(ComponentPayload::new(SizesOp(sizes)))
        },
    )
    .with_documentation("Sets panel sizes in pixels; `*` is the flexible panel.")
}

struct ResizableMaterializer;

impl ComponentMaterializer for ResizableMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Resizable received an incompatible payload".to_string())?
            .0
            .clone();

        let mut vertical = false;
        let mut sizes: Vec<Option<f32>> = Vec::new();
        for method in request.methods() {
            match method.name() {
                "axis" => {
                    if let Some(op) = method.payload().downcast_ref::<AxisOp>() {
                        vertical = op.0;
                    }
                }
                "sizes" => {
                    if let Some(op) = method.payload().downcast_ref::<SizesOp>() {
                        sizes = op.0.clone();
                    }
                }
                _ => {}
            }
        }

        let name = SharedString::from(id);
        let mut group = if vertical {
            v_resizable(name)
        } else {
            h_resizable(name)
        };

        for (index, child) in request.take_children().into_iter().enumerate() {
            let mut panel = resizable_panel();
            if let Some(Some(size)) = sizes.get(index) {
                panel = panel.size(px(*size));
            }
            panel.extend([child]);
            group = group.child(panel);
        }

        request.finish(div().size_full().child(group))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Resizable", Arc::new(ResizableMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Resizable",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    id_payload,
                )])
                .with_methods(vec![axis_method(), sizes_method()])
                .with_documentation(
                    "Draggable panels in a row or column. Give the frame a width/height.",
                ),
        )
        .expect("the built-in Resizable descriptor is valid");
}
