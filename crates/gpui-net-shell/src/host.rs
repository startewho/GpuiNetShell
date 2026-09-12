//! The native application host: one GPUI application, one window, and the
//! any-thread ingress that lets the managed host ask for a re-render or open an
//! overlay.
//!
//! The view lives in [`crate::view`] and the overlay host in [`crate::root`].
//! This module owns the event loop, the window, and the session channel.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use gpui::prelude::*;
use gpui::{px, size, App, Bounds, TitlebarOptions, WindowBounds, WindowOptions};
use gpui_base::Placement;

use crate::abi::GpuiNetCallbacks;
use crate::root::{NotificationLevel, PopupItem, Root};
use crate::schema::{STATUS_INVALID_ARGUMENT, STATUS_OK};
use crate::view::ShellView;

const STATUS_NO_SESSION: i32 = -20;
const STATUS_INGRESS_FULL: i32 = -21;
const STATUS_INGRESS_POISONED: i32 = -22;

/// A managed request delivered on the GPUI thread.
enum Command {
    Invalidate,
    OpenPopup {
        items: Vec<PopupItem>,
    },
    OpenDialog {
        title: String,
        body: String,
    },
    CloseDialog,
    OpenSheet {
        placement: Placement,
        title: String,
        body: String,
    },
    CloseSheet,
    Notify {
        message: String,
        level: NotificationLevel,
    },
}

type Ingress = Mutex<HashMap<u64, async_channel::Sender<Command>>>;

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

fn send(session_id: u64, command: Command) -> i32 {
    let Ok(sessions) = ingress().lock() else {
        return STATUS_INGRESS_POISONED;
    };
    match sessions.get(&session_id) {
        Some(sender) if sender.try_send(command).is_ok() => STATUS_OK,
        Some(_) => STATUS_INGRESS_FULL,
        None => STATUS_NO_SESSION,
    }
}

/// Requests a re-render of `session_id`'s view from any thread.
pub fn invalidate(session_id: u64) -> i32 {
    send(session_id, Command::Invalidate)
}

/// Opens the session's popup with the given items from any thread.
pub fn open_popup(session_id: u64, items: Vec<PopupItem>) -> i32 {
    send(session_id, Command::OpenPopup { items })
}

/// Opens a dialog with the given title and body from any thread.
pub fn open_dialog(session_id: u64, title: String, body: String) -> i32 {
    send(session_id, Command::OpenDialog { title, body })
}

/// Closes the topmost dialog from any thread.
pub fn close_dialog(session_id: u64) -> i32 {
    send(session_id, Command::CloseDialog)
}

/// Opens a sheet on the given edge from any thread.
pub fn open_sheet(session_id: u64, placement: Placement, title: String, body: String) -> i32 {
    send(
        session_id,
        Command::OpenSheet {
            placement,
            title,
            body,
        },
    )
}

/// Closes the sheet from any thread.
pub fn close_sheet(session_id: u64) -> i32 {
    send(session_id, Command::CloseSheet)
}

/// Posts a notification from any thread.
pub fn push_notification(session_id: u64, message: String, level: NotificationLevel) -> i32 {
    send(session_id, Command::Notify { message, level })
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
            let root = cx.new(|cx| Root::new(view, application_id, callbacks, cx));
            let weak_root = root.downgrade();

            // Native menu selection dispatches a `ManagedMenuAction`; route it to
            // the managed callback token it carries, then request a re-render.
            cx.on_action({
                let weak_root = weak_root.clone();
                move |action: &crate::menu_action::ManagedMenuAction, cx| {
                    if let Some(click) = callbacks.click {
                        // SAFETY: the managed callback copies anything it keeps.
                        unsafe {
                            let _ = click(application_id, action.token());
                        }
                    }
                    let _ = weak_root.update(cx, |root, cx| root.invalidate_view(cx));
                }
            });

            // Any-thread commands are delivered here, on the GPUI thread, with
            // the window's current handle so overlay operations are
            // window-scoped.
            let (sender, receiver) = async_channel::bounded(64);
            if let Ok(mut sessions) = ingress().lock() {
                sessions.insert(application_id, sender);
            }

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
                move |window, cx| cx.new(|cx| gpui_component::Root::new(root, window, cx)),
            );
            let window = match opened {
                Ok(window) => window,
                Err(error) => {
                    eprintln!("gpui-net-shell: failed to open the window: {error}");
                    cx.activate(true);
                    return;
                }
            };

            cx.spawn(async move |cx| {
                while let Ok(command) = receiver.recv().await {
                    cx.update(|cx| {
                        let _ = window.update(cx, |_root, window, cx| {
                            let _ = weak_root.update(cx, |root, cx| match command {
                                Command::Invalidate => root.invalidate_view(cx),
                                Command::OpenPopup { items } => root.open_popup(items, window, cx),
                                Command::OpenDialog { title, body } => {
                                    root.open_message_dialog(title, body, window, cx)
                                }
                                Command::CloseDialog => {
                                    root.close_dialog(window, cx);
                                }
                                Command::OpenSheet {
                                    placement,
                                    title,
                                    body,
                                } => root.open_message_sheet(placement, title, body, window, cx),
                                Command::CloseSheet => {
                                    root.close_sheet(window, cx);
                                }
                                Command::Notify { message, level } => {
                                    root.push_notification(message, level, window, cx)
                                }
                            });
                        });
                    });
                }
            })
            .detach();

            cx.activate(true);
        });

    STATUS_OK
}
