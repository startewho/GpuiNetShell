//! `Tag`, ported from `component-shell`'s `controls/display.rs`.
//!
//! A compact semantic status tag that renders ordinary children. Variant,
//! outline, rounding, and size are recorded methods.

use std::sync::Arc;

use gpui::AnyElement;
use gpui_component::{
    tag::{Tag, TagVariant},
    Sizable as _, Size,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone, Copy)]
struct UnitPayload;

#[derive(Clone)]
enum TagOp {
    Size(Size),
    Variant(TagVariant),
    Outline,
    RoundedFull,
}

fn nullary(export: &'static str) -> ConstructorDescriptor {
    ConstructorDescriptor::new(export, Vec::new(), |_| {
        Ok(ComponentPayload::new(UnitPayload))
    })
}

struct TagMaterializer;

impl ComponentMaterializer for TagMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<UnitPayload>()
            .ok_or_else(|| "Tag received an incompatible payload".to_string())?;
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<TagOp>().cloned())
            .collect::<Vec<_>>();
        let mut component = Tag::new();
        for operation in operations {
            component = match operation {
                TagOp::Size(size) => component.with_size(size),
                TagOp::Variant(variant) => component.with_variant(variant),
                TagOp::Outline => component.outline(),
                TagOp::RoundedFull => component.rounded_full(),
            };
        }
        request.finish(component)
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Tag", Arc::new(TagMaterializer))
                .with_constructors(vec![nullary("Tag")])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "variant",
                        vec![ArgumentDescriptor::new(
                            "variant",
                            ArgumentSchema::Enum(&[
                                "primary",
                                "secondary",
                                "danger",
                                "success",
                                "warning",
                                "info",
                            ]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "primary" => {
                                    Ok(ComponentPayload::new(TagOp::Variant(TagVariant::Primary)))
                                }
                                "secondary" => {
                                    Ok(ComponentPayload::new(TagOp::Variant(TagVariant::Secondary)))
                                }
                                "danger" => {
                                    Ok(ComponentPayload::new(TagOp::Variant(TagVariant::Danger)))
                                }
                                "success" => {
                                    Ok(ComponentPayload::new(TagOp::Variant(TagVariant::Success)))
                                }
                                "warning" => {
                                    Ok(ComponentPayload::new(TagOp::Variant(TagVariant::Warning)))
                                }
                                "info" => {
                                    Ok(ComponentPayload::new(TagOp::Variant(TagVariant::Info)))
                                }
                                _ => Err(format!("unsupported Tag variant `{value}`")),
                            },
                            _ => Err("Tag.variant expects a supported variant".into()),
                        },
                    )
                    .with_documentation("Sets the semantic tag variant."),
                    MethodDescriptor::new("outline", Vec::new(), |_| {
                        Ok(ComponentPayload::new(TagOp::Outline))
                    })
                    .with_documentation("Uses the outline presentation."),
                    MethodDescriptor::new("rounded_full", Vec::new(), |_| {
                        Ok(ComponentPayload::new(TagOp::RoundedFull))
                    })
                    .with_documentation("Uses pill-shaped corners."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |args| match args {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => Ok(ComponentPayload::new(TagOp::Size(Size::XSmall))),
                                "small" => Ok(ComponentPayload::new(TagOp::Size(Size::Small))),
                                "medium" => Ok(ComponentPayload::new(TagOp::Size(Size::Medium))),
                                "large" => Ok(ComponentPayload::new(TagOp::Size(Size::Large))),
                                _ => Err(format!("unsupported Tag size `{value}`")),
                            },
                            _ => Err("Tag.size expects a semantic size literal".into()),
                        },
                    )
                    .with_documentation("Sets the semantic component size."),
                ])
                .with_documentation(
                    "A compact semantic status tag that renders ordinary children.",
                ),
        )
        .expect("the built-in Tag descriptor is valid");
}
