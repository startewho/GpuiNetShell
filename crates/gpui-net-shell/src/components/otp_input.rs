//! `OtpInput`, adapted from `component-shell`'s `retained_forms/mod.rs`.
//!
//! A retained fixed-length one-time-password field. It keeps a native `OtpState`
//! in keyed state and reports the entered code through `on_change(string)`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _, Subscription,
};
use gpui_component::input::{OtpEvent, OtpInput, OtpState};
use gpui_component::Disableable as _;

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

const DEFAULT_LENGTH: usize = 6;

#[derive(Clone)]
struct OtpInputPayload(String);

#[derive(Clone)]
enum OtpInputOp {
    Length(usize),
    Groups(usize),
    Disabled(bool),
    OnChange(ComponentArgument),
}

struct Host {
    state: Entity<OtpState>,
    callback: Rc<RefCell<Option<ComponentCallback>>>,
    _selection: Subscription,
}

struct OtpInputMaterializer;

impl ComponentMaterializer for OtpInputMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<OtpInputPayload>()
            .ok_or_else(|| "OtpInput received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<OtpInputOp>().cloned())
            .collect::<Vec<_>>();
        let length = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                OtpInputOp::Length(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(DEFAULT_LENGTH);
        let groups = operations.iter().rev().find_map(|op| match op {
            OtpInputOp::Groups(value) => Some(*value),
            _ => None,
        });
        let disabled = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                OtpInputOp::Disabled(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_else(|| request.disabled());
        let on_change = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                OtpInputOp::OnChange(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_callback(&argument))
            .transpose()?;
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-otp:{id}")));
        let on_change_for_init = on_change.clone();
        let host = request.use_keyed_state(key, move |window, cx| {
            let state = cx.new(|cx| OtpState::new(length, window, cx));
            let callback = Rc::new(RefCell::new(on_change_for_init));
            let event_callback = callback.clone();
            let selection = window.subscribe(
                &state,
                cx,
                move |state: Entity<OtpState>, event: &OtpEvent, window, cx| {
                    if matches!(event, OtpEvent::Change) {
                        let text = state.read(cx).value().to_string();
                        if let Some(callback) = event_callback.borrow().clone() {
                            callback.invoke_with(
                                "OtpInput.on_change callback failed",
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
        });
        request.update_entity(&host, |host, _| {
            *host.callback.borrow_mut() = on_change;
        });

        let state = request.with_window_app(|_, app| host.read(app).state.clone());
        let mut input = OtpInput::new(&state);
        if let Some(groups) = groups {
            input = input.groups(groups);
        }
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
    make: fn(&ComponentArgument) -> Option<OtpInputOp>,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, schema)],
        move |args| {
            args.first()
                .and_then(make)
                .map(ComponentPayload::new)
                .ok_or_else(|| format!("OtpInput.{name} received an invalid value"))
        },
    )
    .with_documentation(documentation)
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("OtpInput", Arc::new(OtpInputMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "OtpInput",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "OtpInput")
                            .map(OtpInputPayload)
                            .map(ComponentPayload::new),
                        _ => Err("OtpInput(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    method(
                        "length",
                        "Sets the fixed number of digits.",
                        ArgumentSchema::Number,
                        |arg| match arg {
                            ComponentArgument::Number(value)
                                if value.is_finite() && *value >= 1.0 && value.fract() == 0.0 =>
                            {
                                Some(OtpInputOp::Length(*value as usize))
                            }
                            _ => None,
                        },
                    ),
                    method(
                        "groups",
                        "Splits the code into the requested number of visual groups.",
                        ArgumentSchema::Number,
                        |arg| match arg {
                            ComponentArgument::Number(value)
                                if value.is_finite() && *value >= 1.0 && value.fract() == 0.0 =>
                            {
                                Some(OtpInputOp::Groups(*value as usize))
                            }
                            _ => None,
                        },
                    ),
                    method(
                        "disabled",
                        "Disables the field.",
                        ArgumentSchema::Boolean,
                        |arg| match arg {
                            ComponentArgument::Boolean(value) => Some(OtpInputOp::Disabled(*value)),
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
                                OtpInputOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("OtpInput.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the entered code after each edit."),
                ])
                .with_documentation("A retained fixed-length one-time-password field."),
        )
        .expect("the built-in OtpInput descriptor is valid");
}
