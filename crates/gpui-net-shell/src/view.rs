//! The native view, modeled on `gpui-shell`'s `view.rs`.
//!
//! A [`ShellView`] owns the [`RenderSnapshot`] the managed host last published
//! and the state that decides when to ask for a new one:
//!
//! ```text
//! dirty render ─▶ publish snapshot ─▶ materialize
//! clean render ─────────────────────▶ materialize   (no managed call)
//! ```
//!
//! The managed `render` callback runs only when the view is dirty — first
//! mount, an event binding, or an explicit [`ShellView::refresh`] from the
//! managed side. A clean GPUI repaint replays the retained snapshot. When a
//! snapshot is replaced (or the view is dropped), its `retire_callbacks`
//! callback releases the event handlers that generation registered.

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    div, px, AnyElement, App, Context, IntoElement, MouseButton, Render, ScrollDelta, Window,
};

use crate::abi::{GpuiNetArena, GpuiNetCallbacks};
use crate::context::{EntityHosts, HostContext, Invalidate};
use crate::materialize::{materialize, prepare};
use crate::registry::FrozenComponentRegistry;
use crate::schema::{STATUS_INVALID_ARGUMENT, STATUS_OK};
use crate::snapshot::{RenderSnapshot, Snapshot};
use gpui_component::ActiveTheme as _;

/// A window root carrying a managed view.
pub struct ShellView {
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    registry: Rc<FrozenComponentRegistry>,
    /// The description the managed host last published.
    current: Option<RenderSnapshot>,
    revision: u64,
    dirty: bool,
    retired: bool,
    /// Retained entity subtrees for this window, keyed by managed entity id.
    /// Rebuilt on each render; `notify_entity` repaints one in place.
    entity_hosts: EntityHosts,
    /// The failure of the most recent build, if it failed. Held rather than
    /// re-derived so a broken render is not re-run every frame.
    error: Option<String>,
}

#[allow(dead_code)] // accessors used by tests and future host integrations
impl ShellView {
    pub fn new(
        session_id: u64,
        callbacks: GpuiNetCallbacks,
        registry: Rc<FrozenComponentRegistry>,
    ) -> Self {
        Self {
            session_id,
            callbacks,
            registry,
            current: None,
            revision: 0,
            dirty: true,
            retired: false,
            entity_hosts: EntityHosts::default(),
            error: None,
        }
    }

    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Marks the description as possibly out of date.
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    /// Invalidates and notifies: the managed host changed something the view
    /// reads, and a repaint is requested.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.invalidate();
        cx.notify();
    }

    /// Notifies one retained entity subtree so only that subtree repaints.
    ///
    /// The managed `Context::notify` arrives as a command carrying the entity
    /// id; the notifier was registered by the `EntityHost` materializer on the
    /// most recent render.
    pub fn notify_entity(&mut self, entity_id: u64, cx: &mut Context<Self>) {
        let notifier = self.entity_hosts.borrow().get(&entity_id).cloned();
        if let Some(notifier) = notifier {
            notifier(cx);
        }
    }

    /// The published description, if one has been built.
    pub fn snapshot(&self) -> Option<&RenderSnapshot> {
        self.current.as_ref()
    }

    /// Why the most recent build failed, if it did.
    pub fn build_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Whether the next render will call managed code.
    pub fn is_dirty(&self) -> bool {
        !self.retired && self.dirty
    }

    /// Makes a retained view inert; a later frame must not rebuild it.
    pub fn retire(&mut self) {
        self.retired = true;
        self.dirty = false;
        self.error = None;
        self.current = None;
    }

    /// Pulls a new description from the managed host.
    ///
    /// Transactional: a snapshot is swapped in only after the managed call
    /// returns and decodes successfully, so a failure leaves the previous
    /// description and its callbacks exactly as they were.
    fn rebuild(&mut self) {
        self.dirty = false;

        let Some(render) = self.callbacks.render else {
            self.error = Some("The host has no render callback.".into());
            return;
        };

        let generation = self.revision + 1;
        let mut arena = GpuiNetArena::empty();
        let mut root = 0u32;
        // SAFETY: managed code fills the borrowed descriptor and returns; the
        // buffers it names are only read during `Snapshot::decode` below.
        let status = unsafe { render(self.session_id, generation, &mut arena, &mut root) };
        if status != STATUS_OK {
            self.error = Some(format!("Managed render failed with status {status}."));
            self.complete(generation, status);
            return;
        }

        match Snapshot::decode(&arena, root) {
            Ok(mut snapshot) => {
                if let Err(message) = prepare(&self.registry, &mut snapshot) {
                    self.error = Some(message);
                    self.complete(generation, STATUS_INVALID_ARGUMENT);
                    return;
                }
                let frozen =
                    RenderSnapshot::new(self.session_id, generation, snapshot, self.callbacks);
                // Replacing `current` drops the snapshot it replaced, which
                // retires that generation's callbacks immediately. Nothing in
                // the host reads the previous description, so it is not kept.
                self.current = Some(frozen);
                self.revision = generation;
                self.error = None;
                self.complete(generation, STATUS_OK);
            }
            Err(code) => {
                self.error = Some(format!("Native snapshot decode failed with status {code}."));
                self.complete(generation, code);
            }
        }
    }

    fn complete(&self, generation: u64, status: i32) {
        if let Some(complete) = self.callbacks.render_completed {
            // SAFETY: acknowledgement only; no managed arena borrow is live.
            unsafe {
                let _ = complete(self.session_id, generation, status);
            }
        }
    }

    /// The subtree for the current description, with a banner when the most
    /// recent build failed over an earlier good snapshot.
    fn content(&mut self, host: &HostContext, window: &mut Window, cx: &mut App) -> AnyElement {
        // Entity notifiers are re-registered as each `EntityHost` materializes;
        // clear so an entity that left the tree stops being retained.
        if let Ok(mut hosts) = self.entity_hosts.try_borrow_mut() {
            hosts.clear();
        }
        let materialized = self
            .current
            .as_ref()
            .map(|snapshot| materialize(&self.registry, snapshot, host, window, cx));

        match (self.error.clone(), materialized) {
            (None, Some(Ok(element))) => element,
            (None, Some(Err(message))) => {
                eprintln!("gpui-net-shell: materialization failed: {message}");
                self.error = Some(message.clone());
                failure_surface(&message)
            }
            (None, None) => waiting_surface(),
            (Some(message), Some(Ok(element))) => div()
                .relative()
                .size_full()
                .child(element)
                .child(failure_surface(&message))
                .into_any_element(),
            (Some(message), _) => failure_surface(&message),
        }
    }
}

impl Render for ShellView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.retired {
            return div().into_any_element();
        }
        if self.is_dirty() {
            self.rebuild();
        }

        let weak = cx.entity().downgrade();
        let invalidate: Invalidate = Rc::new(move |app: &mut App| {
            // Event dispatch runs on the GPUI thread, so the view is reachable
            // synchronously without an ingress queue.
            let _ = weak.update(app, |view, cx| view.refresh(cx));
        });
        let host = HostContext {
            session_id: self.session_id,
            callbacks: self.callbacks,
            invalidate,
            entity_hosts: self.entity_hosts.clone(),
        };

        let content = self.content(&host, window, cx);
        let callbacks = self.callbacks;
        let session = self.session_id;
        div()
            .id("gpui-net-shell-root")
            .size_full()
            .bg(cx.theme().background)
            .on_mouse_down(MouseButton::Left, move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_MOUSE_DOWN,
                    modifiers_flags(event.modifiers),
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                    0.0,
                    "",
                );
            })
            .on_mouse_down(MouseButton::Right, move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_MOUSE_DOWN,
                    modifiers_flags(event.modifiers),
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                    1.0,
                    "",
                );
            })
            .on_mouse_down(MouseButton::Middle, move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_MOUSE_DOWN,
                    modifiers_flags(event.modifiers),
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                    2.0,
                    "",
                );
            })
            .on_mouse_up(MouseButton::Left, move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_MOUSE_UP,
                    modifiers_flags(event.modifiers),
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                    0.0,
                    "",
                );
            })
            .on_mouse_up(MouseButton::Right, move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_MOUSE_UP,
                    modifiers_flags(event.modifiers),
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                    1.0,
                    "",
                );
            })
            .on_mouse_up(MouseButton::Middle, move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_MOUSE_UP,
                    modifiers_flags(event.modifiers),
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                    2.0,
                    "",
                );
            })
            .on_mouse_move(move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_MOUSE_MOVE,
                    modifiers_flags(event.modifiers),
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                    0.0,
                    "",
                );
            })
            .on_scroll_wheel(move |event, _, _| {
                let (dx, dy) = match event.delta {
                    ScrollDelta::Pixels(point) => (f32::from(point.x), f32::from(point.y)),
                    ScrollDelta::Lines(point) => (point.x, point.y),
                };
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_SCROLL,
                    modifiers_flags(event.modifiers),
                    dx,
                    dy,
                    0.0,
                    "",
                );
            })
            .on_key_down(move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_KEY_DOWN,
                    modifiers_flags(event.keystroke.modifiers),
                    0.0,
                    0.0,
                    0.0,
                    &event.keystroke.key,
                );
            })
            .on_key_up(move |event, _, _| {
                emit_input(
                    callbacks,
                    session,
                    crate::schema::INPUT_KEY_UP,
                    modifiers_flags(event.keystroke.modifiers),
                    0.0,
                    0.0,
                    0.0,
                    &event.keystroke.key,
                );
            })
            .child(content)
            .into_any_element()
    }
}

/// Forwards one window input event to the managed host, if it subscribed.
#[allow(clippy::too_many_arguments)]
fn emit_input(
    callbacks: GpuiNetCallbacks,
    session_id: u64,
    kind: u32,
    flags: u32,
    a: f32,
    b: f32,
    c: f32,
    text: &str,
) {
    if let Some(input) = callbacks.input_event {
        // SAFETY: managed callback; it copies anything it keeps.
        unsafe {
            let _ = input(
                session_id,
                kind,
                flags,
                a,
                b,
                c,
                text.as_ptr(),
                text.len() as u32,
            );
        }
    }
}

fn modifiers_flags(modifiers: gpui::Modifiers) -> u32 {
    let mut flags = 0u32;
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

impl Drop for ShellView {
    fn drop(&mut self) {
        crate::host::unregister_session(self.session_id);
    }
}

fn failure_surface(message: &str) -> AnyElement {
    div()
        .p(px(16.0))
        .child(message.to_owned())
        .into_any_element()
}

fn waiting_surface() -> AnyElement {
    div()
        .p(px(16.0))
        .child("Waiting for the managed host to publish a view.")
        .into_any_element()
}
