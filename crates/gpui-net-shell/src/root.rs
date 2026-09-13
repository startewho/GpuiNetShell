//! The window-level overlay host.
//!
//! [`Root`] owns the managed content view and the one overlay it still needs:
//! the combobox popup, which is a `Bottom` sheet. Dialogs, alert dialogs,
//! sheets, and notifications are opened by the managed window-effect controls
//! through `gpui_component`'s `WindowExt`; their layers are rendered here too,
//! because `gpui_component::Root` draws only its child and expects the
//! application root to place them.
//!
//! # Stacking order
//!
//! Painted back to front:
//!
//! 1. **Content** — the managed `ShellView`.
//! 2. **Sheet** (`8`) — at most one, anchored to a viewport edge. The combobox
//!    popup is a `Bottom` sheet whose content is the option menu.
//! 3. **`gpui_component` effects** — sheet, dialog, and notification layers for
//!    the `Dialog`/`AlertDialog`/`Sheet`/`Notification` window-effect controls.
//!
//! # Focus
//!
//! Opening the popup sheet records what was focused and focuses the sheet;
//! closing it restores that handle.

use gpui::prelude::*;
use gpui::{
    actions, deferred, div, px, rgba, AnyElement, AnyView, App, Context, Entity, FocusHandle,
    Global, IntoElement, KeyBinding, MouseButton, Render, WeakEntity, WeakFocusHandle, Window,
};
use gpui_base::Placement;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::{ActiveTheme as _, Sizable as _};

use crate::abi::GpuiNetCallbacks;
use crate::view::ShellView;

actions!(gpui_net_shell_root, [Escape]);

/// The key context the root installs.
const CONTEXT: &str = "GpuiNetShellRoot";

/// Paint priority of the sheet layer: above content, below the effects.
const SHEET_PRIORITY: usize = 8;

/// One selectable entry of a popup. `token` is the managed callback to run;
/// `token == 0` means the entry is inert.
#[derive(Clone)]
pub struct PopupItem {
    pub label: String,
    pub token: u64,
}

/// The open popup sheet. A combobox popup is the only user.
struct ActiveSheet {
    content: AnyView,
    placement: Placement,
    focus_handle: FocusHandle,
    restore_focus: Option<WeakFocusHandle>,
}

/// Marks that this `App` already has the root's key bindings.
struct KeyBindingsInstalled;

impl Global for KeyBindingsInstalled {}

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

/// The window-level overlay host: managed content and the popup sheet.
pub struct Root {
    view: Entity<ShellView>,
    session_id: u64,
    callbacks: GpuiNetCallbacks,
    sheet: Option<ActiveSheet>,
    /// Draw a custom title bar above the content (the native one is hidden).
    custom_titlebar: bool,
}

#[allow(dead_code)] // accessors used by tests and future integrations
impl Root {
    pub fn new(
        view: Entity<ShellView>,
        session_id: u64,
        callbacks: GpuiNetCallbacks,
        custom_titlebar: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        install_key_bindings(cx);
        Self {
            view,
            session_id,
            callbacks,
            sheet: None,
            custom_titlebar,
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

    /// Repaints one retained entity subtree inside the managed content view.
    pub fn notify_entity(&mut self, entity_id: u64, cx: &mut Context<Self>) {
        self.view
            .update(cx, |view, cx| view.notify_entity(entity_id, cx));
    }

    // -- Popup (a sheet) -----------------------------------------------------

    /// Opens the option menu as a `Bottom` sheet, replacing any open sheet.
    ///
    /// Selecting an item runs its managed token and closes the sheet.
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

    // -- Sheet ---------------------------------------------------------------

    /// Opens a sheet on the given edge, replacing any sheet already open.
    fn open_sheet(
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

    /// Closes the sheet, returning whether one was open.
    pub fn close_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(sheet) = self.sheet.take() else {
            return false;
        };
        restore_focus(sheet.restore_focus, window, cx);
        cx.notify();
        true
    }

    /// Escape closes the sheet (including a combobox popup).
    fn on_escape(&mut self, _: &Escape, window: &mut Window, cx: &mut Context<Self>) {
        if self.sheet.is_some() {
            self.close_sheet(window, cx);
        }
    }

    /// The title bar's theme toggle: flips between light and dark.
    fn on_titlebar_theme(
        &mut self,
        _: &gpui::ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dark = !cx.theme().is_dark();
        gpui_component::Theme::change(
            if dark {
                gpui_component::ThemeMode::Dark
            } else {
                gpui_component::ThemeMode::Light
            },
            Some(window),
            cx,
        );
    }

    // -- Layers --------------------------------------------------------------

    /// The sheet, anchored to a viewport edge. A combobox popup is this layer.
    fn sheet_layer(&self, _window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let sheet = self.sheet.as_ref()?;
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, window, cx| {
                                    this.close_sheet(window, cx);
                                }),
                            ),
                    )
                    .child(surface_element),
            )
            .with_priority(SHEET_PRIORITY)
            .into_any_element(),
        )
    }
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (background, foreground) = {
            let theme = cx.theme();
            (theme.background, theme.foreground)
        };

        let sheet = self.sheet_layer(window, cx);

        // The `gpui_component` root draws only its child, its tooltip overlay,
        // and its native menu; its sheet/dialog/notification layers are the
        // application root's to render. The window-effect controls open through
        // `WindowExt`, so without these layers their effects open into a window
        // that never draws them.
        let component_sheets = gpui_component::Root::render_sheet_layer(window, cx);
        let component_dialogs = gpui_component::Root::render_dialog_layer(window, cx);
        let component_notifications = gpui_component::Root::render_notification_layer(window, cx);

        let titlebar_height = px(34.0);
        let content = div()
            .relative()
            .size_full()
            .when(self.custom_titlebar, |this| this.pt(titlebar_height))
            .child(self.view.clone())
            .children(sheet)
            .children(component_sheets)
            .children(component_dialogs)
            .children(component_notifications);

        let theme_icon = if cx.theme().is_dark() {
            gpui_component::IconName::Sun
        } else {
            gpui_component::IconName::Moon
        };
        let titlebar = gpui_component::TitleBar::new()
            .child(div().text_sm().child("GpuiNetShell"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_2()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        Button::new("titlebar-theme-toggle")
                            .icon(theme_icon)
                            .small()
                            .ghost()
                            .tooltip("Toggle light and dark")
                            .on_click(cx.listener(Self::on_titlebar_theme)),
                    ),
            );

        div()
            .id("gpui-net-shell-host")
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::on_escape))
            .relative()
            .size_full()
            .bg(background)
            .text_color(foreground)
            .child(content)
            .when(self.custom_titlebar, |this| {
                this.child(div().absolute().top_0().left_0().right_0().child(titlebar))
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_popup_item_keeps_its_label_and_token() {
        let item = PopupItem {
            label: "Open".into(),
            token: 7,
        };
        assert_eq!(item.label, "Open");
        assert_eq!(item.token, 7);
    }
}
