//! `Menu`/`MenuBar`/`MenuItem`/`MenuSeparator`, adapted from
//! `component-shell`'s `lifecycle/menu.rs`.
//!
//! `component-shell` drives the OS application menu bar through shell actions,
//! which this runtime does not have. This port renders an **in-window** menu bar
//! instead: `MenuBar` is a row of `Menu` dropdown buttons, each built from its
//! `MenuItem`/`MenuSeparator` children, and a `MenuItem` invokes a managed
//! callback rather than dispatching a native action.

use std::sync::Arc;

use gpui::{div, AnyElement, IntoElement as _, ParentElement as _, Styled as _};
use gpui_component::button::Button;
use gpui_component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_component::Disableable as _;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone)]
enum MenuEntry {
    Item {
        label: String,
        disabled: bool,
        checked: bool,
        callback: Option<ComponentCallback>,
    },
    Separator,
}

#[derive(Clone)]
struct MenuSpec {
    label: String,
    disabled: bool,
    entries: Vec<MenuEntry>,
}

#[derive(Clone)]
struct MenuItemPayload(String);

#[derive(Clone)]
enum MenuItemOp {
    Disabled(bool),
    Checked(bool),
    OnSelect(ComponentArgument),
}

#[derive(Clone)]
struct MenuPayload(String);

struct MenuItemMaterializer;

impl ComponentMaterializer for MenuItemMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let label = request
            .payload()
            .downcast_ref::<MenuItemPayload>()
            .ok_or_else(|| "MenuItem received an incompatible payload".to_string())?
            .0
            .clone();
        let mut disabled = false;
        let mut checked = false;
        let mut callback = None;
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<MenuItemOp>())
        {
            match operation {
                MenuItemOp::Disabled(value) => disabled = *value,
                MenuItemOp::Checked(value) => checked = *value,
                MenuItemOp::OnSelect(argument) => {
                    callback = Some(request.resolve_callback(argument)?);
                }
            }
        }
        let _ = request.take_style();
        Ok(Carrier::new(MenuEntry::Item {
            label,
            disabled,
            checked,
            callback,
        })
        .into_any_element())
    }
}

struct MenuSeparatorMaterializer;

impl ComponentMaterializer for MenuSeparatorMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let _ = request.take_style();
        Ok(Carrier::new(MenuEntry::Separator).into_any_element())
    }
}

struct MenuMaterializer;

impl ComponentMaterializer for MenuMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let label = request
            .payload()
            .downcast_ref::<MenuPayload>()
            .ok_or_else(|| "Menu received an incompatible payload".to_string())?
            .0
            .clone();
        let disabled = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<MenuItemOp>())
            .any(|op| matches!(op, MenuItemOp::Disabled(true)));
        let entries = request.take_typed_children::<MenuEntry>(&["MenuItem", "MenuSeparator"])?;
        let _ = request.take_style();
        Ok(Carrier::new(MenuSpec {
            label,
            disabled,
            entries,
        })
        .into_any_element())
    }
}

fn menu_button(id: String, spec: MenuSpec) -> AnyElement {
    let mut button = Button::new(id).label(spec.label.clone());
    if spec.disabled {
        button = button.disabled(true);
    }
    let entries = spec.entries;
    button
        .dropdown_menu(move |menu, _, _| {
            entries.iter().fold(menu, |menu, entry| match entry {
                MenuEntry::Separator => menu.separator(),
                MenuEntry::Item {
                    label,
                    disabled,
                    checked,
                    callback,
                } => {
                    let callback = callback.clone();
                    let mut item = PopupMenuItem::new(label.clone())
                        .disabled(*disabled)
                        .checked(*checked);
                    if let Some(callback) = callback {
                        item = item.on_click(move |_, window, cx| {
                            callback.invoke_with("MenuItem.on_select", &[], window, cx);
                        });
                    }
                    menu.item(item)
                }
            })
        })
        .into_any_element()
}

struct MenuBarMaterializer;

impl ComponentMaterializer for MenuBarMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<MenuPayload>()
            .ok_or_else(|| "MenuBar received an incompatible payload".to_string())?
            .0
            .clone();
        let menus = request.take_typed_children::<MenuSpec>(&["Menu"])?;
        let _ = request.take_style();
        let mut bar = div().flex().flex_row().items_center().gap_1();
        for (index, spec) in menus.into_iter().enumerate() {
            bar = bar.child(menu_button(format!("{id}:menu:{index}"), spec));
        }
        Ok(bar.into_any_element())
    }
}

fn label_constructor(export: &'static str) -> ConstructorDescriptor {
    ConstructorDescriptor::new(
        export,
        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(label)] if !label.trim().is_empty() => {
                Ok(ComponentPayload::new(MenuPayload(label.clone())))
            }
            _ => Err(format!("{export} expects a non-empty label")),
        },
    )
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("MenuItem", Arc::new(MenuItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "MenuItem",
                    vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(label)] if !label.trim().is_empty() => {
                            Ok(ComponentPayload::new(MenuItemPayload(label.clone())))
                        }
                        _ => Err("MenuItem expects a non-empty label".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "disabled",
                        vec![ArgumentDescriptor::new("disabled", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(MenuItemOp::Disabled(*value)))
                            }
                            _ => Err("MenuItem.disabled expects a boolean".into()),
                        },
                    )
                    .with_documentation("Disables the item."),
                    MethodDescriptor::new(
                        "checked",
                        vec![ArgumentDescriptor::new("checked", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(MenuItemOp::Checked(*value)))
                            }
                            _ => Err("MenuItem.checked expects a boolean".into()),
                        },
                    )
                    .with_documentation("Shows a check mark."),
                    MethodDescriptor::new(
                        "on_select",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                MenuItemOp::OnSelect(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("MenuItem.on_select(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Runs when the item is chosen."),
                ])
                .with_documentation("A typed menu item accepted only by a Menu."),
        )
        .expect("the built-in MenuItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("MenuSeparator", Arc::new(MenuSeparatorMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "MenuSeparator",
                    vec![],
                    |_| Ok(ComponentPayload::new(())),
                )])
                .with_methods(Vec::new())
                .with_documentation("A typed separator accepted only by a Menu."),
        )
        .expect("the built-in MenuSeparator descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Menu", Arc::new(MenuMaterializer))
                .with_constructors(vec![label_constructor("Menu")])
                .with_methods(vec![MethodDescriptor::new(
                    "disabled",
                    vec![ArgumentDescriptor::new("disabled", ArgumentSchema::Boolean)],
                    |arguments| match arguments {
                        [ComponentArgument::Boolean(value)] => {
                            Ok(ComponentPayload::new(MenuItemOp::Disabled(*value)))
                        }
                        _ => Err("Menu.disabled expects a boolean".into()),
                    },
                )
                .with_documentation("Disables the whole menu.")])
                .with_documentation("A typed top-level menu accepted only by a MenuBar."),
        )
        .expect("the built-in Menu descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("MenuBar", Arc::new(MenuBarMaterializer))
                .with_constructors(vec![label_constructor("MenuBar")])
                .with_methods(Vec::new())
                .with_documentation("A row of Menu dropdowns built from typed children."),
        )
        .expect("the built-in MenuBar descriptor is valid");
}
