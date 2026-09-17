//! `Div` element events exposed to the managed host.
//!
//! The managed side subscribes to a subset of GPUI's `Div` handlers. Only the
//! subscribed events are bound here, so an unsubscribed event costs nothing:
//! no listener, no ABI traffic. `on_click` stays parameterless and uses the
//! existing `click` callback; every other event is delivered through `invoke`
//! with a tab-separated UTF-8 payload.
//!
//! The event names are the GPUI method names (`on_mouse_down`, `on_hover`, …),
//! so the wire and the Rust side read the same.

use std::fmt::Write as _;

use gpui::{
    App, ClickEvent, InteractiveElement, Keystroke, Modifiers, MouseButton, MouseDownEvent,
    MouseMoveEvent, MousePressureEvent, MouseUpEvent, ScrollDelta, ScrollWheelEvent,
    StatefulInteractiveElement,
};

use crate::context::HostContext;
use crate::schema::{CALLBACK_VALUE_BOOLEAN, CALLBACK_VALUE_STRING};

/// One element event the managed host can subscribe to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementEvent {
    Click,
    AuxClick,
    Hover,
    MouseDown,
    MouseUp,
    MouseDownOut,
    MouseUpOut,
    MouseMove,
    MousePressure,
    ScrollWheel,
    KeyDown,
    KeyUp,
}

impl ElementEvent {
    /// The event for a wire name, if the name is an element event.
    pub fn from_wire_name(name: &str) -> Option<Self> {
        Some(match name {
            "on_click" => Self::Click,
            "on_aux_click" => Self::AuxClick,
            "on_hover" => Self::Hover,
            "on_mouse_down" => Self::MouseDown,
            "on_mouse_up" => Self::MouseUp,
            "on_mouse_down_out" => Self::MouseDownOut,
            "on_mouse_up_out" => Self::MouseUpOut,
            "on_mouse_move" => Self::MouseMove,
            "on_mouse_pressure" => Self::MousePressure,
            "on_scroll_wheel" => Self::ScrollWheel,
            "on_key_down" => Self::KeyDown,
            "on_key_up" => Self::KeyUp,
            _ => return None,
        })
    }

    /// Whether GPUI keys this event's state by the element's `ElementId`, so a
    /// stable id is required for it to complete across frames.
    pub fn needs_element_id(self) -> bool {
        matches!(self, Self::Click | Self::AuxClick | Self::Hover)
    }

    /// Whether the handler requests a full repaint after firing. The
    /// high-frequency events (`move`, `scroll`) do not: a repaint per event
    /// would rebuild the whole window.
    pub fn auto_invalidate(self) -> bool {
        !matches!(self, Self::MouseMove | Self::ScrollWheel)
    }
}

/// The element events a node subscribed to, in declaration order.
#[derive(Clone, Default, Debug)]
pub struct ElementEvents {
    entries: Vec<(ElementEvent, u64)>,
}

impl ElementEvents {
    pub fn push(&mut self, event: ElementEvent, token: u64) {
        self.entries.push((event, token));
    }

    /// The most recently declared handler for `event`, if any.
    pub fn get(&self, event: ElementEvent) -> Option<u64> {
        self.entries
            .iter()
            .rev()
            .find(|(candidate, _)| *candidate == event)
            .map(|(_, token)| *token)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether any subscribed event needs a stable element id.
    pub fn needs_element_id(&self) -> bool {
        self.entries
            .iter()
            .any(|(event, _)| event.needs_element_id())
    }
}

/// Encodes modifiers the way the managed `InputModifiers` flags expect.
pub(crate) fn modifiers_code(modifiers: Modifiers) -> u32 {
    let mut flags = 0;
    if modifiers.shift {
        flags |= 1;
    }
    if modifiers.control {
        flags |= 2;
    }
    if modifiers.alt {
        flags |= 4;
    }
    if modifiers.platform {
        flags |= 8;
    }
    flags
}

fn button_code(button: MouseButton) -> i32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        _ => -1,
    }
}

fn pointer_payload(x: f32, y: f32, button: i32, click_count: usize, modifiers: u32) -> String {
    format!("{x}\t{y}\t{button}\t{click_count}\t{modifiers}")
}

fn mouse_down_payload(event: &MouseDownEvent) -> String {
    pointer_payload(
        f32::from(event.position.x),
        f32::from(event.position.y),
        button_code(event.button),
        event.click_count,
        modifiers_code(event.modifiers),
    )
}

fn mouse_up_payload(event: &MouseUpEvent) -> String {
    pointer_payload(
        f32::from(event.position.x),
        f32::from(event.position.y),
        button_code(event.button),
        event.click_count,
        modifiers_code(event.modifiers),
    )
}

fn mouse_move_payload(event: &MouseMoveEvent) -> String {
    let button = event.pressed_button.map_or(-1, button_code);
    pointer_payload(
        f32::from(event.position.x),
        f32::from(event.position.y),
        button,
        0,
        modifiers_code(event.modifiers),
    )
}

fn mouse_pressure_payload(event: &MousePressureEvent) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "{}\t{}\t{}\t{}\t{}",
        event.pressure,
        f32::from(event.position.x),
        f32::from(event.position.y),
        modifiers_code(event.modifiers),
        pressure_stage_code(&event.stage),
    );
    out
}

fn pressure_stage_code(stage: &gpui::PressureStage) -> i32 {
    match stage {
        gpui::PressureStage::Zero => 0,
        gpui::PressureStage::Normal => 1,
        gpui::PressureStage::Force => 2,
    }
}

fn scroll_payload(event: &ScrollWheelEvent) -> String {
    let (dx, dy) = match event.delta {
        ScrollDelta::Pixels(point) => (f32::from(point.x), f32::from(point.y)),
        ScrollDelta::Lines(point) => (point.x, point.y),
    };
    format!(
        "{}\t{}\t{}\t{}\t{}",
        f32::from(event.position.x),
        f32::from(event.position.y),
        dx,
        dy,
        modifiers_code(event.modifiers)
    )
}

fn key_payload(keystroke: &Keystroke, held: bool) -> String {
    format!(
        "{}\t{}\t{}",
        keystroke.key,
        modifiers_code(keystroke.modifiers),
        if held { 1 } else { 0 }
    )
}

fn click_payload(event: &ClickEvent) -> String {
    match event {
        ClickEvent::Mouse(click) => pointer_payload(
            f32::from(click.down.position.x),
            f32::from(click.down.position.y),
            button_code(click.down.button),
            click.down.click_count,
            modifiers_code(click.down.modifiers),
        ),
        _ => "0\t0\t-1\t0\t0".to_string(),
    }
}

/// Delivers a string payload to a managed handler.
fn invoke(host: &HostContext, token: u64, payload: &str) {
    if let Some(callback) = host.callbacks.invoke {
        // SAFETY: the managed callback copies anything it keeps; `payload` is
        // live for the call.
        unsafe {
            let _ = callback(
                host.session_id,
                token,
                CALLBACK_VALUE_STRING,
                0.0,
                payload.as_ptr(),
                payload.len() as u32,
            );
        }
    }
}

/// Delivers a boolean to a managed handler.
fn invoke_bool(host: &HostContext, token: u64, value: bool) {
    if let Some(callback) = host.callbacks.invoke {
        // SAFETY: a boolean carries no pointer.
        unsafe {
            let _ = callback(
                host.session_id,
                token,
                CALLBACK_VALUE_BOOLEAN,
                if value { 1.0 } else { 0.0 },
                std::ptr::null(),
                0,
            );
        }
    }
}

/// Fires one event at its managed token, then repaints if the event is discrete.
fn fire(
    event: ElementEvent,
    host: &HostContext,
    token: u64,
    payload: impl FnOnce() -> String,
    cx: &mut App,
) {
    invoke(host, token, &payload());
    if event.auto_invalidate() {
        (host.invalidate)(cx);
    }
}

/// Binds the subscribed stateless events onto an interactive element.
pub fn bind_stateless<E: InteractiveElement>(
    mut element: E,
    events: &ElementEvents,
    host: &HostContext,
) -> E {
    if let Some(token) = events.get(ElementEvent::MouseDown) {
        let host = host.clone();
        element = element.on_any_mouse_down(move |event, _, cx| {
            fire(
                ElementEvent::MouseDown,
                &host,
                token,
                || mouse_down_payload(event),
                cx,
            )
        });
    }
    if let Some(token) = events.get(ElementEvent::MouseUp) {
        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            let host = host.clone();
            element = element.on_mouse_up(button, move |event, _, cx| {
                fire(
                    ElementEvent::MouseUp,
                    &host,
                    token,
                    || mouse_up_payload(event),
                    cx,
                )
            });
        }
    }
    if let Some(token) = events.get(ElementEvent::MouseDownOut) {
        let host = host.clone();
        element = element.on_mouse_down_out(move |event, _, cx| {
            fire(
                ElementEvent::MouseDownOut,
                &host,
                token,
                || mouse_down_payload(event),
                cx,
            )
        });
    }
    if let Some(token) = events.get(ElementEvent::MouseUpOut) {
        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            let host = host.clone();
            element = element.on_mouse_up_out(button, move |event, _, cx| {
                fire(
                    ElementEvent::MouseUpOut,
                    &host,
                    token,
                    || mouse_up_payload(event),
                    cx,
                )
            });
        }
    }
    if let Some(token) = events.get(ElementEvent::MouseMove) {
        let host = host.clone();
        element = element.on_mouse_move(move |event, _, cx| {
            fire(
                ElementEvent::MouseMove,
                &host,
                token,
                || mouse_move_payload(event),
                cx,
            )
        });
    }
    if let Some(token) = events.get(ElementEvent::MousePressure) {
        let host = host.clone();
        element = element.on_mouse_pressure(move |event, _, cx| {
            fire(
                ElementEvent::MousePressure,
                &host,
                token,
                || mouse_pressure_payload(event),
                cx,
            )
        });
    }
    if let Some(token) = events.get(ElementEvent::ScrollWheel) {
        let host = host.clone();
        element = element.on_scroll_wheel(move |event, _, cx| {
            fire(
                ElementEvent::ScrollWheel,
                &host,
                token,
                || scroll_payload(event),
                cx,
            )
        });
    }
    if let Some(token) = events.get(ElementEvent::KeyDown) {
        let host = host.clone();
        element = element.on_key_down(move |event, _, cx| {
            fire(
                ElementEvent::KeyDown,
                &host,
                token,
                || key_payload(&event.keystroke, event.is_held),
                cx,
            )
        });
    }
    if let Some(token) = events.get(ElementEvent::KeyUp) {
        let host = host.clone();
        element = element.on_key_up(move |event, _, cx| {
            fire(
                ElementEvent::KeyUp,
                &host,
                token,
                || key_payload(&event.keystroke, false),
                cx,
            )
        });
    }
    element
}

/// Binds every subscribed event onto a stateful element (after `.id(...)`).
pub fn bind_stateful<E: StatefulInteractiveElement>(
    mut element: E,
    events: &ElementEvents,
    host: &HostContext,
) -> E {
    if let Some(token) = events.get(ElementEvent::Click) {
        let host = host.clone();
        element = element.on_click(move |_event, _, cx| {
            if let Some(click) = host.callbacks.click {
                // SAFETY: the managed callback retires only its own handler.
                unsafe {
                    let _ = click(host.session_id, token);
                }
            }
            (host.invalidate)(cx);
        });
    }
    if let Some(token) = events.get(ElementEvent::AuxClick) {
        let host = host.clone();
        element = element.on_aux_click(move |event, _, cx| {
            fire(
                ElementEvent::AuxClick,
                &host,
                token,
                || click_payload(event),
                cx,
            )
        });
    }
    if let Some(token) = events.get(ElementEvent::Hover) {
        let host = host.clone();
        element = element.on_hover(move |hovered, _, cx| {
            invoke_bool(&host, token, *hovered);
            (host.invalidate)(cx);
        });
    }
    bind_stateless(element, events, host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names_round_trip_the_event_names() {
        for name in [
            "on_click",
            "on_aux_click",
            "on_hover",
            "on_mouse_down",
            "on_mouse_up",
            "on_mouse_down_out",
            "on_mouse_up_out",
            "on_mouse_move",
            "on_mouse_pressure",
            "on_scroll_wheel",
            "on_key_down",
            "on_key_up",
        ] {
            assert!(
                ElementEvent::from_wire_name(name).is_some(),
                "`{name}` is not recognized"
            );
        }
        assert!(ElementEvent::from_wire_name("on_change").is_none());
    }

    #[test]
    fn only_click_hover_and_aux_click_need_an_element_id() {
        assert!(ElementEvent::Click.needs_element_id());
        assert!(ElementEvent::AuxClick.needs_element_id());
        assert!(ElementEvent::Hover.needs_element_id());
        assert!(!ElementEvent::MouseMove.needs_element_id());
        assert!(!ElementEvent::KeyDown.needs_element_id());
    }

    #[test]
    fn only_move_and_scroll_skip_the_automatic_repaint() {
        assert!(!ElementEvent::MouseMove.auto_invalidate());
        assert!(!ElementEvent::ScrollWheel.auto_invalidate());
        assert!(ElementEvent::Click.auto_invalidate());
        assert!(ElementEvent::MouseDown.auto_invalidate());
        assert!(ElementEvent::KeyUp.auto_invalidate());
    }

    #[test]
    fn the_last_handler_for_an_event_wins() {
        let mut events = ElementEvents::default();
        events.push(ElementEvent::MouseDown, 1);
        events.push(ElementEvent::MouseDown, 2);
        assert_eq!(events.get(ElementEvent::MouseDown), Some(2));
        assert_eq!(events.get(ElementEvent::Click), None);
    }
}
