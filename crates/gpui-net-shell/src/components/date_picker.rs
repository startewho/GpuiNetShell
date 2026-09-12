//! `DatePicker`, adapted from `component-shell`'s `retained_forms/mod.rs`.
//!
//! A retained single-date picker. It keeps a native `DatePickerState` in keyed
//! state and reports the chosen date (ISO `YYYY-MM-DD`) through
//! `on_change(string)`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _, Subscription,
};
use gpui_component::date_picker::{DatePicker, DatePickerEvent, DatePickerState};
use gpui_component::Disableable as _;

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct DatePickerPayload(String);

#[derive(Clone)]
enum DatePickerOp {
    Placeholder(String),
    Disabled(bool),
    OnChange(ComponentArgument),
}

struct Host {
    state: Entity<DatePickerState>,
    callback: Rc<RefCell<Option<ComponentCallback>>>,
    _selection: Subscription,
}

struct DatePickerMaterializer;

impl ComponentMaterializer for DatePickerMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<DatePickerPayload>()
            .ok_or_else(|| "DatePicker received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<DatePickerOp>().cloned())
            .collect::<Vec<_>>();
        let placeholder = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                DatePickerOp::Placeholder(value) => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let disabled = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                DatePickerOp::Disabled(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_else(|| request.disabled());
        let on_change = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                DatePickerOp::OnChange(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_callback(&argument))
            .transpose()?;
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-date-picker:{id}")));
        let on_change_for_init = on_change.clone();
        let host = request.use_keyed_state(key, move |window, cx| {
            let state = cx.new(|cx| DatePickerState::new(window, cx));
            let callback = Rc::new(RefCell::new(on_change_for_init));
            let event_callback = callback.clone();
            let selection = window.subscribe(
                &state,
                cx,
                move |_state: Entity<DatePickerState>, event: &DatePickerEvent, window, cx| {
                    let DatePickerEvent::Change(date) = event;
                    let text = date.to_string();
                    if let Some(callback) = event_callback.borrow().clone() {
                        callback.invoke_with(
                            "DatePicker.on_change callback failed",
                            &[ComponentCallbackArgument::String(text)],
                            window,
                            cx,
                        );
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
        let mut picker = DatePicker::new(&state);
        if !placeholder.is_empty() {
            picker = picker.placeholder(placeholder);
        }
        if disabled {
            picker = picker.disabled(true);
        }
        let mut wrapper = div().child(picker);
        wrapper.style().refine(&style);
        Ok(wrapper.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("DatePicker", Arc::new(DatePickerMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "DatePicker",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "DatePicker")
                            .map(DatePickerPayload)
                            .map(ComponentPayload::new),
                        _ => Err("DatePicker(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "placeholder",
                        vec![ArgumentDescriptor::new(
                            "placeholder",
                            ArgumentSchema::String,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(
                                DatePickerOp::Placeholder(value.clone()),
                            )),
                            _ => Err("DatePicker.placeholder expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the empty-value prompt."),
                    MethodDescriptor::new(
                        "disabled",
                        vec![ArgumentDescriptor::new("disabled", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(DatePickerOp::Disabled(*value)))
                            }
                            _ => Err("DatePicker.disabled expects a boolean".into()),
                        },
                    )
                    .with_documentation("Disables the picker."),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                DatePickerOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("DatePicker.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the chosen date as an ISO string."),
                ])
                .with_documentation("A retained single-date picker."),
        )
        .expect("the built-in DatePicker descriptor is valid");
}
