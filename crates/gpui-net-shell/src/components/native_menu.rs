//! The `NativeMenu` family, adapted from `component-shell`'s `command/native_menu.rs`.
//!
//! `component-shell` dispatches a named shell action when a native menu item is
//! selected; this runtime has no action system, so each item carries a managed
//! callback token and selection is delivered through a global
//! [`crate::menu_action::ManagedMenuAction`] listener.
//!
//! `NativeMenuItem` and `NativeMenuSeparator` are typed children of a real
//! `NativeMenuTrigger` button; they render nothing themselves and are consumed
//! by the trigger.

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, Refineable as _, Styled as _};
use gpui_component::button::Button;
use gpui_component::native_menu::NativeMenu;
use gpui_component::Disableable as _;

use crate::menu_action::ManagedMenuAction;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
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
        token: u64,
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
struct TriggerPayload {
    id: String,
    label: String,
}

struct ItemMaterializer;

impl ComponentMaterializer for ItemMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let label = request
            .payload()
            .downcast_ref::<ItemPayload>()
            .ok_or_else(|| "NativeMenuItem received an incompatible payload".to_string())?
            .label
            .clone();
        if request.children_len() != 0 {
            return Err("NativeMenuItem does not accept children".to_string());
        }
        let disabled = request.disabled();
        let mut checked = false;
        let mut token = 0;
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ItemOp>().cloned())
            .collect::<Vec<_>>();
        for operation in &operations {
            match operation {
                ItemOp::Checked(value) => checked = *value,
                ItemOp::OnSelect(argument) => {
                    token = request.resolve_callback(argument)?.token();
                }
            }
        }
        let _ = request.take_style();
        Ok(Carrier::new(Entry::Item {
            label,
            disabled,
            checked,
            token,
        })
        .into_any_element())
    }
}

struct SeparatorMaterializer;

impl ComponentMaterializer for SeparatorMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("NativeMenuSeparator does not accept children".to_string());
        }
        let _ = request.take_style();
        Ok(Carrier::new(Entry::Separator).into_any_element())
    }
}

struct TriggerMaterializer;

impl ComponentMaterializer for TriggerMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<TriggerPayload>()
            .ok_or_else(|| "NativeMenuTrigger received an incompatible payload".to_string())?
            .clone();
        let disabled = request.disabled();
        let mut entries = Vec::new();
        let mut request = request;
        for (name, mut element) in request.take_children_named() {
            match name {
                "NativeMenuItem" | "NativeMenuSeparator" => {
                    entries.push(take_typed::<Entry>(&mut element, name)?);
                }
                other => {
                    return Err(format!(
                        "NativeMenuTrigger accepts only NativeMenuItem or \
                         NativeMenuSeparator children; received {other}"
                    ))
                }
            }
        }
        let mut button = Button::new(payload.id)
            .label(payload.label)
            .disabled(disabled)
            .on_click(move |event, window, cx| {
                let mut menu = NativeMenu::new();
                for entry in entries.clone() {
                    menu = match entry {
                        Entry::Item {
                            label,
                            disabled: true,
                            token,
                            ..
                        } => menu.menu_with_disabled(
                            label,
                            true,
                            Box::new(ManagedMenuAction::new(token)),
                        ),
                        Entry::Item {
                            label,
                            checked: true,
                            token,
                            ..
                        } => menu.menu_with_check(
                            label,
                            true,
                            Box::new(ManagedMenuAction::new(token)),
                        ),
                        Entry::Item { label, token, .. } => {
                            menu.menu(label, Box::new(ManagedMenuAction::new(token)))
                        }
                        Entry::Separator => menu.separator(),
                    };
                }
                menu.show(event.position(), window, cx);
            });
        button.style().refine(&request.take_style());
        Ok(button.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("NativeMenuItem", Arc::new(ItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "NativeMenuItem",
                    vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(label)] if !label.trim().is_empty() => {
                            Ok(ComponentPayload::new(ItemPayload {
                                label: label.clone(),
                            }))
                        }
                        _ => Err("NativeMenuItem expects a non-empty label".into()),
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
                            _ => Err("NativeMenuItem.checked expects a boolean".into()),
                        },
                    )
                    .with_documentation("Sets the native menu item checked state."),
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
                            _ => Err("NativeMenuItem.on_select expects a callback".into()),
                        },
                    )
                    .with_documentation("Runs when the native menu item is selected."),
                ])
                .with_documentation("Typed native-menu item data with last-call-wins checked."),
        )
        .expect("the built-in NativeMenuItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("NativeMenuSeparator", Arc::new(SeparatorMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "NativeMenuSeparator",
                    vec![],
                    |_| Ok(ComponentPayload::new(SeparatorPayload)),
                )])
                .with_methods(Vec::new())
                .with_documentation("Typed native-menu separator data."),
        )
        .expect("the built-in NativeMenuSeparator descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("NativeMenuTrigger", Arc::new(TriggerMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "NativeMenuTrigger",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("label", ArgumentSchema::String),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(label)]
                            if !id.trim().is_empty() && !label.trim().is_empty() =>
                        {
                            Ok(ComponentPayload::new(TriggerPayload {
                                id: id.clone(),
                                label: label.clone(),
                            }))
                        }
                        _ => Err("NativeMenuTrigger expects a non-empty id and label".into()),
                    },
                )])
                .with_methods(vec![MethodDescriptor::new(
                    "on_effect_error",
                    vec![ArgumentDescriptor::new(
                        "callback",
                        ArgumentSchema::Callback,
                    )],
                    |arguments| match arguments {
                        [argument @ ComponentArgument::Callback(_)] => {
                            let _ = argument;
                            Ok(ComponentPayload::new(()))
                        }
                        _ => Err("NativeMenuTrigger.on_effect_error expects a callback".into()),
                    },
                )
                .with_documentation(
                    "Accepted for parity; native menu display is synchronous.",
                )])
                .with_documentation(
                    "A real button that shows an OS native menu. Typed NativeMenuItem children \
                     report selection through their managed callbacks.",
                ),
        )
        .expect("the built-in NativeMenuTrigger descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_native_menu_family_registers_three_components() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        let names = frozen
            .descriptors()
            .map(|descriptor| descriptor.name())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            ["NativeMenuItem", "NativeMenuSeparator", "NativeMenuTrigger"]
        );
    }
}
