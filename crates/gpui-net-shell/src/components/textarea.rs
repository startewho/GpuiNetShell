//! `Textarea`, adapted from `component-shell`'s `retained_forms/mod.rs`.
//!
//! A retained multi-line text field. It keeps a native `TextareaState` in keyed
//! state and reports edits through `on_change(string)`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _, Subscription,
};
use gpui_component::input::{InputEvent, Textarea, TextareaState};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct TextareaPayload(String);

#[derive(Clone)]
enum TextareaOp {
    Placeholder(String),
    Value(String),
    Disabled(bool),
    OnChange(ComponentArgument),
}

struct Host {
    state: Entity<TextareaState>,
    callback: Rc<RefCell<Option<ComponentCallback>>>,
    _selection: Subscription,
}

struct TextareaMaterializer;

impl ComponentMaterializer for TextareaMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<TextareaPayload>()
            .ok_or_else(|| "Textarea received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<TextareaOp>().cloned())
            .collect::<Vec<_>>();
        let placeholder = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                TextareaOp::Placeholder(value) => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let default_value = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                TextareaOp::Value(value) => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let disabled = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                TextareaOp::Disabled(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_else(|| request.disabled());
        let on_change = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                TextareaOp::OnChange(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_callback(&argument))
            .transpose()?;
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-textarea:{id}")));
        let on_change_for_init = on_change.clone();
        let host = request.use_keyed_state(key, {
            let placeholder = placeholder.clone();
            let default_value = default_value.clone();
            move |window, cx| {
                let state = cx.new(|cx| {
                    TextareaState::new(window, cx)
                        .placeholder(placeholder)
                        .default_value(default_value)
                });
                let callback = Rc::new(RefCell::new(on_change_for_init));
                let event_callback = callback.clone();
                let selection = window.subscribe(
                    &state,
                    cx,
                    move |state: Entity<TextareaState>, event: &InputEvent, window, cx| {
                        if matches!(event, InputEvent::Change) {
                            let text = state.read(cx).value().to_string();
                            if let Some(callback) = event_callback.borrow().clone() {
                                callback.invoke_with(
                                    "Textarea.on_change callback failed",
                                    &[ComponentCallbackArgument::String(text)],
                                    window,
                                    cx,
                                );
                            }
                        }
                    },
                );
                Host {
                    state,
                    callback,
                    _selection: selection,
                }
            }
        });
        request.update_entity(&host, |host, _| {
            *host.callback.borrow_mut() = on_change;
        });

        let state = request.with_window_app(|_, app| host.read(app).state.clone());
        let mut textarea = Textarea::new(&state);
        if disabled {
            textarea = textarea.disabled(true);
        }
        let mut wrapper = div().child(textarea);
        wrapper.style().refine(&style);
        Ok(wrapper.into_any_element())
    }
}

fn method(
    name: &'static str,
    documentation: &'static str,
    schema: ArgumentSchema,
    make: fn(&ComponentArgument) -> Option<TextareaOp>,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, schema)],
        move |args| {
            args.first()
                .and_then(make)
                .map(ComponentPayload::new)
                .ok_or_else(|| format!("Textarea.{name} received an invalid value"))
        },
    )
    .with_documentation(documentation)
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Textarea", Arc::new(TextareaMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Textarea",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "Textarea")
                            .map(TextareaPayload)
                            .map(ComponentPayload::new),
                        _ => Err("Textarea(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    method(
                        "placeholder",
                        "Sets the placeholder shown while empty.",
                        ArgumentSchema::String,
                        |arg| match arg {
                            ComponentArgument::String(value) => {
                                Some(TextareaOp::Placeholder(value.clone()))
                            }
                            _ => None,
                        },
                    ),
                    method(
                        "value",
                        "Sets the initial text.",
                        ArgumentSchema::String,
                        |arg| match arg {
                            ComponentArgument::String(value) => {
                                Some(TextareaOp::Value(value.clone()))
                            }
                            _ => None,
                        },
                    ),
                    method(
                        "disabled",
                        "Disables the field.",
                        ArgumentSchema::Boolean,
                        |arg| match arg {
                            ComponentArgument::Boolean(value) => Some(TextareaOp::Disabled(*value)),
                            _ => None,
                        },
                    ),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                TextareaOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("Textarea.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the new text after each edit."),
                ])
                .with_documentation("A retained multi-line text field."),
        )
        .expect("the built-in Textarea descriptor is valid");
}
