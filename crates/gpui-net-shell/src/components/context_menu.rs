//! `ContextMenu`, wrapping an element and showing a menu on right-click,
//! built on `gpui_component`'s `ContextMenuExt`.
//!
//! Ordinary children are the target the menu attaches to; `ContextMenuItem` and
//! `ContextMenuSeparator` typed children are the menu's entries. Selection runs
//! a managed callback, matching the rest of the catalog (this runtime has no
//! action system).

use std::sync::Arc;

use gpui::{
    div, AnyElement, InteractiveElement as _, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _,
};
use gpui_component::menu::{ContextMenuExt as _, PopupMenuItem};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::{take_typed, Carrier};

#[derive(Clone)]
enum Entry {
    Item {
        label: String,
        disabled: bool,
        checked: bool,
        callback: Option<ComponentCallback>,
    },
    Separator,
}

#[derive(Clone)]
struct ItemPayload {
    label: String,
}

#[derive(Clone)]
enum ItemOp {
    Checked(bool),
    OnSelect(ComponentArgument),
}

#[derive(Clone, Copy)]
struct SeparatorPayload;

#[derive(Clone)]
struct ContextMenuPayload(String);

struct ItemMaterializer;

impl ComponentMaterializer for ItemMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let label = request
            .payload()
            .downcast_ref::<ItemPayload>()
            .ok_or_else(|| "ContextMenuItem received an incompatible payload".to_string())?
            .label
            .clone();
        if request.children_len() != 0 {
            return Err("ContextMenuItem does not accept children".to_string());
        }
        let disabled = request.disabled();
        let mut checked = false;
        let mut callback = None;
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ItemOp>().cloned())
            .collect::<Vec<_>>();
        for operation in operations {
            match operation {
                ItemOp::Checked(value) => checked = value,
                ItemOp::OnSelect(argument) => callback = Some(request.resolve_callback(&argument)?),
            }
        }
        Ok(Carrier::new(Entry::Item {
            label,
            disabled,
            checked,
            callback,
        })
        .into_any_element())
    }
}

struct SeparatorMaterializer;

impl ComponentMaterializer for SeparatorMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<SeparatorPayload>()
            .ok_or_else(|| "ContextMenuSeparator received an incompatible payload".to_string())?;
        if request.children_len() != 0 {
            return Err("ContextMenuSeparator does not accept children".to_string());
        }
        Ok(Carrier::new(Entry::Separator).into_any_element())
    }
}

struct ContextMenuMaterializer;

impl ComponentMaterializer for ContextMenuMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<ContextMenuPayload>()
            .ok_or_else(|| "ContextMenu received an incompatible payload".to_string())?
            .0
            .clone();
        let mut target = Vec::new();
        let mut entries = Vec::new();
        let mut request = request;
        for (name, mut element) in request.take_children_named() {
            match name {
                "ContextMenuItem" | "ContextMenuSeparator" => {
                    entries.push(take_typed::<Entry>(&mut element, name)?);
                }
                _ => target.push(element),
            }
        }
        let target = div()
            .id(SharedString::from(format!("shell-context-menu:{id}")))
            .children(target);
        let component = target.context_menu(move |mut menu, _window, _cx| {
            for entry in entries.clone() {
                menu = match entry {
                    Entry::Item {
                        label,
                        disabled,
                        checked,
                        callback,
                    } => {
                        let mut item = PopupMenuItem::new(label)
                            .disabled(disabled)
                            .checked(checked);
                        if let Some(callback) = callback {
                            item = item.on_click(move |_, window, cx| {
                                callback.invoke_with(
                                    "ContextMenuItem.on_select callback failed",
                                    &[],
                                    window,
                                    cx,
                                )
                            });
                        }
                        menu.item(item)
                    }
                    Entry::Separator => menu.separator(),
                };
            }
            menu
        });
        let mut component = component;
        component.style().refine(&request.take_style());
        Ok(component.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("ContextMenuItem", Arc::new(ItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "ContextMenuItem",
                    vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(label)] if !label.trim().is_empty() => {
                            Ok(ComponentPayload::new(ItemPayload {
                                label: label.clone(),
                            }))
                        }
                        _ => Err("ContextMenuItem expects a non-empty label".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "checked",
                        vec![ArgumentDescriptor::new("checked", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(ItemOp::Checked(*value)))
                            }
                            _ => Err("ContextMenuItem.checked expects a boolean".into()),
                        },
                    )
                    .with_documentation("Sets the menu item checked state."),
                    MethodDescriptor::new(
                        "on_select",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [argument @ ComponentArgument::Callback(_)] => {
                                Ok(ComponentPayload::new(ItemOp::OnSelect(argument.clone())))
                            }
                            _ => Err("ContextMenuItem.on_select expects a callback".into()),
                        },
                    )
                    .with_documentation("Runs when the context-menu item is selected."),
                ])
                .with_documentation("Typed context-menu item consumed by a ContextMenu."),
        )
        .expect("the built-in ContextMenuItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("ContextMenuSeparator", Arc::new(SeparatorMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "ContextMenuSeparator",
                    vec![],
                    |_| Ok(ComponentPayload::new(SeparatorPayload)),
                )])
                .with_methods(Vec::new())
                .with_documentation("Typed context-menu separator."),
        )
        .expect("the built-in ContextMenuSeparator descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("ContextMenu", Arc::new(ContextMenuMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "ContextMenu",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "ContextMenu")
                            .map(ContextMenuPayload)
                            .map(ComponentPayload::new),
                        _ => Err("ContextMenu(id) expects a non-empty string id".into()),
                    },
                )])
                .with_methods(Vec::new())
                .with_documentation(
                    "Attaches a right-click menu to its ordinary children, consuming \
                     ContextMenuItem and ContextMenuSeparator entries.",
                ),
        )
        .expect("the built-in ContextMenu descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_context_menu_family_registers_three_components() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        let names = frozen
            .descriptors()
            .map(|descriptor| descriptor.name())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            ["ContextMenuItem", "ContextMenuSeparator", "ContextMenu"]
        );
    }
}
