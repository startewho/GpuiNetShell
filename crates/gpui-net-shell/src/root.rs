//! The window-level overlay host, modeled on `gpui-shell`'s `root.rs`.
//!
//! [`Root`] owns the managed content view and the layers painted over it: one
//! sheet, a dialog stack, and notifications. Tooltips are handled by the
//! `gpui-component` window root this sits inside.
//!
//! # Stacking order
//!
//! Painted back to front, each layer above the one before it:
//!
//! 1. **Content** — the managed `ShellView`.
//! 2. **Sheet** (`8`) — at most one, anchored to a viewport edge. A sheet is a
//!    *place* in the window, so it sits below the dialog stack. The combobox
//!    popup is a sheet too: it opens a `Bottom` sheet whose content is the
//!    option menu.
//! 3. **Dialog stack** (`10 + index`) — in open order, oldest at the bottom.
//!    Only the topmost is interactive and only it draws a backdrop.
//! 4. **Notifications** (`100`).
//!
//! # Dismissal order
//!
//! - **Escape** closes the topmost dismissable dialog, otherwise the sheet
//!   (including a combobox popup).
//! - **Backdrop press** closes the topmost dialog only if it allows it, and
//!   closes the sheet when no dialog is open.
//!
//! # Focus
//!
//! Opening a dialog or sheet records what was focused and focuses the overlay;
//! closing it restores that handle. Because each open records what was focused
//! at the time, a stack restores focus through its own history.

use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    actions, deferred, div, px, rgba, AnyElement, AnyView, App, Context, Entity, FocusHandle,
    Global, IntoElement, KeyBinding, MouseButton, Render, WeakEntity, WeakFocusHandle, Window,
};
use gpui_base::Placement;
use gpui_component::button::Button;
use gpui_component::ActiveTheme as _;

use crate::abi::GpuiNetCallbacks;
use crate::view::ShellView;

actions!(gpui_net_shell_root, [Escape]);

/// The key context the root installs.
const CONTEXT: &str = "GpuiNetShellRoot";

/// How long a notification stays before it dismisses itself.
const NOTIFICATION_LIFETIME: Duration = Duration::from_secs(4);

/// Paint priority of the sheet layer: above content, below dialogs.
const SHEET_PRIORITY: usize = 8;
/// Paint priority of the dialog stack's first entry.
const DIALOG_PRIORITY: usize = 10;
/// Paint priority of the notification layer: above everything but the tooltip.
const NOTIFICATION_PRIORITY: usize = 100;

/// Severity of a notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    /// Maps a wire value onto a level, defaulting to `Info`.
    pub fn from_wire(value: u32) -> Self {
        match value {
            1 => Self::Success,
            2 => Self::Warning,
            3 => Self::Error,
            _ => Self::Info,
        }
    }
}

/// One selectable entry of a popup. `token` is the managed callback to run;
/// `token == 0` means the entry is inert.
#[derive(Clone)]
pub struct PopupItem {
    pub label: String,
    pub token: u64,
}

/// How a dialog may be dismissed. Both default to `true`. A dialog that refuses
/// dismissal is asking a question the user must answer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DialogOptions {
    escape_dismissable: bool,
    backdrop_dismissable: bool,
}

impl Default for DialogOptions {
    fn default() -> Self {
        Self {
            escape_dismissable: true,
            backdrop_dismissable: true,
        }
    }
}

#[allow(dead_code)] // builders used by host integrations
impl DialogOptions {
    pub fn escape_dismissable(mut self, dismissable: bool) -> Self {
        self.escape_dismissable = dismissable;
        self
    }

    pub fn backdrop_dismissable(mut self, dismissable: bool) -> Self {
        self.backdrop_dismissable = dismissable;
        self
    }

    pub fn is_escape_dismissable(self) -> bool {
        self.escape_dismissable
    }

    pub fn is_backdrop_dismissable(self) -> bool {
        self.backdrop_dismissable
    }
}

/// One open dialog, matching `gpui-shell`'s `ActiveDialog`.
struct ActiveDialog {
    content: AnyView,
    focus_handle: FocusHandle,
    /// What was focused when this dialog opened, to be restored when it closes.
    /// Weak, because the view that held it may be gone by then.
    restore_focus: Option<WeakFocusHandle>,
    options: DialogOptions,
}

/// The open sheet, matching `gpui-shell`'s `ActiveSheet`.
///
/// A combobox popup is a sheet like any other: `open_popup` opens a `Bottom`
/// sheet whose content is the option menu.
struct ActiveSheet {
    content: AnyView,
    placement: Placement,
    focus_handle: FocusHandle,
    restore_focus: Option<WeakFocusHandle>,
}

struct Notification {
    id: u64,
    message: String,
    level: NotificationLevel,
}

/// Marks that this `App` already has the root's key bindings.
struct KeyBindingsInstalled;

impl Global for KeyBindingsInstalled {}

/// A plain title/body body, used when the managed host opens a dialog or sheet
/// with text rather than a managed element tree.
struct MessageView {
    title: String,
    body: String,
}

impl Render for MessageView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        div()
            .flex_col()
            .gap(px(8.0))
            .child(div().text_size(px(16.0)).child(self.title.clone()))
            .child(div().text_color(muted).child(self.body.clone()))
    }
}

/// The option menu a combobox popup shows: one button per item, each running
/// the managed callback token and closing the sheet.
struct MenuView {
    items: Vec<PopupItem>,
    root: WeakEntity<Root>,
    session_id: u64,
    callbacks: GpuiNetCallbacks,
}

impl Render for MenuView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_col()
            .gap(px(4.0))
            .children(self.items.iter().enumerate().map(|(index, item)| {
                let token = item.token;
                let root = self.root.clone();
                let session_id = self.session_id;
                let callbacks = self.callbacks;
                Button::new(format!("root-menu-item-{index}"))
                    .label(item.label.clone())
                    .on_click(cx.listener(move |_this, _, window, cx| {
                        if token != 0 {
                            if let Some(click) = callbacks.click {
                                // SAFETY: the managed callback copies anything it keeps.
                                unsafe {
                                    let _ = click(session_id, token);
                                }
                            }
                        }
                        if let Some(root) = root.upgrade() {
                            root.update(cx, |root, cx| {
                                root.close_sheet(window, cx);
                                root.invalidate_view(cx);
                            });
                        }
                    }))
                    .into_any_element()
            }))
    }
}

/// The window-level overlay host: managed content, one sheet, dialogs,
/// notifications.
pub struct Root {
    view: Entity<ShellView>,
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    sheet: Option<ActiveSheet>,
    dialogs: Vec<ActiveDialog>,
    notifications: Vec<Notification>,
    next_notification: u64,
}

#[allow(dead_code)] // accessors used by tests and future integrations
impl Root {
    pub fn new(
        view: Entity<ShellView>,
        session_id: u64,
        callbacks: GpuiNetCallbacks,
        cx: &mut Context<Self>,
    ) -> Self {
        install_key_bindings(cx);
        Self {
            view,
            session_id,
            callbacks,
            sheet: None,
            dialogs: Vec::new(),
            notifications: Vec::new(),
            next_notification: 0,
        }
    }

    /// The managed content view.
    pub fn content(&self) -> &Entity<ShellView> {
        &self.view
    }

    /// Marks the content view dirty and requests a repaint.
    pub fn invalidate_view(&mut self, cx: &mut Context<Self>) {
        self.view.update(cx, |view, cx| view.refresh(cx));
    }

    /// How many dialogs are open. The topmost is the only interactive one.
    pub fn dialog_count(&self) -> usize {
        self.dialogs.len()
    }

    /// The dialog that owns focus, keyboard dismissal, and the backdrop.
    pub fn topmost_dialog(&self) -> Option<&AnyView> {
        self.dialogs.last().map(|dialog| &dialog.content)
    }

    /// The open sheet, if any. At most one sheet exists at a time.
    pub fn sheet(&self) -> Option<&AnyView> {
        self.sheet.as_ref().map(|sheet| &sheet.content)
    }

    // -- Popup (a sheet) -----------------------------------------------------

    /// Opens the option menu as a `Bottom` sheet, replacing any open sheet.
    ///
    /// The popup is not a layer of its own: it is the sheet, which is what
    /// `gpui-shell` has. Selecting an item runs its managed token and closes the
    /// sheet.
    pub fn open_popup(
        &mut self,
        items: Vec<PopupItem>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if items.is_empty() {
            self.close_sheet(window, cx);
            return;
        }
        let root = cx.entity().downgrade();
        let session_id = self.session_id;
        let callbacks = self.callbacks;
        let content: AnyView = cx
            .new(|_| MenuView {
                items,
                root,
                session_id,
                callbacks,
            })
            .into();
        self.open_sheet(Placement::Bottom, content, window, cx);
    }

    pub fn popup_open(&self) -> bool {
        self.sheet.is_some()
    }

    // -- Dialogs -------------------------------------------------------------

    /// Opens a dialog on top of the stack with default dismissal.
    pub fn open_dialog(&mut self, content: AnyView, window: &mut Window, cx: &mut Context<Self>) {
        self.open_dialog_with(content, DialogOptions::default(), window, cx);
    }

    /// Opens a dialog whose dismissal differs from the default.
    pub fn open_dialog_with(
        &mut self,
        content: AnyView,
        options: DialogOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let focus_handle = cx.focus_handle();
        let restore_focus = window.focused(cx).map(|handle| handle.downgrade());
        focus_handle.focus(window, cx);

        self.dialogs.push(ActiveDialog {
            content,
            focus_handle,
            restore_focus,
            options,
        });
        cx.notify();
    }

    /// Opens a plain title/body dialog, as the managed host requests one.
    pub fn open_message_dialog(
        &mut self,
        title: String,
        body: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let content: AnyView = cx.new(|_| MessageView { title, body }).into();
        self.open_dialog(content, window, cx);
    }

    /// Closes the topmost dialog and restores the focus it took.
    pub fn close_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(dialog) = self.dialogs.pop() else {
            return false;
        };
        restore_focus(dialog.restore_focus, window, cx);
        cx.notify();
        true
    }

    /// Closes every dialog at once, restoring focus to where the *first* dialog
    /// took it from. Leaves an open sheet alone.
    pub fn close_all_dialogs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.dialogs.is_empty() {
            return;
        }
        let restore = self
            .dialogs
            .first()
            .and_then(|dialog| dialog.restore_focus.clone());
        self.dialogs.clear();
        restore_focus(restore, window, cx);
        cx.notify();
    }

    // -- Sheet ---------------------------------------------------------------

    /// Opens a sheet on the given edge, replacing any sheet already open.
    ///
    /// Replacing rather than stacking keeps the focus record honest: the
    /// incoming sheet inherits the outgoing one's restore target.
    pub fn open_sheet(
        &mut self,
        placement: Placement,
        content: AnyView,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let focus_handle = cx.focus_handle();
        let restore_focus = self
            .sheet
            .take()
            .and_then(|sheet| sheet.restore_focus)
            .or_else(|| window.focused(cx).map(|handle| handle.downgrade()));
        focus_handle.focus(window, cx);

        self.sheet = Some(ActiveSheet {
            content,
            placement,
            focus_handle,
            restore_focus,
        });
        cx.notify();
    }

    /// Opens a plain title/body sheet, as the managed host requests one.
    pub fn open_message_sheet(
        &mut self,
        placement: Placement,
        title: String,
        body: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let content: AnyView = cx.new(|_| MessageView { title, body }).into();
        self.open_sheet(placement, content, window, cx);
    }

    /// Closes the sheet, returning whether one was open.
    pub fn close_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(sheet) = self.sheet.take() else {
            return false;
        };
        restore_focus(sheet.restore_focus, window, cx);
        cx.notify();
        true
    }

    // -- Notifications -------------------------------------------------------

    /// Posts a notification that dismisses itself after a short lifetime.
    ///
    /// `_window` is kept in the signature because overlay changes are
    /// window-scoped, matching shell's `push_toast`.
    pub fn push_notification(
        &mut self,
        message: String,
        level: NotificationLevel,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.next_notification;
        self.next_notification += 1;
        self.notifications.push(Notification { id, message, level });
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(NOTIFICATION_LIFETIME).await;
            let _ = this.update(cx, |this, cx| this.dismiss_notification(id, cx));
        })
        .detach();
    }

    /// Removes a notification by id.
    pub fn dismiss_notification(&mut self, id: u64, cx: &mut Context<Self>) {
        let before = self.notifications.len();
        self.notifications
            .retain(|notification| notification.id != id);
        if self.notifications.len() != before {
            cx.notify();
        }
    }

    pub fn notification_count(&self) -> usize {
        self.notifications.len()
    }

    /// Escape closes the topmost dismissable dialog, otherwise the sheet
    /// (including a combobox popup).
    fn on_escape(&mut self, _: &Escape, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(topmost) = self.dialogs.last() {
            if topmost.options.is_escape_dismissable() {
                self.close_dialog(window, cx);
            }
            return;
        }
        if self.sheet.is_some() {
            self.close_sheet(window, cx);
        }
    }

    // -- Layers --------------------------------------------------------------

    /// The sheet, anchored to a viewport edge. A combobox popup is this layer.
    fn sheet_layer(&self, _window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let sheet = self.sheet.as_ref()?;
        // An open dialog holds focus, so the sheet's overlay is inert then.
        let dismissable = self.dialogs.is_empty();
        let (surface, foreground, border, radius) = {
            let theme = cx.theme();
            (
                theme.popover,
                theme.popover_foreground,
                theme.border,
                theme.radius_lg,
            )
        };
        let placement = sheet.placement;

        let surface_element = div()
            .id("root-sheet")
            .occlude()
            .absolute()
            .flex_col()
            .gap(px(12.0))
            .map(|this| match placement {
                Placement::Left => this.top_0().bottom_0().left_0().w_1_3().border_r_1(),
                Placement::Right => this.top_0().bottom_0().right_0().w_1_3().border_l_1(),
                Placement::Top => this.left_0().right_0().top_0().h_1_3().border_b_1(),
                Placement::Bottom => this.left_0().right_0().bottom_0().h_1_3().border_t_1(),
            })
            .track_focus(&sheet.focus_handle)
            .bg(surface)
            .text_color(foreground)
            .border_color(border)
            .rounded(radius)
            .p(px(16.0))
            .child(sheet.content.clone())
            .child(
                div().flex().justify_end().child(
                    Button::new("root-sheet-close")
                        .label("Close")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.close_sheet(window, cx);
                        })),
                ),
            );

        Some(
            deferred(
                div()
                    .absolute()
                    .inset_0()
                    .child(
                        div()
                            .id("root-sheet-overlay")
                            .absolute()
                            .inset_0()
                            .bg(rgba(0x00000066))
                            .when(dismissable, |this| {
                                this.on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, window, cx| {
                                        this.close_sheet(window, cx);
                                    }),
                                )
                            }),
                    )
                    .child(surface_element),
            )
            .with_priority(SHEET_PRIORITY)
            .into_any_element(),
        )
    }

    /// The dialog stack, oldest first. Only the topmost is interactive and only
    /// it draws a backdrop.
    fn dialog_layer(&self, _window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let topmost_index = self.dialogs.len().saturating_sub(1);
        self.dialogs
            .iter()
            .enumerate()
            .map(|(index, dialog)| {
                let topmost = index == topmost_index;
                let backdrop_dismissable = dialog.options.is_backdrop_dismissable();
                let (surface, foreground, border, radius) = {
                    let theme = cx.theme();
                    (
                        theme.popover,
                        theme.popover_foreground,
                        theme.border,
                        theme.radius_lg,
                    )
                };

                let backdrop = div()
                    .id(format!("root-dialog-backdrop-{index}"))
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x00000066))
                    .when(topmost && backdrop_dismissable, |this| {
                        this.on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, window, cx| {
                                this.close_dialog(window, cx);
                            }),
                        )
                    });

                let card = div()
                    .id(format!("root-dialog-{index}"))
                    .occlude()
                    .track_focus(&dialog.focus_handle)
                    .bg(surface)
                    .text_color(foreground)
                    .border_1()
                    .border_color(border)
                    .rounded(radius)
                    .p(px(20.0))
                    .min_w(px(320.0))
                    .flex_col()
                    .gap(px(12.0))
                    .child(dialog.content.clone())
                    .child(
                        div().flex().justify_end().child(
                            Button::new(format!("root-dialog-close-{index}"))
                                .label("Close")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.close_dialog(window, cx);
                                })),
                        ),
                    );

                deferred(
                    div()
                        .absolute()
                        .inset_0()
                        .when(topmost, |this| this.child(backdrop))
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(topmost, |this| this.child(card)),
                        ),
                )
                .with_priority(DIALOG_PRIORITY + index)
                .into_any_element()
            })
            .collect()
    }

    /// The notification stack, anchored to the top-right corner.
    fn notification_layer(&self, cx: &mut Context<Self>) -> AnyElement {
        let (foreground, radius) = {
            let theme = cx.theme();
            (theme.background, theme.radius_lg)
        };

        deferred(
            div()
                .id("root-notifications")
                .absolute()
                .top(px(16.0))
                .right(px(16.0))
                .w(px(320.0))
                .flex_col()
                .gap(px(8.0))
                .children(self.notifications.iter().map(|notification| {
                    let color = match notification.level {
                        NotificationLevel::Info => cx.theme().primary,
                        NotificationLevel::Success => cx.theme().success,
                        NotificationLevel::Warning => cx.theme().warning,
                        NotificationLevel::Error => cx.theme().danger,
                    };
                    div()
                        .occlude()
                        .bg(color)
                        .text_color(foreground)
                        .rounded(radius)
                        .p(px(12.0))
                        .child(notification.message.clone())
                })),
        )
        .with_priority(NOTIFICATION_PRIORITY)
        .into_any_element()
    }
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (background, foreground) = {
            let theme = cx.theme();
            (theme.background, theme.foreground)
        };

        // Painted back to front; the deferred priorities make the order
        // independent of construction order.
        let sheet = self.sheet_layer(window, cx);
        let dialogs = self.dialog_layer(window, cx);
        let notifications = self.notification_layer(cx);

        div()
            .id("gpui-net-shell-root")
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::on_escape))
            .relative()
            .size_full()
            .bg(background)
            .text_color(foreground)
            .child(self.view.clone())
            .children(sheet)
            .children(dialogs)
            .child(notifications)
    }
}

fn install_key_bindings(cx: &mut App) {
    if cx.has_global::<KeyBindingsInstalled>() {
        return;
    }
    cx.bind_keys([KeyBinding::new("escape", Escape, Some(CONTEXT))]);
    cx.set_global(KeyBindingsInstalled);
}

fn restore_focus(handle: Option<WeakFocusHandle>, window: &mut Window, cx: &mut App) {
    if let Some(handle) = handle.and_then(|handle| handle.upgrade()) {
        window.focus(&handle, cx);
    }
}

/// Maps a wire value onto a sheet edge, defaulting to `Left`.
pub fn placement_from_wire(value: u32) -> Placement {
    match value {
        1 => Placement::Right,
        2 => Placement::Top,
        3 => Placement::Bottom,
        _ => Placement::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_options_default_to_dismissable() {
        let options = DialogOptions::default();
        assert!(options.is_escape_dismissable());
        assert!(options.is_backdrop_dismissable());

        let options = options
            .escape_dismissable(false)
            .backdrop_dismissable(false);
        assert!(!options.is_escape_dismissable());
        assert!(!options.is_backdrop_dismissable());
    }

    #[test]
    fn notification_levels_round_trip_through_the_wire() {
        assert_eq!(NotificationLevel::from_wire(0), NotificationLevel::Info);
        assert_eq!(NotificationLevel::from_wire(1), NotificationLevel::Success);
        assert_eq!(NotificationLevel::from_wire(2), NotificationLevel::Warning);
        assert_eq!(NotificationLevel::from_wire(3), NotificationLevel::Error);
        assert_eq!(NotificationLevel::from_wire(9), NotificationLevel::Info);
    }
}
