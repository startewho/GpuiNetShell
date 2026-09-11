//! `Rating`, ported from `component-shell`'s `display/rating.rs`.
//!
//! An interactive star rating. `on_change` reports the clicked star as a number;
//! value, max, color, and size are recorded methods.

use std::sync::Arc;

use gpui::{AnyElement, Hsla, IntoElement as _, Refineable as _, Styled as _};
use gpui_component::{rating::Rating, try_parse_color, Size};

use super::common::{nonempty_id, nonnegative_usize};
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallbackArgument,
    ComponentDescriptor, ComponentMaterializer, ComponentPayload, ComponentRegistry,
    ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct RatingPayload {
    id: String,
}

#[derive(Clone)]
enum RatingOp {
    Value(usize),
    Max(usize),
    Color(Hsla),
    Size(Size),
    OnChange(ComponentArgument),
}

struct RatingMaterializer;

impl RatingMaterializer {
    fn component<'a>(
        payload: &ComponentPayload,
        operations: impl IntoIterator<Item = &'a RatingOp>,
        disabled: bool,
    ) -> Result<Rating, String> {
        let payload = payload
            .downcast_ref::<RatingPayload>()
            .ok_or_else(|| "Rating received an incompatible payload".to_string())?;
        let operations = operations.into_iter().collect::<Vec<_>>();
        let (value, max) = rating_settings(&operations);
        Ok(operations.into_iter().fold(
            Rating::new(payload.id.clone())
                .disabled(disabled)
                .max(max)
                .value(value),
            |component, operation| match operation {
                RatingOp::Value(_) | RatingOp::Max(_) | RatingOp::OnChange(_) => component,
                RatingOp::Color(color) => component.color(*color),
                RatingOp::Size(size) => component.with_size(*size),
            },
        ))
    }
}

fn rating_settings(operations: &[&RatingOp]) -> (usize, usize) {
    let (mut value, mut max) = (0, 5);
    for operation in operations {
        match operation {
            RatingOp::Value(next) => value = (*next).min(max),
            RatingOp::Max(next) => {
                max = *next;
                value = value.min(max);
            }
            RatingOp::Color(_) | RatingOp::Size(_) | RatingOp::OnChange(_) => {}
        }
    }
    (value, max)
}

impl ComponentMaterializer for RatingMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("Rating does not accept children".to_string());
        }
        let mut change = None;
        for method in request.methods() {
            if let Some(RatingOp::OnChange(argument)) = method.payload().downcast_ref::<RatingOp>()
            {
                change = Some(argument.clone());
            }
        }
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<RatingOp>());
        let mut component = Self::component(request.payload(), operations, request.disabled())?;
        if let Some(argument) = change {
            let callback = request.resolve_callback(&argument)?;
            component = component.on_click(move |value, window, cx| {
                callback.invoke_with(
                    "Rating.on_change callback failed",
                    &[ComponentCallbackArgument::Number(*value as f64)],
                    window,
                    cx,
                );
            });
        }
        component.style().refine(&request.take_style());
        Ok(component.into_any_element())
    }
}

fn count_method(
    name: &'static str,
    documentation: &'static str,
    wrap: fn(usize) -> RatingOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Number)],
        move |arguments| match arguments {
            [ComponentArgument::Number(value)] => {
                rating_count(*value, name).map(|value| ComponentPayload::new(wrap(value)))
            }
            _ => Err(format!("Rating.{name}({name}) expects a number")),
        },
    )
    .with_documentation(documentation)
}

fn rating_count(value: f64, name: &str) -> Result<usize, String> {
    nonnegative_usize(value, &format!("Rating.{name}({name})"))
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Rating", Arc::new(RatingMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Rating",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => {
                            Ok(ComponentPayload::new(RatingPayload {
                                id: nonempty_id(id, "Rating")?,
                            }))
                        }
                        _ => Err("Rating(id) expects a string".into()),
                    },
                )])
                .with_methods(vec![
                    count_method(
                        "value",
                        "Sets the current number of active stars.",
                        RatingOp::Value,
                    ),
                    count_method("max", "Sets the maximum number of stars.", RatingOp::Max),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "on_change",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                RatingOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("Rating.on_change expects one callback".into()),
                        },
                    )
                    .with_documentation(
                        "Reports the star the reader clicked, so the script can drive `value`.",
                    ),
                    MethodDescriptor::new(
                        "color",
                        vec![ArgumentDescriptor::new("color", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(color)] => try_parse_color(color)
                                .map(|color| ComponentPayload::new(RatingOp::Color(color)))
                                .map_err(|error| format!("invalid Rating color: {error}")),
                            _ => Err("Rating.color(color) expects a color string".into()),
                        },
                    )
                    .with_documentation("Sets the active star color."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => Ok(ComponentPayload::new(RatingOp::Size(Size::XSmall))),
                                "small" => Ok(ComponentPayload::new(RatingOp::Size(Size::Small))),
                                "medium" => Ok(ComponentPayload::new(RatingOp::Size(Size::Medium))),
                                "large" => Ok(ComponentPayload::new(RatingOp::Size(Size::Large))),
                                _ => Err(format!("unsupported Rating size `{value}`")),
                            },
                            _ => Err("Rating.size expects a semantic size literal".into()),
                        },
                    )
                    .with_documentation("Sets the rating's semantic size."),
                ])
                .with_documentation(
                    "An interactive star rating with configurable value and maximum.",
                ),
        )
        .expect("the built-in Rating descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn later_value_and_max_operations_win_in_call_order() {
        assert_eq!(
            rating_settings(&[&RatingOp::Value(4), &RatingOp::Max(3), &RatingOp::Value(2)]),
            (2, 3)
        );
    }

    #[test]
    fn rejects_the_rounded_usize_overflow_boundary() {
        assert!(rating_count(usize::MAX as f64, "max").is_err());
    }
}
