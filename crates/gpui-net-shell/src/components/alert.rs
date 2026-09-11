//! `Alert`, ported from `component-shell`'s `display/alert.rs`.
//!
//! A message banner with semantic variant constructors (`Alert`, `InfoAlert`,
//! `SuccessAlert`, `WarningAlert`, `ErrorAlert`). It rejects ordinary children;
//! `title`, `banner`, `visible`, and `size` are recorded methods.

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, Refineable as _, Styled as _};
use gpui_component::{
    alert::{Alert, AlertVariant},
    Sizable as _, Size,
};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct AlertPayload {
    id: String,
    message: String,
    variant: AlertVariant,
}

#[derive(Clone)]
enum AlertOp {
    Title(String),
    Banner,
    Visible(bool),
    Size(Size),
}

struct AlertMaterializer;

impl AlertMaterializer {
    fn component<'a>(
        payload: &ComponentPayload,
        operations: impl IntoIterator<Item = &'a AlertOp>,
    ) -> Result<Alert, String> {
        let payload = payload
            .downcast_ref::<AlertPayload>()
            .ok_or_else(|| "Alert received an incompatible payload".to_string())?;
        let alert = match payload.variant {
            AlertVariant::Default => Alert::new(payload.id.clone(), payload.message.clone()),
            AlertVariant::Info => Alert::info(payload.id.clone(), payload.message.clone()),
            AlertVariant::Success => Alert::success(payload.id.clone(), payload.message.clone()),
            AlertVariant::Warning => Alert::warning(payload.id.clone(), payload.message.clone()),
            AlertVariant::Error => Alert::error(payload.id.clone(), payload.message.clone()),
        };
        Ok(operations
            .into_iter()
            .fold(alert, |alert, operation| match operation {
                AlertOp::Title(title) => alert.title(title.clone()),
                AlertOp::Banner => alert.banner(),
                AlertOp::Visible(visible) => alert.visible(*visible),
                AlertOp::Size(size) => alert.with_size(*size),
            }))
    }
}

impl ComponentMaterializer for AlertMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        if request.children_len() != 0 {
            return Err("Alert does not accept children".to_string());
        }
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<AlertOp>());
        let mut alert = Self::component(request.payload(), operations)?;
        alert.style().refine(&request.take_style());
        Ok(alert.into_any_element())
    }
}

fn constructor(export: &'static str, variant: AlertVariant) -> ConstructorDescriptor {
    ConstructorDescriptor::new(
        export,
        vec![
            ArgumentDescriptor::new("id", ArgumentSchema::String),
            ArgumentDescriptor::new("message", ArgumentSchema::String),
        ],
        move |arguments| match arguments {
            [ComponentArgument::String(id), ComponentArgument::String(message)] => {
                Ok(ComponentPayload::new(AlertPayload {
                    id: nonempty_id(id, "Alert")?,
                    message: message.clone(),
                    variant,
                }))
            }
            _ => Err(format!("{export}(id, message) expects two strings")),
        },
    )
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Alert", Arc::new(AlertMaterializer))
                .with_constructors(vec![
                    constructor("Alert", AlertVariant::Default),
                    constructor("InfoAlert", AlertVariant::Info),
                    constructor("SuccessAlert", AlertVariant::Success),
                    constructor("WarningAlert", AlertVariant::Warning),
                    constructor("ErrorAlert", AlertVariant::Error),
                ])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "title",
                        vec![ArgumentDescriptor::new("title", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(title)] => {
                                Ok(ComponentPayload::new(AlertOp::Title(title.clone())))
                            }
                            _ => Err("Alert.title(title) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the alert title."),
                    MethodDescriptor::new("banner", Vec::new(), |_| {
                        Ok(ComponentPayload::new(AlertOp::Banner))
                    })
                    .with_documentation("Uses the full-width banner presentation."),
                    MethodDescriptor::new(
                        "visible",
                        vec![ArgumentDescriptor::new("visible", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(visible)] => {
                                Ok(ComponentPayload::new(AlertOp::Visible(*visible)))
                            }
                            _ => Err("Alert.visible(visible) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Controls whether the alert is rendered."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => Ok(ComponentPayload::new(AlertOp::Size(Size::XSmall))),
                                "small" => Ok(ComponentPayload::new(AlertOp::Size(Size::Small))),
                                "medium" => Ok(ComponentPayload::new(AlertOp::Size(Size::Medium))),
                                "large" => Ok(ComponentPayload::new(AlertOp::Size(Size::Large))),
                                _ => Err(format!("unsupported Alert size `{value}`")),
                            },
                            _ => Err("Alert.size expects a semantic size literal".into()),
                        },
                    )
                    .with_documentation("Sets the alert's semantic size."),
                ])
                .with_documentation(
                    "A message banner with semantic default, info, success, warning, and error \
                     constructors.",
                ),
        )
        .expect("the built-in Alert descriptor is valid");
}
