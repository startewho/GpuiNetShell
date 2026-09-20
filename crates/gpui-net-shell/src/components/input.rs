//! `Input`, adapted from `component-shell`'s `retained_forms/mod.rs`.
//!
//! A retained single-line text field. The native `InputState` entity is kept in
//! keyed state and its `InputEvent::Change` is forwarded to managed code through
//! `on_change(string)`.

use std::sync::Arc;

use gpui::{
    div, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _, Subscription,
};
use gpui_component::input::{Input, InputState};
use gpui_component::{Sizable as _, Size};

use super::common::nonempty_id;
use super::input_events::{InputCallbacks, RetainedInputCallbacks};
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct InputPayload(String);

#[derive(Clone)]
enum InputOp {
    Placeholder(String),
    Value(String),
    Disabled(bool),
    Size(Size),
    OnChange(ComponentArgument),
    OnFocus(ComponentArgument),
    OnBlur(ComponentArgument),
}

struct Host {
    state: Entity<InputState>,
    callbacks: RetainedInputCallbacks,
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
        let size = operations.iter().rev().find_map(|op| match op {
            InputOp::Size(value) => Some(*value),
            _ => None,
        });
        let callback_for = |pick: fn(&InputOp) -> Option<&ComponentArgument>| {
            operations
                .iter()
                .rev()
                .find_map(pick)
                .cloned()
                .map(|argument| request.resolve_callback(&argument))
                .transpose()
        };
        let callbacks = InputCallbacks {
            change: callback_for(|op| match op {
                InputOp::OnChange(argument) => Some(argument),
                _ => None,
            })?,
            focus: callback_for(|op| match op {
                InputOp::OnFocus(argument) => Some(argument),
                _ => None,
            })?,
            blur: callback_for(|op| match op {
                InputOp::OnBlur(argument) => Some(argument),
                _ => None,
            })?,
        };
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-input:{id}")));
        let init_callbacks = callbacks.clone();
        let host = request.use_keyed_state(key, {
            let placeholder = placeholder.clone();
            let default_value = default_value.clone();
            move |window, cx| {
                let state = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder(placeholder)
                        .default_value(default_value)
                });
                let retained = RetainedInputCallbacks::new(init_callbacks);
                let selection = retained.subscribe(window, cx, &state, |state: &InputState| {
                    state.value().to_string()
                });
                Host {
                    state,
                    callbacks: retained,
                    _selection: selection,
                }
            }
        });
        request.update_entity(&host, |host, _| host.callbacks.set(callbacks));

        let state = request.with_window_app(|_, app| host.read(app).state.clone());
        let mut input = Input::new(&state);
        if disabled {
            input = input.disabled(true);
        }
        if let Some(size) = size {
            input = input.with_size(size);
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

fn callback(
    name: &'static str,
    documentation: &'static str,
    make: fn(ComponentArgument) -> InputOp,
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
            _ => Err(format!("Input.{name}(callback) expects a callback")),
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
                    method(
                        "size",
                        "Sets the control size; it also fixes the field height.",
                        ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        |arg| match arg {
                            ComponentArgument::Enum(value) => {
                                Some(InputOp::Size(Size::from_str(value)))
                            }
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
                    callback(
                        "on_focus",
                        "Runs when the field gains focus.",
                        InputOp::OnFocus,
                    ),
                    callback(
                        "on_blur",
                        "Runs when the field loses focus.",
                        InputOp::OnBlur,
                    ),
                ])
                .with_documentation("A retained single-line text field."),
        )
        .expect("the built-in Input descriptor is valid");
}

#[cfg(test)]
mod tests {
    #[test]
    fn input_declares_focus_and_blur_events() {
        let frozen = crate::components::catalog();
        let descriptor = frozen
            .descriptors()
            .find(|descriptor| descriptor.name() == "Input")
            .expect("Input is registered");
        assert!(descriptor.method("on_focus").is_some());
        assert!(descriptor.method("on_blur").is_some());
    }
}
