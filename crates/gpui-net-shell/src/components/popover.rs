//! `Popover`, ported from `component-shell`'s `overlays/popover.rs`.
//!
//! A button-triggered popover. The trigger is built from the constructor's
//! `(id, label)`; the surface content arrives as the named `content` slot.
//!
//! The materializer uses `gpui-base`'s `Popover`, whose content builder is
//! `FnOnce`: this runtime materializes children eagerly, so it has no lazy slot
//! factory to hand a re-runnable `Fn`. The method vocabulary still matches
//! `component-shell` (`card_anchor`, `default_open`, `open`, `overlay_closable`,
//! `on_open_change`); `appearance` is omitted because the base surface has no
//! unstyled mode.

use std::sync::Arc;

use gpui::{div, Anchor, AnyElement, ParentElement as _};
use gpui_base::Popover;
use gpui_component::button::{Button, ButtonVariants as _};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallbackArgument,
    ComponentDescriptor, ComponentMaterializer, ComponentPayload, ComponentRegistry,
    ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct PopoverPayload {
    id: String,
    label: String,
}

#[derive(Clone)]
enum PopoverOp {
    Anchor(Anchor),
    DefaultOpen(bool),
    Open(bool),
    OverlayClosable(bool),
    OnOpenChange(ComponentArgument),
}

struct PopoverMaterializer;

impl ComponentMaterializer for PopoverMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<PopoverPayload>()
            .ok_or_else(|| "Popover received an incompatible payload".to_string())?
            .clone();
        let content = request
            .take_slot("content")
            .ok_or_else(|| "Popover requires content(element)".to_string())?;

        let mut popover = Popover::new(payload.id.clone()).trigger(
            Button::new(format!("popover-trigger:{}", payload.id))
                .ghost()
                .label(payload.label),
        );
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<PopoverOp>().cloned())
        {
            popover = match operation {
                PopoverOp::Anchor(anchor) => popover.anchor(anchor),
                PopoverOp::DefaultOpen(open) => popover.default_open(open),
                PopoverOp::Open(open) => popover.open(open),
                PopoverOp::OverlayClosable(closable) => popover.overlay_closable(closable),
                PopoverOp::OnOpenChange(argument) => {
                    let callback = request.resolve_callback(&argument)?;
                    popover.on_open_change(move |open, window, cx| {
                        callback.invoke_with(
                            "Popover.on_open_change callback failed",
                            &[ComponentCallbackArgument::Boolean(*open)],
                            window,
                            cx,
                        );
                    })
                }
            };
        }
        popover = popover.content(move |_, _window, _cx| content);
        request.finish(div().child(popover))
    }
}

fn boolean_op(
    component_method: &'static str,
    arguments: &[ComponentArgument],
    wrap: impl FnOnce(bool) -> PopoverOp,
) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(wrap(*value))),
        _ => Err(format!(
            "Popover.{component_method}(value) expects a boolean"
        )),
    }
}

fn anchor_op(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    let [ComponentArgument::Enum(anchor)] = arguments else {
        return Err("Popover.card_anchor(anchor) expects an anchor literal".into());
    };
    let anchor = match anchor.as_str() {
        "top_left" => Anchor::TopLeft,
        "top_center" => Anchor::TopCenter,
        "top_right" => Anchor::TopRight,
        "bottom_left" => Anchor::BottomLeft,
        "bottom_center" => Anchor::BottomCenter,
        "bottom_right" => Anchor::BottomRight,
        "left_center" => Anchor::LeftCenter,
        "right_center" => Anchor::RightCenter,
        _ => return Err(format!("unsupported Popover anchor `{anchor}`")),
    };
    Ok(ComponentPayload::new(PopoverOp::Anchor(anchor)))
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    let boolean = |name, wrap: fn(bool) -> PopoverOp| {
        MethodDescriptor::new(
            name,
            vec![ArgumentDescriptor::new("value", ArgumentSchema::Boolean)],
            move |arguments| boolean_op(name, arguments, wrap),
        )
    };
    registry
        .register(
            ComponentDescriptor::new("Popover", Arc::new(PopoverMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Popover",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("label", ArgumentSchema::String),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(label)]
                            if !id.trim().is_empty() && !label.trim().is_empty() =>
                        {
                            Ok(ComponentPayload::new(PopoverPayload {
                                id: id.clone(),
                                label: label.clone(),
                            }))
                        }
                        [ComponentArgument::String(_), ComponentArgument::String(_)] => {
                            Err("Popover id and label must not be empty".into())
                        }
                        _ => Err("Popover(id, label) expects two strings".into()),
                    },
                )])
                .with_methods(vec![
                    // Not `anchor`: `component-shell` uses `card_anchor` because
                    // its element prototype reserves the bare name.
                    MethodDescriptor::new(
                        "card_anchor",
                        vec![ArgumentDescriptor::new(
                            "anchor",
                            ArgumentSchema::Enum(&[
                                "top_left",
                                "top_center",
                                "top_right",
                                "bottom_left",
                                "bottom_center",
                                "bottom_right",
                                "left_center",
                                "right_center",
                            ]),
                        )],
                        anchor_op,
                    )
                    .with_documentation("Positions the popover relative to its trigger."),
                    boolean("default_open", PopoverOp::DefaultOpen)
                        .with_documentation("Sets the initial uncontrolled open state."),
                    boolean("open", PopoverOp::Open)
                        .with_documentation("Controls whether the popover is open."),
                    boolean("overlay_closable", PopoverOp::OverlayClosable).with_documentation(
                        "Controls whether pressing outside dismisses the popover.",
                    ),
                    MethodDescriptor::new(
                        "on_open_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                PopoverOp::OnOpenChange(ComponentArgument::Callback(*token)),
                            )),
                            [ComponentArgument::Number(token)] => Ok(ComponentPayload::new(
                                PopoverOp::OnOpenChange(ComponentArgument::Callback(*token as u64)),
                            )),
                            _ => Err("Popover.on_open_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Runs when pointer interaction changes the open state."),
                ])
                .with_documentation("A button-triggered popover with content(element)."),
        )
        .expect("the built-in Popover descriptor is valid");
}
