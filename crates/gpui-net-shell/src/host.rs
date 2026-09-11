//! The native application host: one GPUI application, one window, one retained
//! managed snapshot.
//!
//! A clean repaint materializes the retained [`Snapshot`] without crossing into
//! managed code. Managed `render` runs only when the view is dirty, which is
//! set on first mount and whenever an event binding requests a re-render.

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    div, px, size, AnyElement, App, Bounds, Context, IntoElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions,
};

use crate::abi::{GpuiNetArena, GpuiNetCallbacks};
use crate::materialize::materialize;
use crate::registry::{FrozenComponentRegistry, HostContext, Invalidate};
use crate::schema::{STATUS_INVALID_ARGUMENT, STATUS_OK, STATUS_STALE_REVISION};
use crate::snapshot::Snapshot;

/// A window root that owns the last accepted managed snapshot.
struct ShellView {
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    registry: FrozenComponentRegistry,
    snapshot: Option<Snapshot>,
    revision: u64,
    dirty: bool,
    error: Option<String>,
}

impl ShellView {
    fn new(
        session_id: u64,
        callbacks: GpuiNetCallbacks,
        registry: FrozenComponentRegistry,
    ) -> Self {
        Self {
            session_id,
            callbacks,
            registry,
            snapshot: None,
            revision: 0,
            dirty: true,
            error: None,
        }
    }

    /// Pulls a new description from managed code if this view is dirty.
    fn refresh(&mut self) {
        if !self.dirty {
            return;
        }
        self.dirty = false;

        let Some(render) = self.callbacks.render else {
            self.error = Some("The host has no render callback.".into());
            return;
        };

        let mut arena = GpuiNetArena::empty();
        let mut root = 0u32;
        let mut revision = 0u64;
        // SAFETY: managed code fills the borrowed descriptor and returns; the
        // buffers it names are only read during `Snapshot::decode` below.
        let status = unsafe { render(self.session_id, &mut arena, &mut root, &mut revision) };
        if status != STATUS_OK {
            self.error = Some(format!("Managed render failed with status {status}."));
            return;
        }
        if revision <= self.revision {
            self.error = Some("Managed render returned a stale revision.".into());
            self.complete(revision, STATUS_STALE_REVISION);
            return;
        }

        match Snapshot::decode(&arena, root) {
            Ok(snapshot) => {
                self.snapshot = Some(snapshot);
                self.revision = revision;
                self.error = None;
                self.complete(revision, STATUS_OK);
            }
            Err(code) => {
                self.error = Some(format!("Native snapshot decode failed with status {code}."));
                self.complete(revision, code);
            }
        }
    }

    fn complete(&self, revision: u64, status: i32) {
        if let Some(complete) = self.callbacks.render_completed {
            // SAFETY: acknowledgement only; no managed arena borrow is live.
            unsafe {
                let _ = complete(self.session_id, revision, status);
            }
        }
    }

    fn content(&mut self, invalidate: &Invalidate) -> AnyElement {
        if let Some(error) = &self.error {
            return div().p(px(16.0)).child(error.clone()).into_any_element();
        }

        let materialized = match self.snapshot.as_ref() {
            Some(snapshot) => {
                let host = HostContext {
                    session_id: self.session_id,
                    callbacks: self.callbacks,
                    invalidate: invalidate.clone(),
                };
                materialize(&self.registry, snapshot, &host)
            }
            None => Err("Waiting for the managed host to publish a view.".to_string()),
        };

        match materialized {
            Ok(element) => element,
            Err(message) => {
                self.error = Some(message.clone());
                div().p(px(16.0)).child(message).into_any_element()
            }
        }
    }
}

impl Render for ShellView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.refresh();

        let weak = cx.entity().downgrade();
        let invalidate: Invalidate = Rc::new(move |app: &mut App| {
            // Event dispatch runs on the GPUI thread, so the view is reachable
            // synchronously without an ingress queue.
            let _ = weak.update(app, |view, cx| {
                view.dirty = true;
                cx.notify();
            });
        });

        let content = self.content(&invalidate);
        div().size_full().bg(gpui::rgba(0xFFFFFFFF)).child(content)
    }
}

/// Runs one native application until its window closes. Blocking.
pub fn run(application_id: u64, callbacks: GpuiNetCallbacks) -> i32 {
    if callbacks.render.is_none() || callbacks.render_completed.is_none() {
        return STATUS_INVALID_ARGUMENT;
    }
    if let Some(started) = callbacks.application_started {
        // SAFETY: managed startup probe; any data it retains is copied.
        let status = unsafe { started(application_id) };
        if status != STATUS_OK {
            return status;
        }
    }

    gpui_platform::application()
        .with_assets(())
        .run(move |cx: &mut App| {
            gpui_component::init(cx);

            let registry = crate::components::catalog();
            let bounds = Bounds::centered(None, size(px(900.0), px(600.0)), cx);
            let opened = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("GpuiNetShell".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                move |window, cx| {
                    let view = cx.new(|_| ShellView::new(application_id, callbacks, registry));
                    cx.new(|cx| gpui_component::Root::new(view, window, cx))
                },
            );
            if let Err(error) = opened {
                eprintln!("gpui-net-shell: failed to open the window: {error}");
            }
            cx.activate(true);
        });

    STATUS_OK
}
