//! `DropdownButton`, ported from `component-shell`'s `basic/dropdown_button.rs`.
//!
//! A real split dropdown button with a labeled action half and optional
//! callback menu items. It accepts no children; `disabled`, `selected`, and
//! `on_click` are shell behavior.

use std::sync::Arc;

use gpui::{Anchor, AnyElement, IntoElement as _, Refineable as _, Styled as _};
use gpui_component::{
    button::{Button, ButtonVariants as _, DropdownButton},
    menu::PopupMenuItem,
    Disableable as _, Selectable as _, Sizable as _, Size,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct DropdownPayload {
    id: String,
    label: String,
}

#[derive(Clone)]
enum DropdownOp {
    Outline,
    Size(Size),
    Variant(Variant),
    Anchor(Anchor),
    Item {
        label: String,
        callback: ComponentArgument,
    },
}

#[derive(Clone, Copy)]
enum Variant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

struct DropdownMaterializer;

impl ComponentMaterializer for DropdownMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<DropdownPayload>()
            .ok_or_else(|| "DropdownButton received an incompatible payload".to_string())?
            .clone();
        if request.children_len() != 0 {
            return Err("DropdownButton does not accept children".to_string());
        }

        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<DropdownOp>().cloned())
            .collect::<Vec<_>>();
        let mut action = Button::new("action").label(payload.label);
        if let Some(token) = request.on_click() {
            let host = request.host().clone();
            action = action.on_click(move |_event, _window, cx| {
                if let Some(click) = host.callbacks.click {
                    // SAFETY: managed callback; it copies anything it keeps.
                    unsafe {
                        let _ = click(host.session_id, token);
                    }
                }
                (host.invalidate)(cx);
            });
        }
        let mut dropdown = DropdownButton::new(payload.id)
            .button(action)
            .disabled(request.disabled())
            .selected(request.selected());
        let mut anchor = Anchor::TopRight;
        let mut items: Vec<(String, ComponentCallback)> = Vec::new();
        for operation in operations {
            dropdown = match operation {
                DropdownOp::Outline => dropdown.outline(),
                DropdownOp::Size(value) => dropdown.with_size(value),
                DropdownOp::Variant(Variant::Primary) => dropdown.primary(),
                DropdownOp::Variant(Variant::Secondary) => dropdown.secondary(),
                DropdownOp::Variant(Variant::Danger) => dropdown.danger(),
                DropdownOp::Variant(Variant::Ghost) => dropdown.ghost(),
                DropdownOp::Anchor(value) => {
                    anchor = value;
                    dropdown
                }
                DropdownOp::Item { label, callback } => {
                    items.push((label, request.resolve_callback(&callback)?));
                    dropdown
                }
            };
        }
        if !items.is_empty() {
            dropdown = dropdown.dropdown_menu_with_anchor(anchor, move |mut menu, _, _| {
                for (label, callback) in &items {
                    let callback = callback.clone();
                    menu = menu.item(PopupMenuItem::new(label.clone()).on_click(
                        move |_, window, cx| {
                            callback.invoke_with(
                                "DropdownButton.menu_item callback failed",
                                &[],
                                window,
                                cx,
                            );
                        },
                    ));
                }
                menu
            });
        }
        dropdown.style().refine(&request.take_style());
        Ok(dropdown.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("DropdownButton", Arc::new(DropdownMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "DropdownButton",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("label", ArgumentSchema::String),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(label)]
                            if !id.trim().is_empty() =>
                        {
                            Ok(ComponentPayload::new(DropdownPayload {
                                id: id.clone(),
                                label: label.clone(),
                            }))
                        }
                        [ComponentArgument::String(_), ComponentArgument::String(_)] => {
                            Err("DropdownButton id must not be empty".into())
                        }
                        _ => Err("DropdownButton(id, label) expects two strings".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new("outline", vec![], |_| {
                        Ok(ComponentPayload::new(DropdownOp::Outline))
                    })
                    .with_documentation("Uses the outlined button treatment."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => Ok(ComponentPayload::new(DropdownOp::Size(Size::XSmall))),
                                "small" => Ok(ComponentPayload::new(DropdownOp::Size(Size::Small))),
                                "medium" => {
                                    Ok(ComponentPayload::new(DropdownOp::Size(Size::Medium)))
                                }
                                "large" => Ok(ComponentPayload::new(DropdownOp::Size(Size::Large))),
                                _ => Err(format!("unsupported DropdownButton size `{value}`")),
                            },
                            _ => Err("DropdownButton.size(size) expects a size literal".into()),
                        },
                    )
                    .with_documentation("Sets the size of both halves."),
                    MethodDescriptor::new(
                        "variant",
                        vec![ArgumentDescriptor::new(
                            "variant",
                            ArgumentSchema::Enum(&["primary", "secondary", "danger", "ghost"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "primary" => {
                                    Ok(ComponentPayload::new(DropdownOp::Variant(Variant::Primary)))
                                }
                                "secondary" => Ok(ComponentPayload::new(DropdownOp::Variant(
                                    Variant::Secondary,
                                ))),
                                "danger" => {
                                    Ok(ComponentPayload::new(DropdownOp::Variant(Variant::Danger)))
                                }
                                "ghost" => {
                                    Ok(ComponentPayload::new(DropdownOp::Variant(Variant::Ghost)))
                                }
                                _ => Err(format!(
                                    "unsupported DropdownButton variant `{value}`"
                                )),
                            },
                            _ => Err(
                                "DropdownButton.variant(variant) expects a variant literal".into(),
                            ),
                        },
                    )
                    .with_documentation("Sets the semantic variant of both halves."),
                    MethodDescriptor::new(
                        "menu_anchor",
                        vec![ArgumentDescriptor::new(
                            "anchor",
                            ArgumentSchema::Enum(&[
                                "top_right",
                                "bottom_right",
                                "bottom_left",
                                "top_left",
                            ]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "top_right" => {
                                    Ok(ComponentPayload::new(DropdownOp::Anchor(Anchor::TopRight)))
                                }
                                "bottom_right" => Ok(ComponentPayload::new(DropdownOp::Anchor(
                                    Anchor::BottomRight,
                                ))),
                                "bottom_left" => Ok(ComponentPayload::new(DropdownOp::Anchor(
                                    Anchor::BottomLeft,
                                ))),
                                "top_left" => {
                                    Ok(ComponentPayload::new(DropdownOp::Anchor(Anchor::TopLeft)))
                                }
                                _ => Err(format!(
                                    "unsupported DropdownButton anchor `{value}`"
                                )),
                            },
                            _ => Err(
                                "DropdownButton.menu_anchor(anchor) expects an anchor literal"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Sets the popup menu anchor."),
                    MethodDescriptor::new(
                        "menu_item",
                        vec![
                            ArgumentDescriptor::new("label", ArgumentSchema::String),
                            ArgumentDescriptor::new("callback", ArgumentSchema::Callback),
                        ],
                        |arguments| match arguments {
                            [
                                ComponentArgument::String(label),
                                callback @ ComponentArgument::Callback(_),
                            ] => Ok(ComponentPayload::new(DropdownOp::Item {
                                label: label.clone(),
                                callback: callback.clone(),
                            })),
                            _ => Err(
                                "DropdownButton.menu_item(label, callback) expects a string and callback"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Appends a clickable popup-menu item in call order."),
                ])
                .with_documentation(
                    "A real split DropdownButton with a labeled action half and optional callback \
                     menu items. It accepts no children.",
                ),
        )
        .expect("the built-in DropdownButton descriptor is valid");
}
