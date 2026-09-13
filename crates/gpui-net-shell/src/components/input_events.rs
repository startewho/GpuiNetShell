//! Shared focus/change event forwarding for the retained input family.
//!
//! `Input`, `Textarea`, `NumberInput`, and `Editor` all emit
//! `gpui_component::input::InputEvent`; `OtpInput` emits its own `OtpEvent`.
//! Both carry `Change`, `Focus`, and `Blur`. This module keeps the callback
//! cells and installs the entity subscription once, so each component only
//! declares which managed callbacks it exposes.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{Entity, EventEmitter, Subscription, Window};
use gpui_component::input::{InputEvent, OtpEvent};

use crate::registry::{ComponentCallback, ComponentCallbackArgument};
/// The managed callbacks an input component exposes.
#[derive(Clone, Default)]
pub(crate) struct InputCallbacks {
    pub change: Option<ComponentCallback>,
    pub focus: Option<ComponentCallback>,
    pub blur: Option<ComponentCallback>,
}

/// The retained cells behind [`InputCallbacks`], shared with the subscription.
#[derive(Clone)]
pub(crate) struct RetainedInputCallbacks {
    change: Rc<RefCell<Option<ComponentCallback>>>,
    focus: Rc<RefCell<Option<ComponentCallback>>>,
    blur: Rc<RefCell<Option<ComponentCallback>>>,
}

impl RetainedInputCallbacks {
    pub(crate) fn new(initial: InputCallbacks) -> Self {
        Self {
            change: Rc::new(RefCell::new(initial.change)),
            focus: Rc::new(RefCell::new(initial.focus)),
            blur: Rc::new(RefCell::new(initial.blur)),
        }
    }

    /// Replaces the callbacks for the current render.
    pub(crate) fn set(&self, callbacks: InputCallbacks) {
        *self.change.borrow_mut() = callbacks.change;
        *self.focus.borrow_mut() = callbacks.focus;
        *self.blur.borrow_mut() = callbacks.blur;
    }

    /// Subscribes to a state that emits `InputEvent`.
    pub(crate) fn subscribe<S, V>(
        &self,
        window: &mut Window,
        cx: &mut gpui::App,
        state: &Entity<S>,
        value_of: V,
    ) -> Subscription
    where
        S: EventEmitter<InputEvent> + 'static,
        V: Fn(&S) -> String + 'static,
    {
        let change = self.change.clone();
        let focus = self.focus.clone();
        let blur = self.blur.clone();
        window.subscribe(
            state,
            cx,
            move |state: Entity<S>, event: &InputEvent, window, cx| match event {
                InputEvent::Change => {
                    if let Some(callback) = change.borrow().clone() {
                        callback.invoke_with(
                            "input on_change callback failed",
                            &[ComponentCallbackArgument::String(value_of(state.read(cx)))],
                            window,
                            cx,
                        );
                    }
                }
                InputEvent::Focus => invoke(&focus, "input on_focus callback failed", window, cx),
                InputEvent::Blur => invoke(&blur, "input on_blur callback failed", window, cx),
                InputEvent::PressEnter { .. } => {}
            },
        )
    }

    /// Subscribes to a state that emits `OtpEvent`.
    pub(crate) fn subscribe_otp(
        &self,
        window: &mut Window,
        cx: &mut gpui::App,
        state: &Entity<gpui_component::input::OtpState>,
        value_of: impl Fn(&gpui_component::input::OtpState) -> String + 'static,
    ) -> Subscription {
        let change = self.change.clone();
        let focus = self.focus.clone();
        let blur = self.blur.clone();
        window.subscribe(
            state,
            cx,
            move |state, event: &OtpEvent, window, cx| match event {
                OtpEvent::Change => {
                    if let Some(callback) = change.borrow().clone() {
                        callback.invoke_with(
                            "input on_change callback failed",
                            &[ComponentCallbackArgument::String(value_of(state.read(cx)))],
                            window,
                            cx,
                        );
                    }
                }
                OtpEvent::Complete => {}
                OtpEvent::Focus => invoke(&focus, "input on_focus callback failed", window, cx),
                OtpEvent::Blur => invoke(&blur, "input on_blur callback failed", window, cx),
            },
        )
    }
}

fn invoke(
    callback: &Rc<RefCell<Option<ComponentCallback>>>,
    label: &str,
    window: &mut Window,
    cx: &mut gpui::App,
) {
    if let Some(callback) = callback.borrow().clone() {
        callback.invoke_with(label, &[], window, cx);
    }
}
