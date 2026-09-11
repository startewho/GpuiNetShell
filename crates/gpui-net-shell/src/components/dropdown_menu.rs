//! `DropdownMenu`, ported from `component-shell`'s `overlays/dropdown_menu.rs`.
//!
//! A button-triggered native popup menu. Items are declared with
//! `item(label, callback)`, a two-argument method (P3 wire): the label as a
//! string and the activation as a callback.

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, Refineable as _, Styled as _};
use gpui_component::{
    button::Button,
    menu::{DropdownMenu as _, PopupMenuItem},
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct DropdownMenuPayload {
    id: String,
    label: String,
}

#[derive(Clone)]
struct MenuItemOp {
    label: String,
    callback: ComponentArgument,
}

struct DropdownMenuMaterializer;

impl ComponentMaterializer for DropdownMenuMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err(
                "DropdownMenu accepts item(label, callback) methods only; ordinary children are unsupported"
                    .to_string(),
            );
        }
        let payload = request
            .payload()
            .downcast_ref::<DropdownMenuPayload>()
            .ok_or_else(|| "DropdownMenu received an incompatible payload".to_string())?
            .clone();
        let items = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<MenuItemOp>().cloned())
            .map(|item| Ok((item.label, request.resolve_callback(&item.callback)?)))
            .collect::<Result<Vec<_>, String>>()?;

        let mut button = Button::new(payload.id).label(payload.label);
        button.style().refine(&request.take_style());
        let menu = button.dropdown_menu(move |menu, _, _| {
            items.iter().fold(menu, |menu, (label, callback)| {
                let callback = callback.clone();
                menu.item(
                    PopupMenuItem::new(label.clone()).on_click(move |_, window, cx| {
                        callback.invoke_with("DropdownMenu.item callback failed", &[], window, cx);
                    }),
                )
            })
        });
        Ok(menu.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("DropdownMenu", Arc::new(DropdownMenuMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "DropdownMenu",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("label", ArgumentSchema::String),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(label)]
                            if !id.trim().is_empty() && !label.trim().is_empty() =>
                        {
                            Ok(ComponentPayload::new(DropdownMenuPayload {
                                id: id.clone(),
                                label: label.clone(),
                            }))
                        }
                        [ComponentArgument::String(_), ComponentArgument::String(_)] => {
                            Err("DropdownMenu id and label must not be empty".into())
                        }
                        _ => Err("DropdownMenu(id, label) expects two strings".into()),
                    },
                )])
                .with_methods(vec![MethodDescriptor::new(
                    "item",
                    vec![
                        ArgumentDescriptor::new("label", ArgumentSchema::String),
                        ArgumentDescriptor::new("callback", ArgumentSchema::Callback),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(label), callback @ ComponentArgument::Callback(_)]
                            if !label.trim().is_empty() =>
                        {
                            Ok(ComponentPayload::new(MenuItemOp {
                                label: label.clone(),
                                callback: callback.clone(),
                            }))
                        }
                        [ComponentArgument::String(_), ComponentArgument::Callback(_)] => {
                            Err("DropdownMenu.item label must not be empty".into())
                        }
                        _ => Err(
                            "DropdownMenu.item(label, callback) expects a string and callback"
                                .into(),
                        ),
                    },
                )
                .with_documentation("Appends a command item in call order.")])
                .with_documentation(
                    "A button-triggered native popup menu containing closed command items.",
                ),
        )
        .expect("the built-in DropdownMenu descriptor is valid");
}
