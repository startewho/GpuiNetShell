//! `Settings` family, ported from `component-shell`'s `settings/mod.rs`.
//!
//! A typed settings hierarchy: `Settings` accepts `SettingPage` children, a
//! `SettingPage` accepts `SettingGroup` children, and a `SettingGroup` accepts
//! `SettingItem` children. `SettingItem` requires a lazy `content` element.

use std::sync::Arc;

use gpui::{
    div, px, AnyElement, Axis, IntoElement as _, ParentElement as _, Refineable as _, Styled as _,
};
use gpui_component::setting::{
    SelectIndex, SettingField, SettingGroup, SettingItem, SettingPage, Settings,
};
use gpui_component::{Sizable as _, Size};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone)]
struct TextPayload(String);

#[derive(Clone, Copy)]
struct Empty;

#[derive(Clone)]
enum TextOp {
    Title(String),
    Description(String),
}

#[derive(Clone)]
enum ItemOp {
    Description(String),
    Layout(Axis),
    Keywords(Vec<String>),
    Disabled(bool),
}

#[derive(Clone, Copy)]
enum PageBoolOp {
    DefaultOpen(bool),
    Resettable(bool),
}

#[derive(Clone, Copy)]
enum SettingsOp {
    Size(Size),
    SidebarWidth(f32),
    Selected(usize),
}

fn positive(value: f64, label: &str) -> Result<f32, String> {
    if value.is_finite() && value > 0.0 && value <= f32::MAX as f64 {
        Ok(value as f32)
    } else {
        Err(format!("{label} expects a positive finite pixel value"))
    }
}

struct ItemMaterializer;

impl ComponentMaterializer for ItemMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let title = request
            .payload()
            .downcast_ref::<TextPayload>()
            .ok_or_else(|| "SettingItem received an incompatible payload".to_string())?
            .0
            .clone();
        let field = request
            .take_slot_factory("content")
            .ok_or_else(|| "SettingItem requires content(element)".to_string())?;
        if request.children_len() != 0 {
            return Err("SettingItem does not accept children".to_string());
        }
        let mut sf = SettingField::render(move |_, window, cx| match field.build(window, cx) {
            Ok(element) => element,
            Err(error) => div()
                .child(format!("Failed to render setting field: {error}"))
                .into_any_element(),
        });
        sf.style().refine(&request.take_style());
        let mut item = SettingItem::new(title, sf);
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ItemOp>())
        {
            item = match op {
                ItemOp::Description(value) => item.description(value.clone()),
                ItemOp::Layout(value) => item.layout(*value),
                ItemOp::Keywords(value) => item.keywords(value.clone()),
                ItemOp::Disabled(value) => item.disabled(*value),
            };
        }
        Ok(Carrier::new(item).into_any_element())
    }
}

struct GroupMaterializer;

impl ComponentMaterializer for GroupMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<Empty>()
            .ok_or_else(|| "SettingGroup received an incompatible payload".to_string())?;
        let mut group = SettingGroup::new();
        for op in request.methods() {
            match op.payload().downcast_ref::<TextOp>() {
                Some(TextOp::Title(value)) => group = group.title(value.clone()),
                Some(TextOp::Description(value)) => group = group.description(value.clone()),
                None => {}
            }
        }
        group.style().refine(&request.take_style());
        let items = request.take_typed_children::<SettingItem>(&["SettingItem"])?;
        for item in items {
            group = group.item(item);
        }
        Ok(Carrier::new(group).into_any_element())
    }
}

struct PageMaterializer;

impl ComponentMaterializer for PageMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let title = request
            .payload()
            .downcast_ref::<TextPayload>()
            .ok_or_else(|| "SettingPage received an incompatible payload".to_string())?
            .0
            .clone();
        let mut page = SettingPage::new(title);
        for op in request.methods() {
            match op.payload().downcast_ref::<TextOp>() {
                Some(TextOp::Title(_)) => {}
                Some(TextOp::Description(value)) => page = page.description(value.clone()),
                None => {}
            }
            match op.payload().downcast_ref::<PageBoolOp>() {
                Some(PageBoolOp::DefaultOpen(value)) => page = page.default_open(*value),
                Some(PageBoolOp::Resettable(value)) => page = page.resettable(*value),
                None => {}
            }
        }
        if let Some(factory) = request.take_slot_factory("content") {
            page = page.title_suffix(move |window, cx| match factory.build(window, cx) {
                Ok(element) => element,
                Err(error) => div()
                    .child(format!("Failed to render title suffix: {error}"))
                    .into_any_element(),
            });
        }
        let _ = request.take_style();
        let groups = request.take_typed_children::<SettingGroup>(&["SettingGroup"])?;
        for group in groups {
            page = page.group(group);
        }
        Ok(Carrier::new(page).into_any_element())
    }
}

struct SettingsMaterializer;

impl ComponentMaterializer for SettingsMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<TextPayload>()
            .ok_or_else(|| "Settings received an incompatible payload".to_string())?
            .0
            .clone();
        let mut settings = Settings::new(id);
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<SettingsOp>())
        {
            settings = match op {
                SettingsOp::Size(value) => settings.with_size(*value),
                SettingsOp::SidebarWidth(value) => settings.sidebar_width(px(*value)),
                SettingsOp::Selected(value) => settings.default_selected_index(SelectIndex {
                    page_ix: *value,
                    group_ix: None,
                }),
            };
        }
        let _ = request.take_style();
        let pages = request.take_typed_children::<SettingPage>(&["SettingPage"])?;
        for page in pages {
            settings = settings.page(page);
        }
        Ok(settings.into_any_element())
    }
}

fn text_method(name: &'static str, op: fn(String) -> TextOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new("text", ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                Ok(ComponentPayload::new(op(value.clone())))
            }
            _ => Err(format!("{name} expects non-empty text")),
        },
    )
    .with_documentation("Sets the supporting text.")
}

fn bool_method(name: &'static str, op: fn(bool) -> PageBoolOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(op(*value))),
            _ => Err(format!("{name} expects one boolean")),
        },
    )
    .with_documentation("Controls native behavior.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("SettingItem", Arc::new(ItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "SettingItem",
                    vec![ArgumentDescriptor::new("title", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                            Ok(ComponentPayload::new(TextPayload(value.clone())))
                        }
                        _ => Err("SettingItem expects a non-empty title".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "description",
                        vec![ArgumentDescriptor::new("text", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(
                                ItemOp::Description(value.clone()),
                            )),
                            _ => Err("SettingItem.description expects text".into()),
                        },
                    )
                    .with_documentation("Sets the supporting description."),
                    MethodDescriptor::new(
                        "layout",
                        vec![ArgumentDescriptor::new(
                            "axis",
                            ArgumentSchema::Enum(&["horizontal", "vertical"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "horizontal" => Ok(ComponentPayload::new(ItemOp::Layout(
                                    Axis::Horizontal,
                                ))),
                                "vertical" => {
                                    Ok(ComponentPayload::new(ItemOp::Layout(Axis::Vertical)))
                                }
                                _ => Err("layout expects horizontal or vertical".into()),
                            },
                            _ => Err("layout expects horizontal or vertical".into()),
                        },
                    )
                    .with_documentation("Lays the label and field out along the given axis."),
                    MethodDescriptor::new(
                        "keywords",
                        vec![ArgumentDescriptor::new("keywords", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                let keywords = value
                                    .split('\n')
                                    .filter(|keyword| !keyword.trim().is_empty())
                                    .map(str::to_owned)
                                    .collect();
                                Ok(ComponentPayload::new(ItemOp::Keywords(keywords)))
                            }
                            _ => Err("keywords expects a newline-separated string".into()),
                        },
                    )
                    .with_documentation("Adds search keywords that match this item."),
                    MethodDescriptor::new(
                        "disabled",
                        vec![ArgumentDescriptor::new("disabled", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(ItemOp::Disabled(*value)))
                            }
                            _ => Err("SettingItem.disabled expects a boolean".into()),
                        },
                    )
                    .with_documentation("Disables the item's field."),
                ])
                .with_documentation(
                    "A typed setting item requiring a lazy content(element); style applies to the field.",
                ),
        )
        .expect("the built-in SettingItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("SettingGroup", Arc::new(GroupMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "SettingGroup",
                    vec![],
                    |_| Ok(ComponentPayload::new(Empty)),
                )])
                .with_methods(vec![
                    text_method("title", TextOp::Title),
                    text_method("description", TextOp::Description),
                ])
                .with_documentation("A styled setting group accepting only SettingItem children."),
        )
        .expect("the built-in SettingGroup descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("SettingPage", Arc::new(PageMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "SettingPage",
                    vec![ArgumentDescriptor::new("title", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                            Ok(ComponentPayload::new(TextPayload(value.clone())))
                        }
                        _ => Err("SettingPage expects a non-empty title".into()),
                    },
                )])
                .with_methods(vec![
                    text_method("description", TextOp::Description),
                    bool_method("default_open", PageBoolOp::DefaultOpen),
                    bool_method("resettable", PageBoolOp::Resettable),
                ])
                .with_documentation(
                    "A typed setting page accepting SettingGroup children and a lazy title suffix.",
                ),
        )
        .expect("the built-in SettingPage descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Settings", Arc::new(SettingsMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Settings",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                            Ok(ComponentPayload::new(TextPayload(value.clone())))
                        }
                        _ => Err("Settings expects a non-empty id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => {
                                    Ok(ComponentPayload::new(SettingsOp::Size(Size::XSmall)))
                                }
                                "small" => Ok(ComponentPayload::new(SettingsOp::Size(Size::Small))),
                                "medium" => {
                                    Ok(ComponentPayload::new(SettingsOp::Size(Size::Medium)))
                                }
                                "large" => Ok(ComponentPayload::new(SettingsOp::Size(Size::Large))),
                                _ => Err("unsupported size".into()),
                            },
                            _ => Err("size expects a semantic size".into()),
                        },
                    )
                    .with_documentation("Sets the settings surface's semantic size."),
                    MethodDescriptor::new(
                        "sidebar_width",
                        vec![ArgumentDescriptor::new("pixels", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)] => positive(*value, "sidebar_width")
                                .map(SettingsOp::SidebarWidth)
                                .map(ComponentPayload::new),
                            _ => Err("sidebar_width expects a number".into()),
                        },
                    )
                    .with_documentation("Sets the sidebar's width in pixels."),
                    MethodDescriptor::new(
                        "default_selected_page",
                        vec![ArgumentDescriptor::new("index", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 =>
                            {
                                Ok(ComponentPayload::new(SettingsOp::Selected(*value as usize)))
                            }
                            _ => Err("default_selected_page expects a nonnegative integer".into()),
                        },
                    )
                    .with_documentation("Selects the page shown when the surface first opens."),
                ])
                .with_documentation(
                    "A settings container accepting only SettingPage children; style is rejected.",
                ),
        )
        .expect("the built-in Settings descriptor is valid");
}
