//! `Badge`, ported from `component-shell`'s `controls/display.rs`.
//!
//! A count or dot badge positioned over its ordinary children. The badge is
//! configured entirely through methods; the constructor takes no arguments.

use std::sync::Arc;

use gpui::{div, AnyElement, Hsla, ParentElement as _};
use gpui_component::{badge::Badge, try_parse_color, Sizable as _, Size};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone, Copy)]
struct UnitPayload;

#[derive(Clone)]
enum BadgeOp {
    Size(Size),
    Dot,
    Count(usize),
    Max(usize),
    Color(Hsla),
}

fn nullary(export: &'static str) -> ConstructorDescriptor {
    ConstructorDescriptor::new(export, Vec::new(), |_| {
        Ok(ComponentPayload::new(UnitPayload))
    })
}

fn natural_number(name: &'static str, make: fn(usize) -> BadgeOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Number)],
        move |arguments| match arguments {
            [ComponentArgument::Number(value)] => parse_natural(*value)
                .map(make)
                .map(ComponentPayload::new)
                .ok_or_else(|| format!("Badge.{name} expects a non-negative integer")),
            _ => Err(format!("Badge.{name} expects a non-negative integer")),
        },
    )
    .with_documentation(if name == "count" {
        "Sets the displayed count; zero hides a numeric badge."
    } else {
        "Sets the largest count displayed before the plus suffix."
    })
}

fn parse_natural(value: f64) -> Option<usize> {
    (value.is_finite() && value >= 0.0 && value.fract() == 0.0 && value < usize::MAX as f64)
        .then_some(value as usize)
}

struct BadgeMaterializer;

impl ComponentMaterializer for BadgeMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<UnitPayload>()
            .ok_or_else(|| "Badge received an incompatible payload".to_string())?;
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<BadgeOp>().cloned())
            .collect::<Vec<_>>();
        let mut component = Badge::new();
        for operation in operations {
            component = match operation {
                BadgeOp::Size(size) => component.with_size(size),
                BadgeOp::Dot => component.dot(),
                BadgeOp::Count(count) => component.count(count),
                BadgeOp::Max(max) => component.max(max),
                BadgeOp::Color(color) => component.color(color),
            };
        }
        request.finish(div().child(component))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Badge", Arc::new(BadgeMaterializer))
                .with_constructors(vec![nullary("Badge")])
                .with_methods(vec![
                    MethodDescriptor::new("dot", Vec::new(), |_| {
                        Ok(ComponentPayload::new(BadgeOp::Dot))
                    })
                    .with_documentation("Displays a dot instead of a numeric count."),
                    natural_number("count", BadgeOp::Count),
                    natural_number("max", BadgeOp::Max),
                    MethodDescriptor::new(
                        "color",
                        vec![ArgumentDescriptor::new("color", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => try_parse_color(value)
                                .map(BadgeOp::Color)
                                .map(ComponentPayload::new)
                                .map_err(|error| format!("invalid Badge color: {error}")),
                            _ => Err("Badge.color expects a color string".into()),
                        },
                    )
                    .with_documentation("Sets the badge background from a supported color token."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |args| match args {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => Ok(ComponentPayload::new(BadgeOp::Size(Size::XSmall))),
                                "small" => Ok(ComponentPayload::new(BadgeOp::Size(Size::Small))),
                                "medium" => Ok(ComponentPayload::new(BadgeOp::Size(Size::Medium))),
                                "large" => Ok(ComponentPayload::new(BadgeOp::Size(Size::Large))),
                                _ => Err(format!("unsupported Badge size `{value}`")),
                            },
                            _ => Err("Badge.size expects a semantic size literal".into()),
                        },
                    )
                    .with_documentation("Sets the semantic component size."),
                ])
                .with_documentation("A count or dot badge positioned over its ordinary children."),
        )
        .expect("the built-in Badge descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_numbers_reject_fractional_negative_and_overflow_values() {
        assert_eq!(parse_natural(7.0), Some(7));
        assert_eq!(parse_natural(1.5), None);
        assert_eq!(parse_natural(-1.0), None);
        assert_eq!(parse_natural(usize::MAX as f64), None);
        assert_eq!(parse_natural(f64::INFINITY), None);
    }
}
