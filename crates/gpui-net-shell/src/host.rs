//! The native application host: one GPUI application, one window, and the
//! any-thread ingress that lets the managed host ask for a re-render.
//!
//! The view itself lives in [`crate::view`]. This module only owns the event
//! loop, the window, and the session-to-view channel.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use gpui::prelude::*;
use gpui::{px, size, App, Bounds, TitlebarOptions, WindowBounds, WindowOptions};

use crate::abi::GpuiNetCallbacks;
use crate::schema::{STATUS_INVALID_ARGUMENT, STATUS_OK};
use crate::view::ShellView;

const STATUS_NO_SESSION: i32 = -20;
const STATUS_INGRESS_FULL: i32 = -21;
const STATUS_INGRESS_POISONED: i32 = -22;

type Ingress = Mutex<HashMap<u64, async_channel::Sender<()>>>;

fn ingress() -> &'static Ingress {
    static INGRESS: OnceLock<Ingress> = OnceLock::new();
    INGRESS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Removes a session's ingress entry; called when its view is dropped.
pub(crate) fn unregister_session(session_id: u64) {
    if let Ok(mut sessions) = ingress().lock() {
        sessions.remove(&session_id);
    }
}

/// Requests a re-render of `session_id`'s view from any thread.
///
/// The request is delivered on the GPUI thread, which marks the view dirty and
/// notifies it. Returns zero on success.
pub fn invalidate(session_id: u64) -> i32 {
    let Ok(sessions) = ingress().lock() else {
        return STATUS_INGRESS_POISONED;
    };
    match sessions.get(&session_id) {
        Some(sender) if sender.try_send(()).is_ok() => STATUS_OK,
        Some(_) => STATUS_INGRESS_FULL,
        None => STATUS_NO_SESSION,
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
            let view = cx.new(|_| ShellView::new(application_id, callbacks, registry));

            // Any-thread invalidations are delivered here, on the GPUI thread.
            let (sender, receiver) = async_channel::bounded(64);
            if let Ok(mut sessions) = ingress().lock() {
                sessions.insert(application_id, sender);
            }
            let weak = view.downgrade();
            cx.spawn(async move |cx| {
                while receiver.recv().await.is_ok() {
                    cx.update(|cx| {
                        let _ = weak.update(cx, |view, cx| view.refresh(cx));
                    });
                }
            })
            .detach();

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
                move |window, cx| cx.new(|cx| gpui_component::Root::new(view.clone(), window, cx)),
            );
            if let Err(error) = opened {
                eprintln!("gpui-net-shell: failed to open the window: {error}");
            }
            cx.activate(true);
        });

    STATUS_OK
}
