//! The window-level overlay host, modeled on `gpui-shell`'s `root.rs`.
//!
//! [`Root`] owns the managed content view and the layers painted over it:
//! popups (an anchored menu), dialogs (a stack, topmost interactive), and
//! notifications (a corner stack). Tooltips are handled by the
//! `gpui-component` window root this sits inside.
//!
//! Painted back to front: content, popup, dialog stack, notifications. A dialog
//! draws a single backdrop above the content; the topmost dialog is the only one
//! that responds.
//!
//! # Render triggering
//!
//! Every overlay mutation ends in [`Context::notify`], so the GPUI repaint is
//! scheduled by the same mechanism as any other view change. Re-rendering the
//! content itself is a separate, explicit step — `invalidate_view` calls the
//! managed `ShellView::refresh`, which also notifies — because a clean content
//! snapshot must not be rebuilt just because an overlay opened.

use std::time::Duration;

use gpui::prelude::*;
use gpui::{div, px, rgba, AnyElement, Context, Entity, IntoElement, Render, Window};
use gpui_component::button::Button;
use gpui_component::ActiveTheme as _;

use crate::abi::GpuiNetCallbacks;
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

/// One selectable entry of a popup. `token` is the managed callback to run;
/// `token == 0` means the entry is inert.
#[derive(Clone)]
pub struct PopupItem {
    pub label: String,
    pub token: u64,
}

struct Dialog {
    title: String,
    body: String,
}

struct Popup {
    items: Vec<PopupItem>,
}

struct Notification {
    id: u64,
    message: String,
    level: NotificationLevel,
}

/// The window-level overlay host: managed content, popups, dialogs,
/// notifications.
pub struct Root {
    view: Entity<ShellView>,
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    popup: Option<Popup>,
    dialogs: Vec<Dialog>,
    notifications: Vec<Notification>,
    next_notification: u64,
}

#[allow(dead_code)] // accessors used by tests and future integrations
impl Root {
    pub fn new(view: Entity<ShellView>, session_id: u64, callbacks: GpuiNetCallbacks) -> Self {
        Self {
            view,
            session_id,
            callbacks,
            popup: None,
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
    ///
    /// This is the explicit part of render triggering: the managed host changed
    /// state the content reads, so the next frame rebuilds its snapshot.
    pub fn invalidate_view(&mut self, cx: &mut Context<Self>) {
        self.view.update(cx, |view, cx| view.refresh(cx));
    }

    /// Opens a popup with the given items, replacing any open popup.
    pub fn open_popup(&mut self, items: Vec<PopupItem>, cx: &mut Context<Self>) {
        if items.is_empty() {
            self.popup = None;
        } else {
            self.popup = Some(Popup { items });
        }
        cx.notify();
    }

    /// Closes the popup. Returns whether one was open.
    pub fn close_popup(&mut self, cx: &mut Context<Self>) -> bool {
        let closed = self.popup.take().is_some();
        if closed {
            cx.notify();
        }
        closed
    }

    pub fn popup_open(&self) -> bool {
        self.popup.is_some()
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

    /// The popup: a backdrop that dismisses, and a centered menu of items.
    fn popup_layer(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let popup = self.popup.as_ref()?;
        let (surface, foreground, border, radius) = {
            let theme = cx.theme();
            (
                theme.popover,
                theme.popover_foreground,
                theme.border,
                theme.radius_lg,
            )
        };

        let items: Vec<AnyElement> = popup
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let token = item.token;
                Button::new(format!("root-popup-item-{index}"))
                    .label(item.label.clone())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if token != 0 {
                            if let Some(click) = this.callbacks.click {
                                // SAFETY: the managed callback copies anything it keeps.
                                unsafe {
                                    let _ = click(this.session_id, token);
                                }
                            }
                        }
                        this.close_popup(cx);
                        this.invalidate_view(cx);
                    }))
                    .into_any_element()
            })
            .collect();

        Some(
            div()
                .absolute()
                .inset_0()
                .child(
                    div()
                        .id("root-popup-backdrop")
                        .absolute()
                        .inset_0()
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.close_popup(cx);
                            }),
                        ),
                )
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
                                .p(px(8.0))
                                .min_w(px(200.0))
                                .flex_col()
                                .gap(px(4.0))
                                .children(items),
                        ),
                )
                .into_any_element(),
        )
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

        // Painted back to front.
        let popup = self.popup_layer(cx);
        let dialog = self.dialog_layer(cx);
        let notifications = self.notification_layer(cx);

        div()
            .relative()
            .size_full()
            .bg(background)
            .text_color(foreground)
            .child(self.view.clone())
            .children(popup)
            .children(dialog)
            .child(notifications)
            .into_any_element()
    }
}
