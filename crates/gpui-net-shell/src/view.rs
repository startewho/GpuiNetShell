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
    div, px, AnyElement, App, Context, Entity, FocusHandle, IntoElement, MouseButton, Render,
    ScrollDelta, Window,
};

use crate::abi::{GpuiNetArena, GpuiNetCallbacks};
use crate::context::{
    new_row_cache, new_row_scratch, EntityHosts, HostContext, Invalidate, RowCache, RowScratch,
};
use crate::materialize::{materialize, prepare};
use crate::registry::{FrozenComponentRegistry, NodeFactory};
use crate::schema::{STATUS_INVALID_ARGUMENT, STATUS_OK};
use crate::snapshot::{RenderSnapshot, Snapshot};
use gpui_component::ActiveTheme as _;

/// A window root carrying a managed view.
pub struct ShellView {
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    registry: Rc<FrozenComponentRegistry>,
    /// The retained content subtree. Materialization happens there, not in
    /// `ShellView::render`, so GPUI can reuse the built tree.
    content: Entity<ContentHost>,
    /// The root's focus handle. Key events dispatch along the path to the
    /// focused node only, so the shell root is focused on mount; otherwise a
    /// window with nothing focused would never deliver keys to the host.
    focus_handle: FocusHandle,
    /// The description the managed host last published and is now displayed.
    current: Option<RenderSnapshot>,
    /// The description `current` replaced, held one generation longer.
    ///
    /// The retained `ContentHost` re-materializes a frame after `current` is
    /// swapped, so the on-screen element tree can still reference the previous
    /// generation's callback tokens for one frame. Keeping this snapshot alive
    /// keeps those tokens valid; without it a click on the not-yet-rebuilt tree
    /// resolves against a retired generation and is dropped.
    previous: Option<RenderSnapshot>,
    /// Root node of the custom title bar content, if the view supplied one. The
    /// title bar is materialized by `Root` from the same snapshot, on demand.
    titlebar: Option<u32>,
    /// The invalidation closure shared with the content host and title bar.
    invalidate: Invalidate,
    /// A reused buffer for `resolve_rows`.
    row_scratch: RowScratch,
    /// The fingerprint of `current`, used to keep it when a rebuild produces
    /// the same interface.
    displayed_fingerprint: Option<u64>,
    /// Whether the content entity must be updated on the next render.
    content_dirty: bool,
    revision: u64,
    dirty: bool,
    retired: bool,
    /// Retained entity subtrees for this window, keyed by managed entity id.
    /// Rebuilt on each render; `notify_entity` repaints one in place.
    entity_hosts: EntityHosts,
    /// Resolved row snapshots for the current description, so a repaint does
    /// not re-enter managed code for a `List`/`Select`/`VirtualList`. The
    /// content host holds the same map; `rebuild` clears it.
    row_cache: RowCache,
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
        cx: &mut Context<Self>,
    ) -> Self {
        let weak = cx.entity().downgrade();
        let invalidate: Invalidate = Rc::new(move |app: &mut App| {
            // Event dispatch runs on the GPUI thread, so the view is reachable
            // synchronously without an ingress queue.
            let _ = weak.update(app, |view, cx| view.refresh(cx));
        });
        let entity_hosts = EntityHosts::default();
        let row_scratch = new_row_scratch();
        let row_cache = new_row_cache();
        let focus_handle = cx.focus_handle();
        let content = cx.new(|_cx| ContentHost {
            session_id,
            callbacks,
            registry: registry.clone(),
            invalidate: invalidate.clone(),
            entity_hosts: entity_hosts.clone(),
            row_scratch: row_scratch.clone(),
            row_cache: row_cache.clone(),
            snapshot: None,
            error: None,
        });
        Self {
            session_id,
            callbacks,
            registry,
            content,
            focus_handle,
            current: None,
            previous: None,
            titlebar: None,
            invalidate,
            row_scratch,
            displayed_fingerprint: None,
            content_dirty: true,
            revision: 0,
            dirty: true,
            retired: false,
            entity_hosts,
            row_cache,
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

    /// Materializes the custom title bar content from the current description.
    /// `None` means the view supplied no title bar and `Root` should draw its
    /// default one.
    pub fn titlebar_element(&mut self, window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        let root = self.titlebar?;
        let snapshot = self.current.as_ref()?;
        let host = HostContext {
            session_id: self.session_id,
            callbacks: self.callbacks,
            invalidate: self.invalidate.clone(),
            entity_hosts: self.entity_hosts.clone(),
            row_scratch: self.row_scratch.clone(),
            row_cache: self.row_cache.clone(),
        };
        let factory = NodeFactory::new(&self.registry, snapshot.snapshot(), &host);
        factory.build(root, window, cx).ok()
    }

    /// The description the current one replaced, if any.
    pub fn previous_snapshot(&self) -> Option<&RenderSnapshot> {
        self.previous.as_ref()
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
        self.displayed_fingerprint = None;
        self.titlebar = None;
        self.previous = None;
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
            self.content_dirty = true;
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
            self.content_dirty = true;
            self.complete(generation, status);
            return;
        }

        match Snapshot::decode(&arena, root) {
            Ok(mut snapshot) => {
                if let Err(message) = prepare(&self.registry, &mut snapshot) {
                    self.error = Some(message);
                    self.content_dirty = true;
                    self.complete(generation, STATUS_INVALID_ARGUMENT);
                    return;
                }
                let fingerprint = snapshot.fingerprint();
                let node_count = snapshot.nodes.len() as u32;
                let titlebar = if arena.titlebar_root == crate::abi::NO_TITLEBAR_NODE
                    || arena.titlebar_root >= node_count
                {
                    None
                } else {
                    Some(arena.titlebar_root)
                };

                // An unchanged interface keeps the displayed description, which
                // also keeps its generation's callbacks alive for the retained
                // subtree. This generation's callbacks are retired instead.
                if self.error.is_none()
                    && self.current.is_some()
                    && self.displayed_fingerprint == Some(fingerprint)
                {
                    self.revision = generation;
                    self.complete(generation, STATUS_OK);
                    if let Some(retire) = self.callbacks.retire_callbacks {
                        // SAFETY: retiring one generation touches no arena.
                        unsafe {
                            let _ = retire(self.session_id, generation);
                        }
                    }
                    return;
                }

                let frozen =
                    RenderSnapshot::new(self.session_id, generation, snapshot, self.callbacks);
                // `previous` holds the description just replaced (and retires
                // the one before it). The retained `ContentHost` rebuilds a
                // frame later, so the on-screen tree may still reference the
                // replaced generation's tokens for one frame; holding it keeps
                // those tokens valid.
                self.previous = self.current.replace(frozen);
                self.titlebar = titlebar;
                self.displayed_fingerprint = Some(fingerprint);
                // A new description retires every callback token, so the
                // resolved-row cache for the old one is dead.
                self.row_cache.borrow_mut().clear();
                self.revision = generation;
                self.error = None;
                self.content_dirty = true;
                self.complete(generation, STATUS_OK);
            }
            Err(code) => {
                self.error = Some(format!("Native snapshot decode failed with status {code}."));
                self.content_dirty = true;
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
}

/// The retained content subtree.
///
/// Materialization lives here rather than in `ShellView::render`, so GPUI can
/// reuse the built element tree: the content re-renders only when a new
/// description is pushed, not on every `ShellView` repaint. Combined with the
/// structural fingerprint that lets `rebuild` keep an unchanged description, a
/// repaint that does not change the interface performs no native work.
struct ContentHost {
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    registry: Rc<FrozenComponentRegistry>,
    invalidate: Invalidate,
    entity_hosts: EntityHosts,
    row_scratch: RowScratch,
    row_cache: RowCache,
    snapshot: Option<RenderSnapshot>,
    error: Option<String>,
}

impl Render for ContentHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Entity notifiers are re-registered as each `EntityHost` materializes;
        // clear so an entity that left the tree stops being retained.
        if let Ok(mut hosts) = self.entity_hosts.try_borrow_mut() {
            hosts.clear();
        }
        let host = HostContext {
            session_id: self.session_id,
            callbacks: self.callbacks,
            invalidate: self.invalidate.clone(),
            entity_hosts: self.entity_hosts.clone(),
            row_scratch: self.row_scratch.clone(),
            row_cache: self.row_cache.clone(),
        };
        let materialized = self
            .snapshot
            .as_ref()
            .map(|snapshot| materialize(&self.registry, snapshot, &host, window, cx));

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
        if self.content_dirty {
            let snapshot = self.current.clone();
            let error = self.error.clone();
            self.content.update(cx, |content, cx| {
                content.snapshot = snapshot;
                content.error = error;
                cx.notify();
            });
            self.content_dirty = false;
        }

        let content = self.content.clone().into_any_element();
        let callbacks = self.callbacks;
        let session = self.session_id;
        // Focus the shell root so keyboard events dispatch along a path that
        // includes this view; with nothing focused they only reach the window
        // root and never reach the host.
        if window.focused(cx).is_none() {
            window.focus(&self.focus_handle, cx);
        }
        div()
            .id("gpui-net-shell-root")
            .track_focus(&self.focus_handle)
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
