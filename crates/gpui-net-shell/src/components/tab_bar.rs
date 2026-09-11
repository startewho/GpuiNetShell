//! `TabBar` and `Tab`, ported from `component-shell`'s `typed_compound/mod.rs`.
//!
//! A typed tab list accepting only `Tab` children. `Tab` carries its native
//! value to the parent through [`crate::typed_child::Carrier`]; it renders
//! nothing on its own.

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, Refineable as _, Styled as _};
use gpui_component::{
    tab::{Tab, TabBar, TabVariant},
    Selectable as _, Sizable as _, Size,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallbackArgument,
    ComponentDescriptor, ComponentMaterializer, ComponentPayload, ComponentRegistry,
    ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone, Copy)]
struct TabPayload;

#[derive(Clone)]
enum TabOp {
    Label(String),
    AriaLabel(String),
    Disabled(bool),
    Selected(bool),
}

struct TabMaterializer;

impl ComponentMaterializer for TabMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<TabPayload>()
            .ok_or_else(|| "Tab received an incompatible payload".to_string())?;
        let mut tab = Tab::new()
            .disabled(request.disabled())
            .selected(request.selected());
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<TabOp>())
        {
            tab = match operation {
                TabOp::Label(value) => tab.label(value.clone()),
                TabOp::AriaLabel(value) => tab.aria_label(value.clone()),
                TabOp::Disabled(value) => tab.disabled(*value),
                TabOp::Selected(value) => tab.selected(*value),
            };
        }
        Ok(Carrier::new(tab).into_any_element())
    }
}

#[derive(Clone)]
struct TabBarPayload(String);

#[derive(Clone)]
enum TabBarOp {
    Selected(usize),
    Variant(TabVariant),
    Menu(bool),
    Size(Size),
    OnChange(ComponentArgument),
}

struct TabBarMaterializer;

impl ComponentMaterializer for TabBarMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<TabBarPayload>()
            .ok_or_else(|| "TabBar received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<TabBarOp>().cloned())
            .collect::<Vec<_>>();
        let tabs = request.take_typed_children::<Tab>(&["Tab"])?;
        let style = request.take_style();

        let mut bar = TabBar::new(id);
        for operation in operations {
            bar = match operation {
                TabBarOp::Selected(value) => bar.selected_index(value),
                TabBarOp::Variant(value) => bar.with_variant(value),
                TabBarOp::Menu(value) => bar.menu(value),
                TabBarOp::Size(value) => bar.with_size(value),
                TabBarOp::OnChange(argument) => {
                    let callback = request.resolve_callback(&argument)?;
                    bar.on_click(move |index, window, cx| {
                        callback.invoke_with(
                            "TabBar.on_change callback failed",
                            &[ComponentCallbackArgument::Number(*index as f64)],
                            window,
                            cx,
                        );
                    })
                }
            };
        }
        for tab in tabs {
            bar = bar.child(tab);
        }
        bar.style().refine(&style);
        Ok(bar.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Tab", Arc::new(TabMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new("Tab", vec![], |_| {
                    Ok(ComponentPayload::new(TabPayload))
                })])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "label",
                        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                Ok(ComponentPayload::new(TabOp::Label(value.clone())))
                            }
                            _ => Err("Tab.label(label) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the visible tab label."),
                    MethodDescriptor::new(
                        "aria_label",
                        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                Ok(ComponentPayload::new(TabOp::AriaLabel(value.clone())))
                            }
                            _ => Err("Tab.aria_label(label) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the accessible tab label."),
                    MethodDescriptor::new(
                        "disabled",
                        vec![ArgumentDescriptor::new("disabled", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(TabOp::Disabled(*value)))
                            }
                            _ => Err("Tab.disabled(disabled) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Disables the tab."),
                    MethodDescriptor::new(
                        "selected",
                        vec![ArgumentDescriptor::new("selected", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(TabOp::Selected(*value)))
                            }
                            _ => Err("Tab.selected(selected) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Controls selected state."),
                ])
                .with_documentation("A tab accepted only as a direct TabBar child."),
        )
        .expect("the built-in Tab descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("TabBar", Arc::new(TabBarMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "TabBar",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(TabBarPayload(id.clone())))
                        }
                        _ => Err("TabBar(id) expects a nonempty string id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "selected_index",
                        vec![ArgumentDescriptor::new("index", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 =>
                            {
                                Ok(ComponentPayload::new(TabBarOp::Selected(*value as usize)))
                            }
                            _ => {
                                Err("TabBar.selected_index(index) expects a nonnegative integer"
                                    .into())
                            }
                        },
                    )
                    .with_documentation("Controls the selected zero-based tab index."),
                    MethodDescriptor::new(
                        "variant",
                        vec![ArgumentDescriptor::new(
                            "variant",
                            ArgumentSchema::Enum(&[
                                "tab",
                                "outline",
                                "pill",
                                "segmented",
                                "underline",
                            ]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => {
                                let variant = match value.as_str() {
                                    "tab" => TabVariant::Tab,
                                    "outline" => TabVariant::Outline,
                                    "pill" => TabVariant::Pill,
                                    "segmented" => TabVariant::Segmented,
                                    "underline" => TabVariant::Underline,
                                    _ => {
                                        return Err(format!("unsupported TabBar variant `{value}`"))
                                    }
                                };
                                Ok(ComponentPayload::new(TabBarOp::Variant(variant)))
                            }
                            _ => Err("TabBar.variant(variant) expects a variant literal".into()),
                        },
                    )
                    .with_documentation("Sets one of the component's five tab variants."),
                    MethodDescriptor::new(
                        "menu",
                        vec![ArgumentDescriptor::new("menu", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(TabBarOp::Menu(*value)))
                            }
                            _ => Err("TabBar.menu(menu) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Enables the overflow menu."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => Ok(ComponentPayload::new(TabBarOp::Size(Size::XSmall))),
                                "small" => Ok(ComponentPayload::new(TabBarOp::Size(Size::Small))),
                                "medium" => Ok(ComponentPayload::new(TabBarOp::Size(Size::Medium))),
                                "large" => Ok(ComponentPayload::new(TabBarOp::Size(Size::Large))),
                                _ => Err(format!("unsupported TabBar size `{value}`")),
                            },
                            _ => Err("TabBar.size(size) expects a semantic size".into()),
                        },
                    )
                    .with_documentation("Sets the semantic component size."),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                TabBarOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("TabBar.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the selected zero-based tab index."),
                ])
                .with_documentation("A typed tab list accepting only Tab children."),
        )
        .expect("the built-in TabBar descriptor is valid");
}
