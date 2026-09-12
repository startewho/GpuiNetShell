//! Event-triggered native window effects, ported from `component-shell`'s
//! `window_effects/mod.rs`.
//!
//! Each registration renders a real button. `Dialog`, `AlertDialog`, `Sheet`,
//! and `Notification` are opened only from the button's native click event, so
//! no effect is created while the managed tree is materialized. The shell's
//! `EffectManager` is not needed: the window already sits inside a
//! `gpui_component::Root`, so the native `WindowExt` methods open and paint the
//! effects directly.
//!
//! Adaptation: `component-shell` takes the effect-error reporter as a
//! constructor argument; this runtime records it as an `on_effect_error`
//! method, matching how other components receive callbacks.

use std::sync::Arc;

use gpui::{
    div, AnyElement, App, IntoElement as _, ParentElement as _, Refineable as _, Styled as _,
    Window,
};
use gpui_component::{
    dialog::DialogButtonProps,
    notification::{Notification, NotificationType},
    Placement, WindowExt as _,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, MaterializeRequest, MethodDescriptor, SlotFactory,
};

#[derive(Clone)]
struct Trigger {
    id: String,
    label: String,
}

#[derive(Clone)]
enum Op {
    Title(String),
    Description(String),
    Placement(Placement),
    Kind(NotificationType),
    Autohide(bool),
    ShowCancel(bool),
    OnEffectError(ComponentArgument),
    OnOk(ComponentArgument),
    OnCancel(ComponentArgument),
    OnClose(ComponentArgument),
    OnClick(ComponentArgument),
}

#[derive(Clone, Copy)]
enum Effect {
    Dialog,
    AlertDialog,
    Sheet,
    Notification,
}

struct Materializer(Effect);

fn invoke(
    callback: &Option<ComponentCallback>,
    label: &'static str,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(callback) = callback {
        callback.invoke_with(label, &[], window, cx);
    }
}

fn report_factory_error(
    reporter: &Option<ComponentCallback>,
    surface: &str,
    error: &str,
    window: &mut Window,
    cx: &mut App,
) -> String {
    let message = format!("Failed to render {surface} content: {error}");
    if let Some(reporter) = reporter {
        reporter.invoke_with(
            "window effect error reporter failed",
            &[ComponentCallbackArgument::String(message.clone())],
            window,
            cx,
        );
    }
    message
}

fn last_callback(
    operations: &[Op],
    pick: fn(&Op) -> Option<&ComponentArgument>,
    request: &MaterializeRequest<'_>,
) -> Result<Option<ComponentCallback>, String> {
    operations
        .iter()
        .filter_map(pick)
        .next_back()
        .map(|argument| request.resolve_callback(argument))
        .transpose()
}

fn last<T: Copy>(operations: &[Op], pick: fn(&Op) -> Option<T>, fallback: T) -> T {
    operations
        .iter()
        .filter_map(pick)
        .next_back()
        .unwrap_or(fallback)
}

fn last_string(operations: &[Op], pick: fn(&Op) -> Option<&String>) -> Option<String> {
    operations.iter().filter_map(pick).next_back().cloned()
}

impl ComponentMaterializer for Materializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let trigger = request
            .payload()
            .downcast_ref::<Trigger>()
            .ok_or_else(|| "window effect received an incompatible payload".to_string())?
            .clone();
        if request.children_len() != 0 {
            return Err("window effect triggers do not accept children".to_string());
        }
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<Op>().cloned())
            .collect::<Vec<_>>();

        let reporter = last_callback(
            &operations,
            |op| match op {
                Op::OnEffectError(value) => Some(value),
                _ => None,
            },
            &request,
        )?;
        let on_ok = last_callback(
            &operations,
            |op| match op {
                Op::OnOk(value) => Some(value),
                _ => None,
            },
            &request,
        )?;
        let on_cancel = last_callback(
            &operations,
            |op| match op {
                Op::OnCancel(value) => Some(value),
                _ => None,
            },
            &request,
        )?;
        let on_close = last_callback(
            &operations,
            |op| match op {
                Op::OnClose(value) => Some(value),
                _ => None,
            },
            &request,
        )?;
        let on_click = last_callback(
            &operations,
            |op| match op {
                Op::OnClick(value) => Some(value),
                _ => None,
            },
            &request,
        )?;

        let content: Option<SlotFactory> = match self.0 {
            Effect::Dialog | Effect::Sheet => {
                reject_slots(&mut request, &["trigger", "header", "footer"])?;
                Some(request.take_slot_factory("content").ok_or_else(|| {
                    "Dialog and Sheet require exactly one content(element) named slot".to_string()
                })?)
            }
            Effect::AlertDialog | Effect::Notification => {
                reject_slots(&mut request, &["content", "trigger", "header", "footer"])?;
                None
            }
        };

        let title = last_string(&operations, |op| match op {
            Op::Title(value) => Some(value),
            _ => None,
        });
        let description = last_string(&operations, |op| match op {
            Op::Description(value) => Some(value),
            _ => None,
        });
        let placement = last(
            &operations,
            |op| match op {
                Op::Placement(value) => Some(*value),
                _ => None,
            },
            Placement::Right,
        );
        let note_kind = last(
            &operations,
            |op| match op {
                Op::Kind(value) => Some(*value),
                _ => None,
            },
            NotificationType::Info,
        );
        let autohide = last(
            &operations,
            |op| match op {
                Op::Autohide(value) => Some(*value),
                _ => None,
            },
            true,
        );
        let show_cancel = last(
            &operations,
            |op| match op {
                Op::ShowCancel(value) => Some(*value),
                _ => None,
            },
            false,
        );

        let style = request.take_style();
        let effect = self.0;
        let id = trigger.id;
        let label = trigger.label;
        let mut button = gpui_component::button::Button::new(id.clone())
            .label(label)
            .on_click(move |_, window, cx| {
                let content = content.clone();
                let reporter = reporter.clone();
                let on_ok = on_ok.clone();
                let on_cancel = on_cancel.clone();
                let on_close = on_close.clone();
                let on_click = on_click.clone();
                let title = title.clone();
                let description = description.clone();
                let id = id.clone();
                match effect {
                    Effect::Dialog => open_dialog(
                        &id, title, content, reporter, on_ok, on_cancel, on_close, window, cx,
                    ),
                    Effect::AlertDialog => open_alert(
                        title,
                        description,
                        show_cancel,
                        reporter,
                        on_ok,
                        on_cancel,
                        on_close,
                        window,
                        cx,
                    ),
                    Effect::Sheet => open_sheet(
                        &id, placement, title, content, reporter, on_close, window, cx,
                    ),
                    Effect::Notification => open_notification(
                        &id,
                        title,
                        description,
                        note_kind,
                        autohide,
                        on_click,
                        on_close,
                        window,
                        cx,
                    ),
                }
            });
        button.style().refine(&style);
        Ok(button.into_any_element())
    }
}

fn reject_slots(request: &mut MaterializeRequest<'_>, names: &[&str]) -> Result<(), String> {
    for name in names {
        if request.take_slot_factory(name).is_some() {
            return Err(format!(
                "window effect triggers accept only the content named slot; found `{name}`"
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn open_dialog(
    id: &str,
    title: Option<String>,
    content: Option<SlotFactory>,
    reporter: Option<ComponentCallback>,
    on_ok: Option<ComponentCallback>,
    on_cancel: Option<ComponentCallback>,
    on_close: Option<ComponentCallback>,
    window: &mut Window,
    cx: &mut App,
) {
    let factory = content.expect("validated dialog content");
    let _ = id;
    window.open_dialog(cx, move |mut dialog, _window, _cx| {
        if let Some(title) = title.clone() {
            dialog = dialog.title(title);
        }
        let factory = factory.clone();
        let reporter = reporter.clone();
        dialog = dialog.content(move |card, window, cx| match factory.build(window, cx) {
            Ok(element) => card.child(element),
            Err(error) => {
                let message = report_factory_error(&reporter, "Dialog", &error, window, cx);
                card.child(div().child(message))
            }
        });
        let ok = on_ok.clone();
        let cancel = on_cancel.clone();
        let close = on_close.clone();
        dialog
            .on_ok(move |_, window, cx| {
                invoke(&ok, "Dialog.on_ok callback failed", window, cx);
                true
            })
            .on_cancel(move |_, window, cx| {
                invoke(&cancel, "Dialog.on_cancel callback failed", window, cx);
                true
            })
            .on_close(move |_, window, cx| {
                invoke(&close, "Dialog.on_close callback failed", window, cx);
            })
    });
}

#[allow(clippy::too_many_arguments)]
fn open_alert(
    title: Option<String>,
    description: Option<String>,
    show_cancel: bool,
    _reporter: Option<ComponentCallback>,
    on_ok: Option<ComponentCallback>,
    on_cancel: Option<ComponentCallback>,
    on_close: Option<ComponentCallback>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |mut alert, _, _| {
        if let Some(title) = title.clone() {
            alert = alert.title(title);
        }
        if let Some(description) = description.clone() {
            alert = alert.description(description);
        }
        let ok = on_ok.clone();
        let cancel = on_cancel.clone();
        let close = on_close.clone();
        alert
            .button_props(
                DialogButtonProps::default()
                    .show_cancel(show_cancel)
                    .on_ok(move |_, window, cx| {
                        invoke(&ok, "AlertDialog.on_ok callback failed", window, cx);
                        true
                    })
                    .on_cancel(move |_, window, cx| {
                        invoke(&cancel, "AlertDialog.on_cancel callback failed", window, cx);
                        true
                    }),
            )
            .on_close(move |_, window, cx| {
                invoke(&close, "AlertDialog.on_close callback failed", window, cx);
            })
    });
}

#[allow(clippy::too_many_arguments)]
fn open_sheet(
    id: &str,
    placement: Placement,
    title: Option<String>,
    content: Option<SlotFactory>,
    reporter: Option<ComponentCallback>,
    on_close: Option<ComponentCallback>,
    window: &mut Window,
    cx: &mut App,
) {
    let factory = content.expect("validated sheet content");
    let _ = id;
    window.open_sheet_at(placement, cx, move |mut sheet, window, cx| {
        if let Some(title) = title.clone() {
            sheet = sheet.title(title);
        }
        match factory.build(window, cx) {
            Ok(element) => sheet = sheet.child(element),
            Err(error) => {
                let message = report_factory_error(&reporter, "Sheet", &error, window, cx);
                sheet = sheet.child(div().child(message));
            }
        }
        let close = on_close.clone();
        sheet.on_close(move |_, window, cx| {
            invoke(&close, "Sheet.on_close callback failed", window, cx);
        })
    });
}

struct NotificationId;

#[allow(clippy::too_many_arguments)]
fn open_notification(
    id: &str,
    title: Option<String>,
    message: Option<String>,
    kind: NotificationType,
    autohide: bool,
    on_click: Option<ComponentCallback>,
    on_close: Option<ComponentCallback>,
    window: &mut Window,
    cx: &mut App,
) {
    let mut notification = Notification::new()
        .id1::<NotificationId>(id.to_owned())
        .with_type(kind)
        .autohide(autohide);
    if let Some(title) = title {
        notification = notification.title(title);
    }
    if let Some(message) = message {
        notification = notification.message(message);
    }
    notification = notification
        .on_click(move |_, window, cx| {
            invoke(
                &on_click,
                "Notification.on_click callback failed",
                window,
                cx,
            );
        })
        .on_close(move |window, cx| {
            invoke(
                &on_close,
                "Notification.on_close callback failed",
                window,
                cx,
            );
        });
    window.push_notification(notification, cx);
}

fn callback_method(name: &'static str, op: fn(ComponentArgument) -> Op) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(
            "callback",
            ArgumentSchema::Callback,
        )],
        move |arguments| match arguments {
            [value @ ComponentArgument::Callback(_)] => {
                Ok(ComponentPayload::new(op(value.clone())))
            }
            _ => Err(format!("{name}(callback) expects a callback")),
        },
    )
    .with_documentation("Configures this native window effect.")
}

fn bool_method(name: &'static str, op: fn(bool) -> Op) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(op(*value))),
            _ => Err(format!("{name}(value) expects a boolean")),
        },
    )
    .with_documentation("Configures this native window effect.")
}

fn text(name: &'static str, op: fn(String) -> Op) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new("text", ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                Ok(ComponentPayload::new(op(value.clone())))
            }
            _ => Err(format!("{name}(text) expects non-empty text")),
        },
    )
    .with_documentation("Configures this native window effect.")
}

fn descriptor(
    name: &'static str,
    effect: Effect,
    methods: Vec<MethodDescriptor>,
) -> ComponentDescriptor {
    ComponentDescriptor::new(name, Arc::new(Materializer(effect)))
        .with_constructors(vec![ConstructorDescriptor::new(
            name,
            vec![
                ArgumentDescriptor::new("id", ArgumentSchema::String),
                ArgumentDescriptor::new("label", ArgumentSchema::String),
            ],
            move |arguments| match arguments {
                [ComponentArgument::String(id), ComponentArgument::String(label)]
                    if !id.trim().is_empty() && !label.trim().is_empty() =>
                {
                    Ok(ComponentPayload::new(Trigger {
                        id: id.clone(),
                        label: label.clone(),
                    }))
                }
                _ => Err(format!("{name}(id, label) expects two non-empty strings")),
            },
        )])
        .with_methods(methods)
        .with_documentation(
            "A real button-triggered native window effect; `on_effect_error` receives \
             asynchronous content failures.",
        )
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(descriptor(
            "Dialog",
            Effect::Dialog,
            vec![
                text("title", Op::Title),
                callback_method("on_ok", Op::OnOk),
                callback_method("on_cancel", Op::OnCancel),
                callback_method("on_close", Op::OnClose),
                callback_method("on_effect_error", Op::OnEffectError),
            ],
        ))
        .expect("the built-in Dialog descriptor is valid");
    registry
        .register(descriptor(
            "AlertDialog",
            Effect::AlertDialog,
            vec![
                text("title", Op::Title),
                text("description", Op::Description),
                bool_method("show_cancel", Op::ShowCancel),
                callback_method("on_ok", Op::OnOk),
                callback_method("on_cancel", Op::OnCancel),
                callback_method("on_close", Op::OnClose),
                callback_method("on_effect_error", Op::OnEffectError),
            ],
        ))
        .expect("the built-in AlertDialog descriptor is valid");
    registry
        .register(descriptor(
            "Sheet",
            Effect::Sheet,
            vec![
                text("title", Op::Title),
                MethodDescriptor::new(
                    "placement",
                    vec![ArgumentDescriptor::new(
                        "placement",
                        ArgumentSchema::Enum(&["top", "right", "bottom", "left"]),
                    )],
                    |arguments| match arguments {
                        [ComponentArgument::Enum(value)] => match value.as_str() {
                            "top" => Ok(Op::Placement(Placement::Top)),
                            "right" => Ok(Op::Placement(Placement::Right)),
                            "bottom" => Ok(Op::Placement(Placement::Bottom)),
                            "left" => Ok(Op::Placement(Placement::Left)),
                            _ => Err(format!("unsupported Sheet placement `{value}`")),
                        }
                        .map(ComponentPayload::new),
                        _ => Err("placement expects top, right, bottom, or left".into()),
                    },
                )
                .with_documentation("Sets which edge the sheet opens from."),
                callback_method("on_close", Op::OnClose),
                callback_method("on_effect_error", Op::OnEffectError),
            ],
        ))
        .expect("the built-in Sheet descriptor is valid");
    registry
        .register(descriptor(
            "Notification",
            Effect::Notification,
            vec![
                text("title", Op::Title),
                text("message", Op::Description),
                MethodDescriptor::new(
                    "type",
                    vec![ArgumentDescriptor::new(
                        "type",
                        ArgumentSchema::Enum(&["info", "success", "warning", "error"]),
                    )],
                    |arguments| match arguments {
                        [ComponentArgument::Enum(value)] => match value.as_str() {
                            "info" => Ok(Op::Kind(NotificationType::Info)),
                            "success" => Ok(Op::Kind(NotificationType::Success)),
                            "warning" => Ok(Op::Kind(NotificationType::Warning)),
                            "error" => Ok(Op::Kind(NotificationType::Error)),
                            _ => Err(format!("unsupported notification type `{value}`")),
                        }
                        .map(ComponentPayload::new),
                        _ => Err("type expects info, success, warning, or error".into()),
                    },
                )
                .with_documentation("Sets the notification severity."),
                bool_method("autohide", Op::Autohide),
                callback_method("on_click", Op::OnClick),
                callback_method("on_close", Op::OnClose),
                callback_method("on_effect_error", Op::OnEffectError),
            ],
        ))
        .expect("the built-in Notification descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_effects_register_in_id_order() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        let names = frozen
            .descriptors()
            .map(|descriptor| descriptor.name())
            .collect::<Vec<_>>();
        assert_eq!(names, ["Dialog", "AlertDialog", "Sheet", "Notification"]);
    }
}
