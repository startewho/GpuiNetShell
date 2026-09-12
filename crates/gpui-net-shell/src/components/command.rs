//! The `Command` palette family, adapted from `component-shell`'s
//! `command/command.rs`.
//!
//! `component-shell` dispatches shell actions from a `CommandItem`; this runtime
//! has no action system, so items carry only data and the palette reports
//! query/selection through managed callbacks. Selection paths are reported as a
//! `"section,row"` string because the managed callback channel carries one value
//! per invocation.

use std::sync::Arc;

use gpui::{
    div, AnyElement, Entity, IntoElement as _, ParentElement as _, Refineable as _, SharedString,
    Styled as _,
};
use gpui_component::command::{Command, CommandGroup, CommandItem, CommandState};
use gpui_component::Disableable as _;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallbackArgument,
    ComponentDescriptor, ComponentMaterializer, ComponentPayload, ComponentRegistry,
    ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::{take_typed, Carrier};

#[derive(Clone)]
struct ItemPayload(String);

#[derive(Clone)]
struct GroupPayload(String);

#[derive(Clone)]
struct CommandPayload(String);

#[derive(Clone, Copy)]
struct Separator;

#[derive(Clone)]
enum ItemOp {
    Keyword(String),
    Checked(bool),
}

#[derive(Clone)]
enum CommandOp {
    Searchable(bool),
    Filterable(bool),
    Bordered(bool),
    Placeholder(String),
    MaxHeight(f32),
    OnQuery(ComponentArgument),
    OnSelect(ComponentArgument),
    OnCancel(ComponentArgument),
}

struct ItemMaterializer;

impl ComponentMaterializer for ItemMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let label = request
            .payload()
            .downcast_ref::<ItemPayload>()
            .ok_or_else(|| "CommandItem received an incompatible payload".to_string())?
            .0
            .clone();
        let mut item = CommandItem::new().label(label).disabled(request.disabled());
        let mut keywords = Vec::new();
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ItemOp>())
        {
            match op {
                ItemOp::Keyword(value) => keywords.push(value.clone()),
                ItemOp::Checked(value) => item = item.checked(*value),
            }
        }
        item = item.keywords(keywords);
        let _ = request.take_style();
        if let Some(factory) = request.take_slot_factory("content") {
            item = item.child(move |window, cx| match factory.build(window, cx) {
                Ok(element) => element,
                Err(error) => div()
                    .child(format!("Failed to render CommandItem content: {error}"))
                    .into_any_element(),
            });
        }
        if request.children_len() != 0 {
            return Err("CommandItem does not accept ordinary children".to_string());
        }
        Ok(Carrier::new(item).into_any_element())
    }
}

struct GroupMaterializer;

impl ComponentMaterializer for GroupMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let label = request
            .payload()
            .downcast_ref::<GroupPayload>()
            .ok_or_else(|| "CommandGroup received an incompatible payload".to_string())?
            .0
            .clone();
        let mut group = CommandGroup::new().label(label);
        let mut request = request;
        let items = request.take_typed_children::<CommandItem>(&["CommandItem"])?;
        for item in items {
            group = group.item(item);
        }
        Ok(Carrier::new(group).into_any_element())
    }
}

struct SeparatorMaterializer;

impl ComponentMaterializer for SeparatorMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let _ = request.take_style();
        Ok(Carrier::new(Separator).into_any_element())
    }
}

struct CommandMaterializer;

impl ComponentMaterializer for CommandMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<CommandPayload>()
            .ok_or_else(|| "Command received an incompatible payload".to_string())?
            .0
            .clone();
        let state: Entity<CommandState> = request.use_keyed_state(
            SharedString::from(format!("shell-command:{id}")),
            CommandState::new,
        );
        let mut command = Command::new(&state);
        if let Some(factory) = request.take_slot_factory("header") {
            command = command.header(move |_, window, cx| match factory.build(window, cx) {
                Ok(element) => element,
                Err(error) => div()
                    .child(format!("Failed to render Command header: {error}"))
                    .into_any_element(),
            });
        }
        if let Some(factory) = request.take_slot_factory("footer") {
            command = command.footer(move |_, window, cx| match factory.build(window, cx) {
                Ok(element) => element,
                Err(error) => div()
                    .child(format!("Failed to render Command footer: {error}"))
                    .into_any_element(),
            });
        }
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<CommandOp>().cloned())
            .collect::<Vec<_>>();
        for op in operations {
            command = match op {
                CommandOp::Searchable(value) => command.searchable(value),
                CommandOp::Filterable(value) => command.filterable(value),
                CommandOp::Bordered(value) => command.bordered(value),
                CommandOp::Placeholder(value) => command.placeholder(value),
                CommandOp::MaxHeight(value) => command.max_h(gpui::px(value)),
                CommandOp::OnQuery(argument) => {
                    let callback = request.resolve_callback(&argument)?;
                    command.on_query(move |query, window, cx| {
                        callback.invoke_with(
                            "Command.on_query",
                            &[ComponentCallbackArgument::String(query.to_owned())],
                            window,
                            cx,
                        )
                    })
                }
                CommandOp::OnSelect(argument) => {
                    let callback = request.resolve_callback(&argument)?;
                    command.on_select(move |path, window, cx| {
                        callback.invoke_with(
                            "Command.on_select",
                            &[ComponentCallbackArgument::String(format!(
                                "{},{}",
                                path.section, path.row
                            ))],
                            window,
                            cx,
                        )
                    })
                }
                CommandOp::OnCancel(argument) => {
                    let callback = request.resolve_callback(&argument)?;
                    command.on_cancel(move |window, cx| {
                        callback.invoke_with("Command.on_cancel", &[], window, cx)
                    })
                }
            };
        }
        for (name, mut element) in request.take_children_named() {
            command = match name {
                "CommandItem" => command.item(take_typed::<CommandItem>(&mut element, name)?),
                "CommandGroup" => command.group(take_typed::<CommandGroup>(&mut element, name)?),
                "CommandSeparator" => {
                    take_typed::<Separator>(&mut element, name)?;
                    command.separator()
                }
                other => {
                    return Err(format!(
                        "Command accepts only CommandItem, CommandGroup, or CommandSeparator \
                         children; received {other}"
                    ))
                }
            };
        }
        command.style().refine(&request.take_style());
        Ok(command.into_any_element())
    }
}

fn bool_method(name: &'static str, make: fn(bool) -> CommandOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(make(*value))),
            _ => Err(format!("Command.{name}({name}) expects a boolean")),
        },
    )
    .with_documentation("Sets native Command behavior.")
}

fn callback_method(
    name: &'static str,
    make: fn(ComponentArgument) -> CommandOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(
            "callback",
            ArgumentSchema::Callback,
        )],
        move |arguments| match arguments {
            [argument @ ComponentArgument::Callback(_)] => {
                Ok(ComponentPayload::new(make(argument.clone())))
            }
            _ => Err(format!("Command.{name} expects a callback")),
        },
    )
    .with_documentation("Runs after the native Command state releases its update lease.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("CommandItem", Arc::new(ItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "CommandItem",
                    vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(label)] if !label.trim().is_empty() => {
                            Ok(ComponentPayload::new(ItemPayload(label.clone())))
                        }
                        _ => Err("CommandItem expects a non-empty label".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "keyword",
                        vec![ArgumentDescriptor::new("keyword", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                                Ok(ComponentPayload::new(ItemOp::Keyword(value.clone())))
                            }
                            _ => Err("CommandItem.keyword expects non-empty text".into()),
                        },
                    )
                    .with_documentation("Sets the item search keyword."),
                    MethodDescriptor::new(
                        "checked",
                        vec![ArgumentDescriptor::new("checked", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(ItemOp::Checked(*value)))
                            }
                            _ => Err("CommandItem.checked expects a boolean".into()),
                        },
                    )
                    .with_documentation("Sets the item checked state."),
                ])
                .with_documentation("Typed native CommandItem data."),
        )
        .expect("the built-in CommandItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("CommandGroup", Arc::new(GroupMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "CommandGroup",
                    vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(label)] if !label.trim().is_empty() => {
                            Ok(ComponentPayload::new(GroupPayload(label.clone())))
                        }
                        _ => Err("CommandGroup expects a non-empty label".into()),
                    },
                )])
                .with_methods(Vec::new())
                .with_documentation(
                    "Typed native CommandGroup data accepting only CommandItem children.",
                ),
        )
        .expect("the built-in CommandGroup descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("CommandSeparator", Arc::new(SeparatorMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "CommandSeparator",
                    vec![],
                    |_| Ok(ComponentPayload::new(Separator)),
                )])
                .with_methods(Vec::new())
                .with_documentation("Typed Command separator data."),
        )
        .expect("the built-in CommandSeparator descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Command", Arc::new(CommandMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Command",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(CommandPayload(id.clone())))
                        }
                        _ => Err("Command expects a non-empty id".into()),
                    },
                )])
                .with_methods(vec![
                    bool_method("searchable", CommandOp::Searchable),
                    bool_method("filterable", CommandOp::Filterable),
                    bool_method("bordered", CommandOp::Bordered),
                    MethodDescriptor::new(
                        "placeholder",
                        vec![ArgumentDescriptor::new(
                            "placeholder",
                            ArgumentSchema::String,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                Ok(ComponentPayload::new(CommandOp::Placeholder(value.clone())))
                            }
                            _ => Err("Command.placeholder expects text".into()),
                        },
                    )
                    .with_documentation("Sets the command search placeholder."),
                    MethodDescriptor::new(
                        "max_height",
                        vec![ArgumentDescriptor::new("pixels", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite() && *value > 0.0 =>
                            {
                                Ok(ComponentPayload::new(CommandOp::MaxHeight(*value as f32)))
                            }
                            _ => Err("Command.max_height expects positive finite pixels".into()),
                        },
                    )
                    .with_documentation("Sets the command results maximum height."),
                    callback_method("on_query", CommandOp::OnQuery),
                    callback_method("on_select", CommandOp::OnSelect),
                    callback_method("on_cancel", CommandOp::OnCancel),
                ])
                .with_documentation(
                    "Styled retained native Command palette consuming CommandItem, CommandGroup \
                     and CommandSeparator in order.",
                ),
        )
        .expect("the built-in Command descriptor is valid");
}
