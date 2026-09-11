//! The window-level overlay host, modeled on `gpui-shell`'s `root.rs`.
//!
//! [`Root`] owns the managed content view and the layers painted over it:
//! dialogs (a stack, topmost interactive) and notifications (a corner stack).
//! Tooltips are handled by the `gpui-component` window root this sits inside.
//!
//! Painted back to front: content, dialog stack, notifications. A dialog draws
//! a single backdrop above the content; the topmost dialog is the only one
//! that responds.
//!
//! Like shell's `ShellRoot`, this type positions and layers. The managed host
//! opens overlays through the command ingress; the content of an overlay is
//! built here from the request so no overlay state crosses the ABI.

use std::time::Duration;

use gpui::prelude::*;
use gpui::{div, px, rgba, AnyElement, Context, Entity, IntoElement, Render, Window};
use gpui_component::button::Button;
use gpui_component::ActiveTheme as _;

use crate::view::ShellView;

/// How long a notification stays before it dismisses itself.
const NOTIFICATION_LIFETIME: Duration = Duration::from_secs(4);

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

struct Dialog {
    title: String,
    body: String,
}

struct Notification {
    id: u64,
    message: String,
    level: NotificationLevel,
}

/// The window-level overlay host: managed content, dialogs, notifications.
pub struct Root {
    view: Entity<ShellView>,
    dialogs: Vec<Dialog>,
    notifications: Vec<Notification>,
    next_notification: u64,
}

#[allow(dead_code)] // accessors used by tests and the host
impl Root {
    pub fn new(view: Entity<ShellView>) -> Self {
        Self {
            view,
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

    /// Opens a dialog on top of the stack.
    pub fn open_dialog(&mut self, title: String, body: String, cx: &mut Context<Self>) {
        self.dialogs.push(Dialog { title, body });
        cx.notify();
    }

    /// Closes the topmost dialog. Returns whether one was open.
    pub fn close_dialog(&mut self, cx: &mut Context<Self>) -> bool {
        let closed = self.dialogs.pop().is_some();
        if closed {
            cx.notify();
        }
        closed
    }

    pub fn dialog_count(&self) -> usize {
        self.dialogs.len()
    }

    /// Posts a notification that dismisses itself after a short lifetime.
    pub fn push_notification(
        &mut self,
        message: String,
        level: NotificationLevel,
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

    /// The topmost dialog as a centered card over a backdrop.
    fn dialog_layer(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let dialog = self.dialogs.last()?;
        let (surface, foreground, border, muted, radius) = {
            let theme = cx.theme();
            (
                theme.popover,
                theme.popover_foreground,
                theme.border,
                theme.muted_foreground,
                theme.radius_lg,
            )
        };

        Some(
            div()
                .absolute()
                .inset_0()
                .child(div().absolute().inset_0().bg(rgba(0x00000066)))
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .occlude()
                                .bg(surface)
                                .text_color(foreground)
                                .border_1()
                                .border_color(border)
                                .rounded(radius)
                                .p(px(20.0))
                                .min_w(px(320.0))
                                .flex_col()
                                .gap(px(12.0))
                                .child(div().text_size(px(16.0)).child(dialog.title.clone()))
                                .child(div().text_color(muted).child(dialog.body.clone()))
                                .child(div().flex().justify_end().child(
                                    Button::new("root-dialog-close").label("Close").on_click(
                                        cx.listener(|this, _, _, cx| {
                                            this.close_dialog(cx);
                                        }),
                                    ),
                                )),
                        ),
                )
                .into_any_element(),
        )
    }

    /// The notification stack, anchored to the top-right corner.
    fn notification_layer(&self, cx: &mut Context<Self>) -> AnyElement {
        let (foreground, radius) = {
            let theme = cx.theme();
            (theme.background, theme.radius_lg)
        };

        div()
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
            }))
            .into_any_element()
    }
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (background, foreground) = {
            let theme = cx.theme();
            (theme.background, theme.foreground)
        };

        let dialog = self.dialog_layer(cx);
        let notifications = self.notification_layer(cx);

        div()
            .relative()
            .size_full()
            .bg(background)
            .text_color(foreground)
            .child(self.view.clone())
            .children(dialog)
            .child(notifications)
            .into_any_element()
    }
}
