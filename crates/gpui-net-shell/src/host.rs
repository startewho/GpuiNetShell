//! The native application host: one GPUI application, one window, and the
//! any-thread ingress that lets the managed host ask for a re-render or open an
//! overlay.
//!
//! The view lives in [`crate::view`] and the overlay host in [`crate::root`].
//! This module owns the event loop, the window, and the session channel.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
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
/// `configure` flag: keep scrollbars visible instead of auto-hiding them.
const FLAG_ALWAYS_SHOW_SCROLLBARS: u32 = 2;

/// The window bounds every new window opens at.
const DEFAULT_WINDOW_WIDTH: f32 = 900.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 600.0;

/// A managed request delivered on the GPUI thread.
enum Command {
    Invalidate,
    OpenPopup {
        items: Vec<PopupItem>,
    },
    SetTheme {
        mode: u32,
        colors: String,
    },
    /// Opens a new top-level window for `session`, delivered to the parent
    /// window's task so `cx.open_window` runs on the GPUI thread.
    OpenChild {
        session: u64,
        flags: u32,
    },
    /// Closes the window belonging to `session`.
    Close,
    /// Repaints the retained entity subtree `entity_id` in `session`.
    NotifyEntity {
        entity_id: u64,
    },
}

/// Per-session window options, set before the window opens.
fn window_configs() -> &'static Mutex<HashMap<u64, u32>> {
    static CONFIGS: OnceLock<Mutex<HashMap<u64, u32>>> = OnceLock::new();
    CONFIGS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Per-session OS window titles, set before the window opens.
fn window_titles() -> &'static Mutex<HashMap<u64, String>> {
    static TITLES: OnceLock<Mutex<HashMap<u64, String>>> = OnceLock::new();
    TITLES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The recorded title for `session`, or `None` when it was never set.
fn session_title(session: u64) -> Option<String> {
    window_titles()
        .lock()
        .ok()
        .and_then(|titles| titles.get(&session).cloned())
}

/// Allocates child session ids. The primary session uses the managed
/// application id; children start above it and never collide.
fn next_session_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    // Start high enough to clear the primary session's numeric id and grow
    // monotonically.
    NEXT.fetch_add(1, Ordering::Relaxed) + 0x1_0000_0000
}

/// The bundle's icons plus a filesystem fallback, so `Image("C:\\a.png")`
/// (and any relative path) resolves. gpui's asset cache owns decoded images
/// and releases them once no view references them.
struct FileAssets;

impl gpui::AssetSource for FileAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        // The bundled source returns an error for a miss (not `Ok(None)`), so a
        // miss must fall through to the filesystem rather than fail the load.
        if let Ok(Some(bytes)) = gpui_kit_assets::Assets.load(path) {
            return Ok(Some(bytes));
        }
        if path.is_empty() {
            return Ok(None);
        }
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(std::borrow::Cow::Owned(bytes))),
            Err(_) => Ok(None),
        }
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        gpui_kit_assets::Assets.list(path)
    }
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

/// Records the OS window title for `session` before it opens.
pub fn set_window_title(session_id: u64, title: String) -> i32 {
    if let Ok(mut titles) = window_titles().lock() {
        titles.insert(session_id, title);
    }
    STATUS_OK
}

/// Applies a theme (mode and optional color overrides) from any thread.
pub fn set_theme(session_id: u64, mode: u32, colors: String) -> i32 {
    send(session_id, Command::SetTheme { mode, colors })
}

/// Opens a new top-level window owned by `parent_session` from any thread.
///
/// Allocates the child session id, records its window options, then asks the
/// parent window's task to open it on the GPUI thread. Returns the child
/// session id, or a negative status.
pub fn open_window(parent_session: u64, flags: u32, title: String) -> i64 {
    let session = next_session_id();
    if let Ok(mut configs) = window_configs().lock() {
        configs.insert(session, flags);
    }
    if let Ok(mut titles) = window_titles().lock() {
        titles.insert(session, title);
    }
    match send(parent_session, Command::OpenChild { session, flags }) {
        STATUS_OK => session as i64,
        status => status as i64,
    }
}

/// Closes the window belonging to `session` from any thread.
pub fn close_window(session_id: u64) -> i32 {
    send(session_id, Command::Close)
}

/// Repaints one entity subtree within `session` from any thread.
pub fn notify_entity(session_id: u64, entity_id: u64) -> i32 {
    send(session_id, Command::NotifyEntity { entity_id })
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

    let flags = session_flags(application_id);
    let always_show_scrollbars = flags & FLAG_ALWAYS_SHOW_SCROLLBARS != 0;

    gpui_platform::application()
        .with_assets(FileAssets)
        .run(move |cx: &mut App| {
            gpui_component::init(cx);
            if always_show_scrollbars {
                gpui_component::Theme::set_scrollbar_mode(
                    gpui_component::scroll::ScrollbarMode::Always,
                    cx,
                );
            }

            // Native menu selection dispatches a `ManagedMenuAction`; route it to
            // the managed callback token it carries. The action names its owning
            // session, so one global listener serves every window. The managed
            // handler requests its own repaint through `invalidate`.
            cx.on_action(move |action: &crate::menu_action::ManagedMenuAction, _cx| {
                if let Some(click) = callbacks.click {
                    // SAFETY: the managed callback copies anything it keeps.
                    unsafe {
                        let _ = click(action.session(), action.token());
                    }
                }
            });

            if let Err(error) = open_managed_window(cx, application_id, callbacks, flags) {
                eprintln!("gpui-net-shell: failed to open the window: {error}");
            }

            cx.activate(true);
        });

    STATUS_OK
}

/// Records the window options for `session` and returns them.
fn session_flags(session: u64) -> u32 {
    window_configs()
        .lock()
        .ok()
        .and_then(|configs| configs.get(&session).copied())
        .unwrap_or(0)
}

/// Opens one managed window and installs its command ingress task.
fn open_managed_window(
    cx: &mut App,
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    flags: u32,
) -> Result<(), String> {
    let custom_titlebar = flags & FLAG_CUSTOM_TITLEBAR != 0;
    let title = session_title(session_id).unwrap_or_else(|| "GpuiNetShell".to_string());
    let registry = std::rc::Rc::new(crate::components::catalog());
    let view = cx.new(|cx| ShellView::new(session_id, callbacks, registry, cx));
    let root = cx.new(|cx| {
        Root::new(
            view,
            session_id,
            callbacks,
            custom_titlebar,
            title.clone(),
            cx,
        )
    });
    let weak_root = root.downgrade();

    let (sender, receiver) = async_channel::bounded(64);
    if let Ok(mut sessions) = ingress().lock() {
        sessions.insert(session_id, sender);
    }

    let bounds = Bounds::centered(
        None,
        size(px(DEFAULT_WINDOW_WIDTH), px(DEFAULT_WINDOW_HEIGHT)),
        cx,
    );
    let base = if custom_titlebar {
        let mut options = gpui_component::TitleBar::window_options();
        let mut titlebar = options.titlebar.take().unwrap_or_default();
        titlebar.title = Some(title.clone().into());
        options.titlebar = Some(titlebar);
        options
    } else {
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(title.clone().into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    };
    let window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..base
            },
            move |window, cx| cx.new(|cx| gpui_component::Root::new(root, window, cx)),
        )
        .map_err(|error| error.to_string())?;

    // When the OS closes this window, release its session and tell managed code.
    // GPUI quits on its own once the last window closes, so children can close
    // without ending the process.
    {
        let window_id = window.window_id();
        cx.on_window_closed(move |_cx, closed_id| {
            if closed_id != window_id {
                return;
            }
            unregister_session(session_id);
            if let Some(closed) = callbacks.window_closed {
                // SAFETY: acknowledgement only; managed code copies what it keeps.
                unsafe {
                    let _ = closed(session_id, STATUS_OK);
                }
            }
        })
        .detach();
    }

    // Any-thread commands are delivered here, on the GPUI thread, with the
    // window's current handle so overlay operations are window-scoped.
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
                        Command::OpenChild { session, flags } => {
                            if let Err(error) = open_managed_window(cx, session, callbacks, flags) {
                                eprintln!("gpui-net-shell: failed to open a child window: {error}");
                            }
                        }
                        Command::Close => {
                            window.remove_window();
                        }
                        Command::NotifyEntity { entity_id } => {
                            root.notify_entity(entity_id, cx);
                        }
                    });
                });
            });
        }
    })
    .detach();

    Ok(())
}
