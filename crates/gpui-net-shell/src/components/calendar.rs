//! `Calendar`, adapted from `component-shell`'s `retained_forms/mod.rs`.
//!
//! A retained calendar. It keeps a native `CalendarState` in keyed state and
//! reports the selected date (ISO `YYYY-MM-DD`) through `on_change(string)`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _, Subscription,
};
use gpui_component::calendar::{Calendar, CalendarEvent, CalendarState};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct CalendarPayload(String);

#[derive(Clone)]
enum CalendarOp {
    Months(usize),
    OnChange(ComponentArgument),
}

struct Host {
    state: Entity<CalendarState>,
    callback: Rc<RefCell<Option<ComponentCallback>>>,
    _selection: Subscription,
}

struct CalendarMaterializer;

impl ComponentMaterializer for CalendarMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<CalendarPayload>()
            .ok_or_else(|| "Calendar received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<CalendarOp>().cloned())
            .collect::<Vec<_>>();
        let months = operations.iter().rev().find_map(|op| match op {
            CalendarOp::Months(value) => Some(*value),
            _ => None,
        });
        let on_change = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                CalendarOp::OnChange(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_callback(&argument))
            .transpose()?;
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-calendar:{id}")));
        let on_change_for_init = on_change.clone();
        let host = request.use_keyed_state(key, move |window, cx| {
            let state = cx.new(|cx| CalendarState::new(window, cx));
            let callback = Rc::new(RefCell::new(on_change_for_init));
            let event_callback = callback.clone();
            let selection = window.subscribe(
                &state,
                cx,
                move |_state: Entity<CalendarState>, event: &CalendarEvent, window, cx| {
                    let CalendarEvent::Selected(date) = event;
                    let text = date.to_string();
                    if let Some(callback) = event_callback.borrow().clone() {
                        callback.invoke_with(
                            "Calendar.on_change callback failed",
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
        let mut calendar = Calendar::new(&state);
        if let Some(months) = months {
            calendar = calendar.number_of_months(months);
        }
        let mut wrapper = div().child(calendar);
        wrapper.style().refine(&style);
        Ok(wrapper.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Calendar", Arc::new(CalendarMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Calendar",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "Calendar")
                            .map(CalendarPayload)
                            .map(ComponentPayload::new),
                        _ => Err("Calendar(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "number_of_months",
                        vec![ArgumentDescriptor::new("count", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite() && *value >= 1.0 && value.fract() == 0.0 =>
                            {
                                Ok(ComponentPayload::new(CalendarOp::Months(*value as usize)))
                            }
                            _ => Err(
                                "Calendar.number_of_months(count) expects a positive integer"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Sets the positive number of adjacent months to display."),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                CalendarOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("Calendar.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the selected date as an ISO string."),
                ])
                .with_documentation("A retained calendar for date navigation and selection."),
        )
        .expect("the built-in Calendar descriptor is valid");
}
