//! `Editor`, ported from `component-shell`'s `media/editor.rs`.
//!
//! A retained source editor backed by a native `EditorState` in keyed state.
//! `component-shell` takes the state as a registered entity; this runtime
//! builds it from the component id plus an initial value and language, matching
//! how `Textarea` retains its state.

use std::sync::Arc;

use gpui::{
    AnyElement, Entity, IntoElement as _, Refineable as _, SharedString, Styled as _,
};
use gpui_component::input::{Editor, EditorState};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct EditorPayload(String);

#[derive(Clone)]
enum EditorOp {
    Value(String),
    Language(String),
    Appearance(bool),
    Bordered(bool),
    Readonly(bool),
    AriaLabel(String),
}

struct EditorMaterializer;

impl ComponentMaterializer for EditorMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<EditorPayload>()
            .ok_or_else(|| "Editor received an incompatible payload".to_string())?
            .0
            .clone();
        if request.children_len() != 0 {
            return Err("Editor does not accept children".to_string());
        }
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<EditorOp>().cloned())
            .collect::<Vec<_>>();
        let value = last_string(&operations, |op| match op {
            EditorOp::Value(value) => Some(value),
            _ => None,
        })
        .unwrap_or_default();
        let language = last_string(&operations, |op| match op {
            EditorOp::Language(value) => Some(value),
            _ => None,
        });
        let disabled = request.disabled();

        let key = SharedString::from(format!("shell-editor:{id}"));
        let state: Entity<EditorState> = request.use_keyed_state(key, move |window, cx| {
            let state = EditorState::new(window, cx).default_value(value);
            match language {
                Some(language) => state.language(language),
                None => state,
            }
        });

        let mut editor = Editor::new(&state).disabled(disabled);
        for operation in &operations {
            editor = match operation {
                EditorOp::Value(_) | EditorOp::Language(_) => editor,
                EditorOp::Appearance(value) => editor.appearance(*value),
                EditorOp::Bordered(value) => editor.bordered(*value),
                EditorOp::Readonly(value) => editor.readonly(*value),
                EditorOp::AriaLabel(value) => editor.aria_label(value.clone()),
            };
        }
        editor.style().refine(&request.take_style());
        Ok(editor.into_any_element())
    }
}

fn last_string(operations: &[EditorOp], pick: fn(&EditorOp) -> Option<&String>) -> Option<String> {
    operations.iter().filter_map(pick).next_back().cloned()
}

fn bool_method(name: &'static str, op: fn(bool) -> EditorOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(op(*value))),
            _ => Err(format!("Editor.{name}({name}) expects a boolean")),
        },
    )
    .with_documentation("Sets native Editor behavior.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Editor", Arc::new(EditorMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Editor",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "Editor")
                            .map(EditorPayload)
                            .map(ComponentPayload::new),
                        _ => Err("Editor(id) expects a non-empty string id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "value",
                        vec![ArgumentDescriptor::new("value", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                Ok(ComponentPayload::new(EditorOp::Value(value.clone())))
                            }
                            _ => Err("Editor.value expects text".into()),
                        },
                    )
                    .with_documentation("Sets the initial source text (first render only)."),
                    MethodDescriptor::new(
                        "language",
                        vec![ArgumentDescriptor::new(
                            "language",
                            ArgumentSchema::Enum(&["rust", "json", "plaintext"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => {
                                Ok(ComponentPayload::new(EditorOp::Language(value.clone())))
                            }
                            _ => Err("Editor.language expects rust, json, or plaintext".into()),
                        },
                    )
                    .with_documentation("Sets the syntax language (first render only)."),
                    bool_method("appearance", EditorOp::Appearance),
                    bool_method("bordered", EditorOp::Bordered),
                    bool_method("readonly", EditorOp::Readonly),
                    MethodDescriptor::new(
                        "aria_label",
                        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] if !value.trim().is_empty() => {
                                Ok(ComponentPayload::new(EditorOp::AriaLabel(value.clone())))
                            }
                            _ => Err("Editor.aria_label expects non-empty text".into()),
                        },
                    )
                    .with_documentation("Sets the editor accessibility label."),
                ])
                .with_documentation(
                    "A retained native source editor. Shell disabled and style are honored; \
                     children are rejected.",
                ),
        )
        .expect("the built-in Editor descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_identity_requires_text() {
        assert!(nonempty_id("", "Editor").is_err());
        assert_eq!(nonempty_id("code", "Editor").unwrap(), "code");
    }
}
