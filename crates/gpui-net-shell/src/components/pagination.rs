//! `Pagination`, ported from `component-shell`'s `compound/pagination.rs`.
//!
//! Controlled page navigation. `on_change` reports the page the reader asked
//! for as a number; disabled common behavior is supported.

use std::sync::Arc;

use gpui::{div, AnyElement, IntoElement as _, ParentElement as _, Refineable as _, Styled as _};
use gpui_component::{pagination::Pagination, Disableable as _, Sizable as _, Size};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallbackArgument,
    ComponentDescriptor, ComponentMaterializer, ComponentPayload, ComponentRegistry,
    ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct PaginationPayload(String);

#[derive(Clone)]
enum PaginationOp {
    OnChange(ComponentArgument),
    Current(usize),
    Total(usize),
    Visible(usize),
    Compact,
    Size(Size),
}

struct PaginationMaterializer;

impl ComponentMaterializer for PaginationMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("Pagination does not accept children".to_string());
        }
        let id = request
            .payload()
            .downcast_ref::<PaginationPayload>()
            .ok_or_else(|| "Pagination received an incompatible payload".to_string())?
            .0
            .clone();
        let mut change = None;
        for method in request.methods() {
            if let Some(PaginationOp::OnChange(argument)) =
                method.payload().downcast_ref::<PaginationOp>()
            {
                change = Some(argument.clone());
            }
        }
        let mut p = Pagination::new(id).disabled(request.disabled());
        for op in request
            .methods()
            .filter_map(|m| m.payload().downcast_ref::<PaginationOp>())
        {
            p = match op {
                PaginationOp::Current(value) => p.current_page(*value),
                PaginationOp::Total(value) => p.total_pages(*value),
                PaginationOp::Visible(value) => p.visible_pages(*value),
                PaginationOp::Compact => p.compact(),
                PaginationOp::Size(value) => p.with_size(*value),
                PaginationOp::OnChange(_) => p,
            }
        }
        if let Some(argument) = change {
            let callback = request.resolve_callback(&argument)?;
            p = p.on_click(move |page, window, cx| {
                callback.invoke_with(
                    "Pagination.on_change callback failed",
                    &[ComponentCallbackArgument::Number(*page as f64)],
                    window,
                    cx,
                );
            });
        }
        let mut wrapper = div().child(p);
        wrapper.style().refine(&request.take_style());
        Ok(wrapper.into_any_element())
    }
}

fn positive(a: &ComponentArgument, label: &str) -> Result<usize, String> {
    match a {
        ComponentArgument::Number(value) => {
            super::common::positive_usize(*value, &format!("Pagination.{label}({label})"))
        }
        _ => Err(format!(
            "Pagination.{label}({label}) expects a positive integer"
        )),
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    let numeric = |name: &'static str, doc: &'static str, wrap: fn(usize) -> PaginationOp| {
        MethodDescriptor::new(
            name,
            vec![ArgumentDescriptor::new(name, ArgumentSchema::Number)],
            move |arguments| match arguments {
                [value] => positive(value, name).map(|value| ComponentPayload::new(wrap(value))),
                _ => Err(format!("Pagination.{name}({name}) expects one argument")),
            },
        )
        .with_documentation(doc)
    };
    registry
        .register(
            ComponentDescriptor::new("Pagination", Arc::new(PaginationMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Pagination",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "Pagination")
                            .map(PaginationPayload)
                            .map(ComponentPayload::new),
                        _ => Err("Pagination(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    numeric(
                        "current_page",
                        "Sets the current 1-based page.",
                        PaginationOp::Current,
                    ),
                    numeric(
                        "total_pages",
                        "Sets the positive page count.",
                        PaginationOp::Total,
                    ),
                    numeric(
                        "visible_pages",
                        "Sets the maximum visible page buttons.",
                        PaginationOp::Visible,
                    ),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "on_change",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                PaginationOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("Pagination.on_change expects one callback".into()),
                        },
                    )
                    .with_documentation(
                        "Reports the page the reader asked for, so the script can drive \
                         `current_page`.",
                    ),
                    MethodDescriptor::new("compact", vec![], |_| {
                        Ok(ComponentPayload::new(PaginationOp::Compact))
                    })
                    .with_documentation("Shows only previous and next icon buttons."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => {
                                    Ok(ComponentPayload::new(PaginationOp::Size(Size::XSmall)))
                                }
                                "small" => {
                                    Ok(ComponentPayload::new(PaginationOp::Size(Size::Small)))
                                }
                                "medium" => {
                                    Ok(ComponentPayload::new(PaginationOp::Size(Size::Medium)))
                                }
                                "large" => {
                                    Ok(ComponentPayload::new(PaginationOp::Size(Size::Large)))
                                }
                                _ => Err(format!("unsupported Pagination size `{value}`")),
                            },
                            _ => Err("Pagination.size(size) expects a size literal".into()),
                        },
                    )
                    .with_documentation("Sets semantic size."),
                ])
                .with_documentation(
                    "Controlled page navigation; disabled common behavior is supported.",
                ),
        )
        .expect("the built-in Pagination descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_integer_validation() {
        assert_eq!(
            positive(&ComponentArgument::Number(3.), "total_pages").unwrap(),
            3
        );
        assert!(positive(&ComponentArgument::Number(0.), "total_pages").is_err());
        assert!(positive(&ComponentArgument::Number(1.5), "total_pages").is_err());
        assert!(positive(&ComponentArgument::Number(f64::INFINITY), "total_pages").is_err());
    }
}
