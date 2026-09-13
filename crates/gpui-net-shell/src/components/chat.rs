//! The Chat family, ported from `component-shell`'s `chat.rs`.
//!
//! `Attachment`, `Bubble`, `Marker`, and `Message` compose ordinary children
//! into their content slot. `ShimmerText` carries only text. `MessageScroller`
//! is a virtualized transcript: unlike `component-shell`, which consumes an
//! externally registered `MessageScrollerState`, this runtime creates and
//! retains that state from the component `id` plus an `item_count`, and renders
//! each row through a managed element callback.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    div, AnyElement, Axis, Entity, IntoElement as _, ParentElement as _, Refineable as _,
    SharedString, Styled as _,
};
use gpui_component::{
    attachment::{Attachment, AttachmentContent, AttachmentStatus},
    avatar::Avatar,
    bubble::{Bubble, BubbleVariant},
    marker::{Marker, MarkerLoadingStyle, MarkerVariant},
    message::{Message, MessageAlignment, MessageContent, MessageFooter, MessageHeader},
    message_scroller::{MessageScroller, MessageScrollerState},
    shimmer::ShimmerText,
    text::Text,
    Sizable as _, Size,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    ElementCallback, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
enum Op {
    Status(AttachmentStatus),
    Axis(Axis),
    Size(Size),
    Alignment(MessageAlignment),
    BubbleVariant(BubbleVariant),
    MarkerVariant(MarkerVariant),
    Loading(bool),
    LoadingStyle(MarkerLoadingStyle),
    Id(String),
    Duration(Duration),
    Spread(f32),
    Reverse(bool),
    Once(bool),
    Scrollbar(bool),
    JumpButton(bool),
    JumpButtonLabel(String),
    RenderItem(ComponentArgument),
    Name(String),
    Time(String),
    Avatar(String),
}

#[derive(Clone)]
struct AttachmentPayload(String);

#[derive(Clone, Copy)]
struct BubblePayload;

#[derive(Clone)]
struct MarkerPayload(String);

#[derive(Clone, Copy)]
struct MessagePayload;

#[derive(Clone)]
struct ShimmerPayload(String);

#[derive(Clone)]
struct ScrollerPayload {
    id: String,
    item_count: usize,
}

fn operations(request: &MaterializeRequest<'_>) -> Vec<Op> {
    request
        .methods()
        .filter_map(|method| method.payload().downcast_ref::<Op>().cloned())
        .collect()
}

struct AttachmentMaterializer;

impl ComponentMaterializer for AttachmentMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<AttachmentPayload>()
            .ok_or_else(|| "Attachment received an incompatible payload".to_string())?
            .0
            .clone();
        let mut attachment = Attachment::new().id(id);
        for operation in operations(&request) {
            attachment = match operation {
                Op::Status(value) => attachment.status(value),
                Op::Axis(value) => attachment.axis(value),
                Op::Size(value) => attachment.with_size(value),
                _ => attachment,
            };
        }
        let children = request.take_children();
        if !children.is_empty() {
            attachment = attachment.content(AttachmentContent::new().children(children));
        }
        attachment.style().refine(&request.take_style());
        Ok(attachment.into_any_element())
    }
}

struct BubbleMaterializer;

impl ComponentMaterializer for BubbleMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut bubble = Bubble::new();
        for operation in operations(&request) {
            bubble = match operation {
                Op::Alignment(value) => bubble.alignment(value),
                Op::BubbleVariant(value) => bubble.with_variant(value),
                _ => bubble,
            };
        }
        request.finish(bubble)
    }
}

struct MarkerMaterializer;

impl ComponentMaterializer for MarkerMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<MarkerPayload>()
            .ok_or_else(|| "Marker received an incompatible payload".to_string())?
            .0
            .clone();
        let mut marker = Marker::new().id(id);
        for operation in operations(&request) {
            marker = match operation {
                Op::MarkerVariant(value) => marker.with_variant(value),
                Op::Loading(value) => marker.loading(value),
                Op::LoadingStyle(value) => marker.with_loading_style(value),
                _ => marker,
            };
        }
        request.finish(marker)
    }
}

struct MessageMaterializer;

impl ComponentMaterializer for MessageMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut message = Message::new();
        for operation in operations(&request) {
            message = match operation {
                Op::Alignment(value) => message.alignment(value),
                Op::Name(value) => message.header(MessageHeader::new().child(Text::from(value))),
                Op::Time(value) => message.footer(MessageFooter::new().child(Text::from(value))),
                Op::Avatar(value) => message.avatar(Avatar::new().name(value)),
                _ => message,
            };
        }
        let children = request.take_children();
        if !children.is_empty() {
            message = message.content(MessageContent::new().children(children));
        }
        message.style().refine(&request.take_style());
        Ok(message.into_any_element())
    }
}

struct ShimmerMaterializer;

impl ComponentMaterializer for ShimmerMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("ShimmerText does not accept children".to_string());
        }
        let text = request
            .payload()
            .downcast_ref::<ShimmerPayload>()
            .ok_or_else(|| "ShimmerText received an incompatible payload".to_string())?
            .0
            .clone();
        let mut shimmer = ShimmerText::new(text);
        for operation in operations(&request) {
            shimmer = match operation {
                Op::Id(value) => shimmer.id(value),
                Op::Duration(value) => shimmer.duration(value),
                Op::Spread(value) => shimmer.spread(value),
                Op::Reverse(value) => shimmer.reverse(value),
                Op::Once(value) => shimmer.once(value),
                _ => shimmer,
            };
        }
        shimmer.style().refine(&request.take_style());
        Ok(shimmer.into_any_element())
    }
}

struct ScrollerMaterializer;

impl ComponentMaterializer for ScrollerMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("MessageScroller does not accept children".to_string());
        }
        let payload = request
            .payload()
            .downcast_ref::<ScrollerPayload>()
            .ok_or_else(|| "MessageScroller received an incompatible payload".to_string())?
            .clone();
        let mut renderer: Option<ElementCallback> = None;
        let mut options = Vec::new();
        for operation in operations(&request) {
            match operation {
                Op::RenderItem(argument) => {
                    renderer = Some(request.resolve_element_callback(&argument)?);
                }
                other => options.push(other),
            }
        }
        let key = SharedString::from(format!("shell-message-scroller:{}", payload.id));
        let item_count = payload.item_count;
        let state: Entity<MessageScrollerState> =
            request.use_keyed_state(key, move |_, cx| MessageScrollerState::new(item_count, cx));
        request.update_entity(&state, |state, cx| {
            if state.item_count() != item_count {
                state.reset(item_count, cx);
            }
        });

        let renderer = renderer;
        let mut scroller =
            MessageScroller::new(
                payload.id,
                state,
                move |index, window, cx| match &renderer {
                    Some(callback) => callback
                        .build(&[index.to_string()], window, cx)
                        .unwrap_or_else(|error| {
                            div()
                                .child(format!("Failed to render message row: {error}"))
                                .into_any_element()
                        }),
                    None => div().child(format!("Row {index}")).into_any_element(),
                },
            );
        for operation in options {
            scroller = match operation {
                Op::Scrollbar(value) => scroller.scrollbar(value),
                Op::JumpButton(value) => scroller.jump_button(value),
                Op::JumpButtonLabel(value) => scroller.with_jump_button_label(value),
                _ => scroller,
            };
        }
        scroller.style().refine(&request.take_style());
        Ok(scroller.into_any_element())
    }
}

fn enum_method(
    component: &'static str,
    name: &'static str,
    values: &'static [&'static str],
    parse: fn(&str) -> Option<Op>,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Enum(values))],
        move |arguments| match arguments {
            [ComponentArgument::Enum(value)] => parse(value)
                .map(ComponentPayload::new)
                .ok_or_else(|| format!("unsupported {component}.{name} value `{value}`")),
            _ => Err(format!("{component}.{name} expects one enum literal")),
        },
    )
    .with_documentation("Sets a native Chat family option.")
}

fn bool_method(
    component: &'static str,
    name: &'static str,
    wrap: fn(bool) -> Op,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(wrap(*value))),
            _ => Err(format!("{component}.{name} expects one boolean")),
        },
    )
    .with_documentation("Sets a native Chat family option.")
}

fn text_method(
    component: &'static str,
    name: &'static str,
    wrap: fn(String) -> Op,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::String)],
        move |arguments| match arguments {
            [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                Ok(ComponentPayload::new(wrap(value.clone())))
            }
            _ => Err(format!("{component}.{name} expects non-empty text")),
        },
    )
    .with_documentation("Sets a native Chat family option.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Attachment", Arc::new(AttachmentMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Attachment",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(AttachmentPayload(id.clone())))
                        }
                        _ => Err("Attachment expects a non-empty stable id".into()),
                    },
                )])
                .with_methods(vec![
                    enum_method(
                        "Attachment",
                        "status",
                        &["pending", "uploading", "processing", "failed", "complete"],
                        |value| {
                            Some(Op::Status(match value {
                                "pending" => AttachmentStatus::Pending,
                                "uploading" => AttachmentStatus::Uploading,
                                "processing" => AttachmentStatus::Processing,
                                "failed" => AttachmentStatus::Failed,
                                "complete" => AttachmentStatus::Complete,
                                _ => return None,
                            }))
                        },
                    ),
                    enum_method("Attachment", "axis", &["horizontal", "vertical"], |value| {
                        Some(Op::Axis(match value {
                            "horizontal" => Axis::Horizontal,
                            "vertical" => Axis::Vertical,
                            _ => return None,
                        }))
                    }),
                    enum_method(
                        "Attachment",
                        "size",
                        &["xsmall", "small", "medium", "large"],
                        |value| {
                            Some(Op::Size(match value {
                                "xsmall" => Size::XSmall,
                                "small" => Size::Small,
                                "medium" => Size::Medium,
                                "large" => Size::Large,
                                _ => return None,
                            }))
                        },
                    ),
                ])
                .with_documentation(
                    "A file or image attachment composing ordinary children into its content.",
                ),
        )
        .expect("the built-in Attachment descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Bubble", Arc::new(BubbleMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new("Bubble", vec![], |_| {
                    Ok(ComponentPayload::new(BubblePayload))
                })])
                .with_methods(vec![
                    enum_method("Bubble", "alignment", &["start", "end"], |value| {
                        Some(Op::Alignment(match value {
                            "start" => MessageAlignment::Start,
                            "end" => MessageAlignment::End,
                            _ => return None,
                        }))
                    }),
                    enum_method(
                        "Bubble",
                        "variant",
                        &[
                            "filled",
                            "secondary",
                            "muted",
                            "tinted",
                            "outline",
                            "ghost",
                            "destructive",
                        ],
                        |value| {
                            Some(Op::BubbleVariant(match value {
                                "filled" => BubbleVariant::Filled,
                                "secondary" => BubbleVariant::Secondary,
                                "muted" => BubbleVariant::Muted,
                                "tinted" => BubbleVariant::Tinted,
                                "outline" => BubbleVariant::Outline,
                                "ghost" => BubbleVariant::Ghost,
                                "destructive" => BubbleVariant::Destructive,
                                _ => return None,
                            }))
                        },
                    ),
                ])
                .with_documentation("A message bubble whose ordinary children form its content."),
        )
        .expect("the built-in Bubble descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Marker", Arc::new(MarkerMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Marker",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(MarkerPayload(id.clone())))
                        }
                        _ => Err("Marker expects a non-empty stable id".into()),
                    },
                )])
                .with_methods(vec![
                    enum_method(
                        "Marker",
                        "variant",
                        &["plain", "separator", "border"],
                        |value| {
                            Some(Op::MarkerVariant(match value {
                                "plain" => MarkerVariant::Plain,
                                "separator" => MarkerVariant::Separator,
                                "border" => MarkerVariant::Border,
                                _ => return None,
                            }))
                        },
                    ),
                    bool_method("Marker", "loading", Op::Loading),
                    enum_method(
                        "Marker",
                        "loading_style",
                        &["spinner", "shimmer"],
                        |value| {
                            Some(Op::LoadingStyle(match value {
                                "spinner" => MarkerLoadingStyle::Spinner,
                                "shimmer" => MarkerLoadingStyle::Shimmer,
                                _ => return None,
                            }))
                        },
                    ),
                ])
                .with_documentation(
                    "A compact conversation status marker with composable children.",
                ),
        )
        .expect("the built-in Marker descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Message", Arc::new(MessageMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new("Message", vec![], |_| {
                    Ok(ComponentPayload::new(MessagePayload))
                })])
                .with_methods(vec![
                    enum_method("Message", "alignment", &["start", "end"], |value| {
                        Some(Op::Alignment(match value {
                            "start" => MessageAlignment::Start,
                            "end" => MessageAlignment::End,
                            _ => return None,
                        }))
                    }),
                    text_method("Message", "name", Op::Name),
                    text_method("Message", "time", Op::Time),
                    text_method("Message", "avatar", Op::Avatar),
                ])
                .with_documentation(
                    "A message row with an avatar, sender header, content, and footer slots.",
                ),
        )
        .expect("the built-in Message descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("ShimmerText", Arc::new(ShimmerMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "ShimmerText",
                    vec![ArgumentDescriptor::new("text", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(text)] => {
                            Ok(ComponentPayload::new(ShimmerPayload(text.clone())))
                        }
                        _ => Err("ShimmerText expects text".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "id",
                        vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                                Ok(ComponentPayload::new(Op::Id(id.clone())))
                            }
                            _ => Err("ShimmerText.id expects a non-empty stable id".into()),
                        },
                    )
                    .with_documentation("Sets an explicit stable animation identity."),
                    MethodDescriptor::new(
                        "duration_ms",
                        vec![ArgumentDescriptor::new(
                            "duration_ms",
                            ArgumentSchema::Number,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite()
                                    && *value >= 0.0
                                    && *value <= u64::MAX as f64 =>
                            {
                                Ok(ComponentPayload::new(Op::Duration(Duration::from_millis(
                                    *value as u64,
                                ))))
                            }
                            _ => Err(
                                "ShimmerText.duration_ms expects a finite non-negative duration"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Sets one shimmer sweep duration in milliseconds."),
                    MethodDescriptor::new(
                        "spread",
                        vec![ArgumentDescriptor::new("spread", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite()
                                    && *value >= f32::MIN as f64
                                    && *value <= f32::MAX as f64 =>
                            {
                                Ok(ComponentPayload::new(Op::Spread(*value as f32)))
                            }
                            _ => Err("ShimmerText.spread expects a finite f32 fraction".into()),
                        },
                    )
                    .with_documentation("Sets the relative highlight half-width."),
                    bool_method("ShimmerText", "reverse", Op::Reverse),
                    bool_method("ShimmerText", "once", Op::Once),
                ])
                .with_documentation("Theme-aware animated loading text."),
        )
        .expect("the built-in ShimmerText descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("MessageScroller", Arc::new(ScrollerMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "MessageScroller",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("item_count", ArgumentSchema::Number),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(count)]
                            if !id.trim().is_empty() =>
                        {
                            let count = count.parse::<usize>().map_err(|_| {
                                "MessageScroller item_count must be a non-negative integer"
                                    .to_string()
                            })?;
                            Ok(ComponentPayload::new(ScrollerPayload {
                                id: id.clone(),
                                item_count: count,
                            }))
                        }
                        _ => Err("MessageScroller expects a non-empty id and an item_count".into()),
                    },
                )])
                .with_methods(vec![
                    bool_method("MessageScroller", "scrollbar", Op::Scrollbar),
                    bool_method("MessageScroller", "jump_button", Op::JumpButton),
                    MethodDescriptor::new(
                        "jump_button_label",
                        vec![ArgumentDescriptor::new(
                            "jump_button_label",
                            ArgumentSchema::String,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                                Ok(ComponentPayload::new(Op::JumpButtonLabel(value.clone())))
                            }
                            _ => {
                                Err("MessageScroller.jump_button_label expects non-empty text"
                                    .into())
                            }
                        },
                    )
                    .with_documentation("Sets the jump-to-latest button label."),
                    MethodDescriptor::new(
                        "render_item",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [argument @ ComponentArgument::Callback(_)] => {
                                Ok(ComponentPayload::new(Op::RenderItem(argument.clone())))
                            }
                            _ => {
                                Err("MessageScroller.render_item(callback) expects a callback"
                                    .into())
                            }
                        },
                    )
                    .with_documentation(
                        "Renders each transcript row with managed code, receiving its index.",
                    ),
                ])
                .with_documentation(
                    "A virtualized message transcript with retained scroll and tail-following \
                     state, rendered through a managed row callback.",
                ),
        )
        .expect("the built-in MessageScroller descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chat_family_registers_six_components() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        let names = frozen
            .descriptors()
            .map(|descriptor| descriptor.name())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "Attachment",
                "Bubble",
                "Marker",
                "Message",
                "ShimmerText",
                "MessageScroller"
            ]
        );
    }
}
