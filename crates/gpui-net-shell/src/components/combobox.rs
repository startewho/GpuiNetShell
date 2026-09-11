//! `Combobox`: a trigger button that opens the root's popup with the option
//! list. Selection is carried to the managed host through per-option callback
//! tokens, so no value crosses the ABI.
//!
//! The component is declarative: the managed host declares the options, the
//! selected index, and one callback token per option; the native side renders a
//! trigger and asks [`crate::host::open_popup`] to show the list. Choosing an
//! item runs that option's token through the ordinary `click` callback.

use std::sync::Arc;

use gpui::{AnyElement, SharedString};
use gpui_component::button::Button;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::root::PopupItem;

#[derive(Clone)]
struct IdPayload(String);

#[derive(Clone)]
struct OptionsOp(Vec<String>);

#[derive(Clone)]
struct TokensOp(Vec<u64>);

#[derive(Clone)]
struct SelectedOp(usize);

fn id_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(id)] => Ok(ComponentPayload::new(IdPayload(id.clone()))),
        _ => Err("Combobox(id) expects one string".into()),
    }
}

fn options_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "options",
        vec![ArgumentDescriptor::new("options", ArgumentSchema::String)],
        |args| {
            let value = args
                .first()
                .and_then(ComponentArgument::as_str)
                .ok_or("Combobox.options expects a string")?;
            Ok(ComponentPayload::new(OptionsOp(
                value.split('\n').map(str::to_owned).collect(),
            )))
        },
    )
    .with_documentation("Sets the option labels, newline separated.")
}

fn tokens_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "tokens",
        vec![ArgumentDescriptor::new("tokens", ArgumentSchema::String)],
        |args| {
            let value = args
                .first()
                .and_then(ComponentArgument::as_str)
                .ok_or("Combobox.tokens expects a string")?;
            let tokens = value
                .split(',')
                .filter(|part| !part.is_empty())
                .filter_map(|part| part.parse::<u64>().ok())
                .collect();
            Ok(ComponentPayload::new(TokensOp(tokens)))
        },
    )
    .with_documentation("Sets one callback token per option, comma separated.")
}

fn selected_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "selected",
        vec![ArgumentDescriptor::new("selected", ArgumentSchema::Number)],
        |args| {
            let index = args
                .first()
                .and_then(ComponentArgument::as_f64)
                .unwrap_or(0.0);
            Ok(ComponentPayload::new(SelectedOp(index.max(0.0) as usize)))
        },
    )
    .with_documentation("Sets the selected option index.")
}

struct ComboboxMaterializer;

impl ComponentMaterializer for ComboboxMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Combobox received an incompatible payload".to_string())?
            .0
            .clone();

        let mut options: Vec<String> = Vec::new();
        let mut tokens: Vec<u64> = Vec::new();
        let mut selected = 0usize;
        for method in request.methods() {
            match method.name() {
                "options" => {
                    if let Some(op) = method.payload().downcast_ref::<OptionsOp>() {
                        options = op.0.clone();
                    }
                }
                "tokens" => {
                    if let Some(op) = method.payload().downcast_ref::<TokensOp>() {
                        tokens = op.0.clone();
                    }
                }
                "selected" => {
                    if let Some(op) = method.payload().downcast_ref::<SelectedOp>() {
                        selected = op.0;
                    }
                }
                _ => {}
            }
        }

        let label = options
            .get(selected)
            .cloned()
            .unwrap_or_else(|| "Select".to_owned());
        let items: Vec<PopupItem> = options
            .iter()
            .enumerate()
            .map(|(index, option)| PopupItem {
                label: option.clone(),
                token: tokens.get(index).copied().unwrap_or(0),
            })
            .collect();

        let session_id = request.host().session_id;
        let button = Button::new(SharedString::from(id))
            .label(SharedString::from(label))
            .on_click(move |_event, _window, _cx| {
                let _ = crate::host::open_popup(session_id, items.clone());
            });

        request.finish(button)
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Combobox", Arc::new(ComboboxMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Combobox",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    id_payload,
                )])
                .with_methods(vec![options_method(), tokens_method(), selected_method()])
                .with_documentation("A trigger that opens a popup of options."),
        )
        .expect("the built-in Combobox descriptor is valid");
}
