//! `Input`, adapted from `component-shell`'s `retained_forms/mod.rs`.
//!
//! A retained single-line text field. The native `InputState` entity is kept in
//! keyed state and its `InputEvent::Change` is forwarded to managed code through
//! `on_change(string)`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _, Subscription,
};
use gpui_component::input::{Input, InputEvent, InputState};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct InputPayload(String);

#[derive(Clone)]
enum InputOp {
    Placeholder(String),
    Value(String),
    Disabled(bool),
    OnChange(ComponentArgument),
}

struct Host {
    state: Entity<InputState>,
    callback: Rc<RefCell<Option<ComponentCallback>>>,
    _selection: Subscription,
}

struct InputMaterializer;

impl ComponentMaterializer for InputMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<InputPayload>()
            .ok_or_else(|| "Input received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<InputOp>().cloned())
            .collect::<Vec<_>>();
        let placeholder = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                InputOp::Placeholder(value) => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let default_value = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                InputOp::Value(value) => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let disabled = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                InputOp::Disabled(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_else(|| request.disabled());
        let on_change = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                InputOp::OnChange(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_callback(&argument))
            .transpose()?;
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-input:{id}")));
        let on_change_for_init = on_change.clone();
        let host = request.use_keyed_state(key, {
            let placeholder = placeholder.clone();
            let default_value = default_value.clone();
            move |window, cx| {
                let state = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder(placeholder)
                        .default_value(default_value)
                });
                let callback = Rc::new(RefCell::new(on_change_for_init));
                let event_callback = callback.clone();
                let selection = window.subscribe(
                    &state,
                    cx,
                    move |state: Entity<InputState>, event: &InputEvent, window, cx| {
                        if matches!(event, InputEvent::Change) {
                            let text = state.read(cx).value().to_string();
                            if let Some(callback) = event_callback.borrow().clone() {
                                callback.invoke_with(
                                    "Input.on_change callback failed",
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
        let mut input = Input::new(&state);
        if disabled {
            input = input.disabled(true);
        }
        let mut wrapper = div().child(input);
        wrapper.style().refine(&style);
        Ok(wrapper.into_any_element())
    }
}

fn method(
    name: &'static str,
    documentation: &'static str,
    schema: ArgumentSchema,
    make: fn(&ComponentArgument) -> Option<InputOp>,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, schema)],
        move |args| {
            args.first()
                .and_then(make)
                .map(ComponentPayload::new)
                .ok_or_else(|| format!("Input.{name} received an invalid value"))
        },
    )
    .with_documentation(documentation)
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Input", Arc::new(InputMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Input",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "Input")
                            .map(InputPayload)
                            .map(ComponentPayload::new),
                        _ => Err("Input(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    method(
                        "placeholder",
                        "Sets the placeholder shown while empty.",
                        ArgumentSchema::String,
                        |arg| match arg {
                            ComponentArgument::String(value) => {
                                Some(InputOp::Placeholder(value.clone()))
                            }
                            _ => None,
                        },
                    ),
                    method(
                        "value",
                        "Sets the initial text.",
                        ArgumentSchema::String,
                        |arg| match arg {
                            ComponentArgument::String(value) => Some(InputOp::Value(value.clone())),
                            _ => None,
                        },
                    ),
                    method(
                        "disabled",
                        "Disables the field.",
                        ArgumentSchema::Boolean,
                        |arg| match arg {
                            ComponentArgument::Boolean(value) => Some(InputOp::Disabled(*value)),
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
                                InputOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("Input.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the new text after each edit."),
                ])
                .with_documentation("A retained single-line text field."),
        )
        .expect("the built-in Input descriptor is valid");
}
