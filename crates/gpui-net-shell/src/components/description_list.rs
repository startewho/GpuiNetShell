//! `DescriptionList` and `DescriptionItem`, ported from `component-shell`'s
//! `structured/description_list.rs`.
//!
//! A structured label/value list accepting `DescriptionItem` children.
//! `DescriptionItem` carries its native value to the parent through
//! [`crate::typed_child::Carrier`] and is not an independently rendered element.

use std::sync::Arc;

use gpui::{div, AnyElement, IntoElement as _, ParentElement as _, Refineable as _, Styled as _};
use gpui_component::{
    description_list::{DescriptionItem, DescriptionList},
    Sizable as _, Size,
};

use super::common::positive_usize;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone)]
struct ItemPayload(String);

#[derive(Clone)]
enum ItemOp {
    Value(String),
    Span(usize),
}

struct ItemMaterializer;

impl ComponentMaterializer for ItemMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("DescriptionItem does not accept children; use value(string)".to_string());
        }
        let label = request
            .payload()
            .downcast_ref::<ItemPayload>()
            .ok_or_else(|| "DescriptionItem received an incompatible payload".to_string())?
            .0
            .clone();
        let mut item = DescriptionItem::new(label);
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ItemOp>())
        {
            item = match operation {
                ItemOp::Value(value) => item.value(value.clone()),
                ItemOp::Span(span) => item.span(*span),
            };
        }
        Ok(Carrier::new(item).into_any_element())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ListConfig {
    vertical: bool,
    bordered: bool,
    columns: usize,
    size: Size,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            vertical: false,
            bordered: true,
            columns: 3,
            size: Size::Medium,
        }
    }
}

#[derive(Clone, Copy)]
enum ListOp {
    Vertical,
    Bordered(bool),
    Columns(usize),
    Size(Size),
}

struct ListMaterializer;

impl ComponentMaterializer for ListMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<()>()
            .ok_or_else(|| "DescriptionList received an incompatible payload".to_string())?;
        let mut config = ListConfig::default();
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ListOp>())
        {
            match op {
                ListOp::Vertical => config.vertical = true,
                ListOp::Bordered(value) => config.bordered = *value,
                ListOp::Columns(value) => config.columns = *value,
                ListOp::Size(value) => config.size = *value,
            }
        }
        let items = request.take_typed_children::<DescriptionItem>(&["DescriptionItem"])?;
        let mut list = if config.vertical {
            DescriptionList::vertical()
        } else {
            DescriptionList::horizontal()
        }
        .bordered(config.bordered)
        .columns(config.columns)
        .with_size(config.size);
        for item in items {
            list = list.child(item);
        }
        let mut wrapper = div().child(list);
        wrapper.style().refine(&request.take_style());
        Ok(wrapper.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("DescriptionItem", Arc::new(ItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "DescriptionItem",
                    vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(value)] => {
                            Ok(ComponentPayload::new(ItemPayload(value.clone())))
                        }
                        _ => Err("DescriptionItem(label) expects a string".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "value",
                        vec![ArgumentDescriptor::new("value", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                Ok(ComponentPayload::new(ItemOp::Value(value.clone())))
                            }
                            _ => Err("DescriptionItem.value(value) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the item's textual value."),
                    MethodDescriptor::new(
                        "span",
                        vec![ArgumentDescriptor::new("span", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)] => {
                                positive_usize(*value, "DescriptionItem.span")
                                    .map(|value| ComponentPayload::new(ItemOp::Span(value)))
                            }
                            _ => Err("DescriptionItem.span expects a positive integer".into()),
                        },
                    )
                    .with_documentation("Sets how many description-list columns the item spans."),
                ])
                .with_documentation(
                    "A typed label/value child for DescriptionList. It accepts value(string) and \
                     span(number), but no children or common style methods.",
                ),
        )
        .expect("the built-in DescriptionItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("DescriptionList", Arc::new(ListMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new("DescriptionList", vec![], |_| {
                    Ok(ComponentPayload::new(()))
                })])
                .with_methods(vec![
                    MethodDescriptor::new("vertical", vec![], |_| {
                        Ok(ComponentPayload::new(ListOp::Vertical))
                    })
                    .with_documentation("Uses the vertical label/value layout."),
                    MethodDescriptor::new(
                        "bordered",
                        vec![ArgumentDescriptor::new("bordered", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(ListOp::Bordered(*value)))
                            }
                            _ => Err("DescriptionList.bordered expects a boolean".into()),
                        },
                    )
                    .with_documentation("Controls the horizontal-layout border."),
                    MethodDescriptor::new(
                        "columns",
                        vec![ArgumentDescriptor::new("columns", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)] => {
                                let columns =
                                    positive_usize(*value, "DescriptionList.columns")?;
                                if columns > 10 {
                                    return Err(
                                        "DescriptionList.columns expects an integer from 1 through 10"
                                            .into(),
                                    );
                                }
                                Ok(ComponentPayload::new(ListOp::Columns(columns)))
                            }
                            _ => Err(
                                "DescriptionList.columns expects an integer from 1 through 10".into(),
                            ),
                        },
                    )
                    .with_documentation("Sets the column count from 1 through 10."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => {
                                    Ok(ComponentPayload::new(ListOp::Size(Size::XSmall)))
                                }
                                "small" => Ok(ComponentPayload::new(ListOp::Size(Size::Small))),
                                "medium" => Ok(ComponentPayload::new(ListOp::Size(Size::Medium))),
                                "large" => Ok(ComponentPayload::new(ListOp::Size(Size::Large))),
                                _ => Err(format!("unsupported DescriptionList size `{value}`")),
                            },
                            _ => Err("DescriptionList.size expects a size literal".into()),
                        },
                    )
                    .with_documentation("Sets the description-list density."),
                ])
                .with_documentation(
                    "A structured label/value list accepting DescriptionItem children.",
                ),
        )
        .expect("the built-in DescriptionList descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_rejects_values_the_component_would_clamp() {
        assert!(positive_usize(10.0, "DescriptionList.columns").is_ok());
        assert!(positive_usize(0.0, "DescriptionList.columns").is_err());
    }
}
