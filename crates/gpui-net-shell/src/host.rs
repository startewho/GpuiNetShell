//! The native application host: one GPUI application, one window, and the
//! any-thread ingress that lets the managed host ask for a re-render or open an
//! overlay.
//!
//! The view lives in [`crate::view`] and the overlay host in [`crate::root`].
//! This module owns the event loop, the window, and the session channel.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use gpui::prelude::*;
use gpui::{px, size, App, Bounds, Hsla, TitlebarOptions, Window, WindowBounds, WindowOptions};

use crate::abi::GpuiNetCallbacks;
use crate::root::{PopupItem, Root};
use crate::schema::{STATUS_INVALID_ARGUMENT, STATUS_OK};
use crate::view::ShellView;

const STATUS_NO_SESSION: i32 = -20;
const STATUS_INGRESS_FULL: i32 = -21;
const STATUS_INGRESS_POISONED: i32 = -22;

/// `configure` flag: draw a custom title bar instead of the native one.
const FLAG_CUSTOM_TITLEBAR: u32 = 1;

/// A managed request delivered on the GPUI thread.
enum Command {
    Invalidate,
    OpenPopup { items: Vec<PopupItem> },
    SetTheme { mode: u32, colors: String },
}

/// Per-session window options, set before `run`.
fn window_configs() -> &'static Mutex<HashMap<u64, u32>> {
    static CONFIGS: OnceLock<Mutex<HashMap<u64, u32>>> = OnceLock::new();
    CONFIGS.get_or_init(|| Mutex::new(HashMap::new()))
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

/// Records per-session window options before the application runs.
pub fn configure(session_id: u64, flags: u32) -> i32 {
    if let Ok(mut configs) = window_configs().lock() {
        configs.insert(session_id, flags);
    }
    STATUS_OK
}

/// Applies a theme (mode and optional color overrides) from any thread.
pub fn set_theme(session_id: u64, mode: u32, colors: String) -> i32 {
    send(session_id, Command::SetTheme { mode, colors })
}

/// Applies a theme on the GPUI thread.
fn apply_theme(mode: u32, colors: &str, window: &mut Window, cx: &mut App) {
    use gpui_component::{Theme, ThemeMode};

    match mode {
        2 => Theme::sync_system_appearance(Some(window), cx),
        1 => Theme::change(ThemeMode::Dark, Some(window), cx),
        _ => Theme::change(ThemeMode::Light, Some(window), cx),
    }

    if !colors.trim().is_empty() {
        let theme = Theme::global_mut(cx);
        for line in colors.lines() {
            let Some((name, value)) = line.split_once('=') else {
                continue;
            };
            let Ok(color) = gpui_component::try_parse_color(value.trim()) else {
                continue;
            };
            apply_theme_color(theme, name.trim(), color);
        }
        Theme::sync_base(cx);
        window.refresh();
    }
}

/// Overrides one semantic theme color by name.
fn apply_theme_color(theme: &mut gpui_component::Theme, name: &str, color: Hsla) {
    match name {
        "background" => theme.background = color,
        "foreground" => theme.foreground = color,
        "primary" => theme.primary = color,
        "primary_foreground" => theme.primary_foreground = color,
        "secondary" => theme.secondary = color,
        "secondary_foreground" => theme.secondary_foreground = color,
        "muted" => theme.muted = color,
        "muted_foreground" => theme.muted_foreground = color,
        "accent" => theme.accent = color,
        "accent_foreground" => theme.accent_foreground = color,
        "border" => theme.border = color,
        "input" => theme.input = color,
        "ring" => theme.ring = color,
        "popover" => theme.popover = color,
        "popover_foreground" => theme.popover_foreground = color,
        "danger" => theme.danger = color,
        "danger_foreground" => theme.danger_foreground = color,
        "success" => theme.success = color,
        "success_foreground" => theme.success_foreground = color,
        "warning" => theme.warning = color,
        "warning_foreground" => theme.warning_foreground = color,
        "info" => theme.info = color,
        "info_foreground" => theme.info_foreground = color,
        "link" => theme.link = color,
        "selection" => theme.selection = color,
        "caret" => theme.caret = color,
        "list" => theme.colors.list = color,
        "list_even" => theme.list_even = color,
        "list_head" => theme.list_head = color,
        "sidebar" => theme.sidebar = color,
        "sidebar_foreground" => theme.sidebar_foreground = color,
        "sidebar_border" => theme.sidebar_border = color,
        "scrollbar" => theme.scrollbar = color,
        "scrollbar_thumb" => theme.scrollbar_thumb = color,
        "title_bar" => theme.title_bar = color,
        "title_bar_border" => theme.title_bar_border = color,
        _ => {}
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

    let flags = window_configs()
        .lock()
        .ok()
        .and_then(|configs| configs.get(&application_id).copied())
        .unwrap_or(0);
    let custom_titlebar = flags & FLAG_CUSTOM_TITLEBAR != 0;

    gpui_platform::application()
        .with_assets(gpui_kit_assets::Assets)
        .run(move |cx: &mut App| {
            gpui_component::init(cx);

            let registry = crate::components::catalog();
            let view = cx.new(|_| ShellView::new(application_id, callbacks, registry));
            let root = cx.new(|cx| Root::new(view, application_id, callbacks, custom_titlebar, cx));
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
            let base = if custom_titlebar {
                gpui_component::TitleBar::window_options()
            } else {
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("GpuiNetShell".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            };
            let opened = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..base
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
                                Command::Invalidate => {
                                    root.invalidate_view(cx);
                                    window.refresh();
                                }
                                Command::OpenPopup { items } => root.open_popup(items, window, cx),
                                Command::SetTheme { mode, colors } => {
                                    apply_theme(mode, &colors, window, cx);
                                    root.invalidate_view(cx);
                                    window.refresh();
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
