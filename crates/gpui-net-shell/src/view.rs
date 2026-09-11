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
use gpui::{div, px, rgba, AnyElement, App, Context, IntoElement, Render, Window};

use crate::abi::{GpuiNetArena, GpuiNetCallbacks};
use crate::materialize::materialize;
use crate::registry::{FrozenComponentRegistry, HostContext, Invalidate};
use crate::schema::STATUS_OK;
use crate::snapshot::{RenderSnapshot, Snapshot};

/// A window root carrying a managed view.
pub struct ShellView {
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    registry: FrozenComponentRegistry,
    /// The description the managed host last published.
    current: Option<RenderSnapshot>,
    /// The snapshot this one replaced, held one generation longer so an event
    /// dispatched against the previous frame still resolves.
    previous: Option<RenderSnapshot>,
    revision: u64,
    dirty: bool,
    retired: bool,
    /// The failure of the most recent build, if it failed. Held rather than
    /// re-derived so a broken render is not re-run every frame.
    error: Option<String>,
}

#[allow(dead_code)] // accessors used by tests and future host integrations
impl ShellView {
    pub fn new(
        session_id: u64,
        callbacks: GpuiNetCallbacks,
        registry: FrozenComponentRegistry,
    ) -> Self {
        Self {
            session_id,
            callbacks,
            registry,
            current: None,
            previous: None,
            revision: 0,
            dirty: true,
            retired: false,
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

    /// The published description, if one has been built.
    pub fn snapshot(&self) -> Option<&RenderSnapshot> {
        self.current.as_ref()
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
            Ok(snapshot) => {
                let frozen =
                    RenderSnapshot::new(self.session_id, generation, snapshot, self.callbacks);
                // Assigning through `previous` retires the snapshot before
                // last, releasing its generation's callbacks.
                self.previous = self.current.replace(frozen);
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
    fn content(&mut self, host: &HostContext) -> AnyElement {
        let materialized = self
            .current
            .as_ref()
            .map(|snapshot| materialize(&self.registry, snapshot, host));

        match (self.error.clone(), materialized) {
            (None, Some(Ok(element))) => element,
            (None, Some(Err(message))) => {
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
        };

        let content = self.content(&host);
        div()
            .size_full()
            .bg(rgba(0xFFFFFFFF))
            .child(content)
            .into_any_element()
    }
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
