//! `Tabs`: a tab bar whose selection is controlled by the managed host.
//!
//! Options are carried as plain data (newline separated labels plus one callback
//! token per option), so the bar is self-contained: it builds `gpui-component`
//! `Tab` values from the option list and reports the chosen index through the
//! option's token.

use std::sync::Arc;

use gpui::{div, AnyElement, ParentElement as _, SharedString, Styled as _};
use gpui_component::tab::{Tab, TabBar};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

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
        _ => Err("Tabs(id) expects one string".into()),
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
                .ok_or("Tabs.options expects a string")?;
            Ok(ComponentPayload::new(OptionsOp(
                value.split('\n').map(str::to_owned).collect(),
            )))
        },
    )
    .with_documentation("Sets the tab labels, newline separated.")
}

fn tokens_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "tokens",
        vec![ArgumentDescriptor::new("tokens", ArgumentSchema::String)],
        |args| {
            let value = args
                .first()
                .and_then(ComponentArgument::as_str)
                .ok_or("Tabs.tokens expects a string")?;
            let tokens = value
                .split(',')
                .filter(|part| !part.is_empty())
                .filter_map(|part| part.parse::<u64>().ok())
                .collect();
            Ok(ComponentPayload::new(TokensOp(tokens)))
        },
    )
    .with_documentation("Sets one callback token per tab, comma separated.")
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
    .with_documentation("Sets the selected tab index.")
}

struct TabsMaterializer;

impl ComponentMaterializer for TabsMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Tabs received an incompatible payload".to_string())?
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

        let tabs: Vec<Tab> = options
            .iter()
            .map(|option| Tab::new().label(SharedString::from(option.clone())))
            .collect();

        let host = request.host().clone();
        let bar = TabBar::new(SharedString::from(id))
            .selected_index(selected)
            .children(tabs)
            .on_click(move |index, _window, cx| {
                let token = tokens.get(*index).copied().unwrap_or(0);
                if token != 0 {
                    if let Some(click) = host.callbacks.click {
                        // SAFETY: the managed callback copies anything it keeps.
                        unsafe {
                            let _ = click(host.session_id, token);
                        }
                    }
                }
                (host.invalidate)(cx);
            });

        request.finish(div().w_full().child(bar))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Tabs", Arc::new(TabsMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Tabs",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    id_payload,
                )])
                .with_methods(vec![options_method(), tokens_method(), selected_method()])
                .with_documentation("A controlled tab bar."),
        )
        .expect("the built-in Tabs descriptor is valid");
}
