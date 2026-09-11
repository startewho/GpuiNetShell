//! Materialization, mirroring `gpui-shell`'s `materialize.rs`.
//!
//! A decoded node carries a [`Component`], style calls, behavior `Method`s, and
//! `Callback`s. [`resolve_ops`] folds the style calls into a [`StyleRefinement`]
//! and accumulates the rest into a [`Behavior`]; [`materialize_node`] recurses
//! into children; [`materialize_component`] dispatches on the component and
//! [`finish`] applies the refinement and children. Which style name, behavior
//! method, or callback means what is decided here, not in the decoder.
//!
//! This is deliberately one dispatch, not a registry of trait objects: adding a
//! component is a variant and a match arm, and a component's behavior is read
//! from the same `Behavior` the whole shell uses.

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{div, AnyElement, App, IntoElement, SharedString, StyleRefinement, Styled};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::{Disableable as _, Selectable as _, Sizable as _, Size};

use crate::abi::GpuiNetCallbacks;
use crate::schema::{COMPONENT_BUTTON, COMPONENT_DIV, COMPONENT_TEXT};
use crate::snapshot::{Node, Op, Snapshot};
use crate::style::{apply_nullary_name, apply_param, StyleArg};

/// Requests one managed re-render from inside a native callback that only has
/// `&mut App`.
pub type Invalidate = Rc<dyn Fn(&mut App)>;

/// What constructed a node.
#[derive(Clone, Debug, PartialEq)]
enum Component {
    Div,
    Text(String),
    Button(String),
}

fn component_of(node: &Node) -> Result<Component, String> {
    match node.component {
        COMPONENT_DIV => Ok(Component::Div),
        COMPONENT_TEXT => Ok(Component::Text(node.data.clone())),
        COMPONENT_BUTTON => Ok(Component::Button(node.data.clone())),
        other => Err(format!("component {other} has no materializer")),
    }
}

/// A button's visual variant, selected by the method name (`primary()`,
/// `danger()`, …), matching `gpui-component`'s own builder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Variant {
    #[default]
    Default,
    Primary,
    Secondary,
    Danger,
    Success,
    Warning,
    Ghost,
    Link,
}

/// Behavior collected from a node's ops, applied when the component is built.
#[derive(Default)]
struct Behavior {
    disabled: bool,
    selected: bool,
    on_click: Option<u64>,
    label: Option<SharedString>,
    tooltip: Option<SharedString>,
    loading: bool,
    compact: bool,
    variant: Variant,
    size: Option<u64>,
}

/// Materializes a snapshot's root.
pub fn materialize(
    snapshot: &Snapshot,
    session_id: u64,
    callbacks: &GpuiNetCallbacks,
    invalidate: &Invalidate,
) -> Result<AnyElement, String> {
    materialize_node(snapshot, snapshot.root, session_id, callbacks, invalidate)
}

/// The same walk as [`materialize`], carrying the node being built.
fn materialize_node(
    snapshot: &Snapshot,
    id: u32,
    session_id: u64,
    callbacks: &GpuiNetCallbacks,
    invalidate: &Invalidate,
) -> Result<AnyElement, String> {
    let node = snapshot
        .nodes
        .get(id as usize)
        .ok_or_else(|| format!("node {id} is outside the snapshot"))?;
    let component = component_of(node)?;
    let (refinement, behavior) = resolve_ops(node);

    let mut children = Vec::with_capacity(node.children.len());
    for child in &node.children {
        children.push(materialize_node(
            snapshot, *child, session_id, callbacks, invalidate,
        )?);
    }

    Ok(materialize_component(
        component, behavior, refinement, children, session_id, callbacks, invalidate,
    ))
}

/// Folds a node's ops into a style refinement and a behavior.
fn resolve_ops(node: &Node) -> (StyleRefinement, Behavior) {
    let mut refinement = StyleRefinement::default();
    let mut behavior = Behavior::default();

    for op in &node.ops {
        match op {
            Op::NullaryStyle(name) => {
                if let Some(next) = apply_nullary_name(name, refinement.clone()) {
                    refinement = next;
                }
            }
            Op::ParamStyle(name, arg) => {
                if let Ok(next) = apply_param(name, arg, refinement.clone()) {
                    refinement = next;
                }
            }
            Op::Method(name, arg) => apply_behavior(&mut behavior, name, arg.as_ref()),
            Op::Callback(name, token) => apply_callback(&mut behavior, name, *token),
        }
    }

    (refinement, behavior)
}

/// Maps a component behavior method onto the behavior it sets.
fn apply_behavior(behavior: &mut Behavior, name: &str, arg: Option<&StyleArg>) {
    let flag = arg.map(StyleArg::is_truthy);
    match name {
        "disabled" => behavior.disabled = flag.unwrap_or(true),
        "selected" => behavior.selected = flag.unwrap_or(true),
        "loading" => behavior.loading = flag.unwrap_or(true),
        "compact" => behavior.compact = true,
        "label" => behavior.label = text(arg),
        "tooltip" => behavior.tooltip = text(arg),
        "size" => behavior.size = number(arg).map(|value| value as u64),
        "primary" => behavior.variant = Variant::Primary,
        "secondary" => behavior.variant = Variant::Secondary,
        "danger" => behavior.variant = Variant::Danger,
        "success" => behavior.variant = Variant::Success,
        "warning" => behavior.variant = Variant::Warning,
        "ghost" => behavior.variant = Variant::Ghost,
        "link" => behavior.variant = Variant::Link,
        _ => {}
    }
}

/// Maps a callback method name onto the behavior it binds.
fn apply_callback(behavior: &mut Behavior, name: &str, token: u64) {
    if name == "on_click" {
        behavior.on_click = Some(token);
    }
}

fn text(arg: Option<&StyleArg>) -> Option<SharedString> {
    arg.and_then(|value| value.as_str().ok())
        .map(SharedString::from)
}

fn number(arg: Option<&StyleArg>) -> Option<f32> {
    arg.and_then(|value| value.as_f32().ok())
}

fn resolved_size(value: u64) -> Size {
    match value {
        0 => Size::XSmall,
        1 => Size::Small,
        3 => Size::Large,
        _ => Size::Medium,
    }
}

fn with_variant(button: Button, variant: Variant) -> Button {
    match variant {
        Variant::Default => button,
        Variant::Primary => button.primary(),
        Variant::Secondary => button.secondary(),
        Variant::Danger => button.danger(),
        Variant::Success => button.success(),
        Variant::Warning => button.warning(),
        Variant::Ghost => button.ghost(),
        Variant::Link => button.link(),
    }
}

/// Builds the concrete element for `component`.
#[allow(clippy::too_many_arguments)]
fn materialize_component(
    component: Component,
    behavior: Behavior,
    refinement: StyleRefinement,
    children: Vec<AnyElement>,
    session_id: u64,
    callbacks: &GpuiNetCallbacks,
    invalidate: &Invalidate,
) -> AnyElement {
    match component {
        Component::Div => finish(div(), refinement, children),
        Component::Text(value) => {
            finish(div().child(SharedString::from(value)), refinement, children)
        }
        Component::Button(id) => {
            let mut button = Button::new(SharedString::from(id))
                .loading(behavior.loading)
                .disabled(behavior.disabled)
                .selected(behavior.selected);

            if behavior.compact {
                button = button.compact();
            }
            if let Some(size) = behavior.size {
                button = button.with_size(resolved_size(size));
            }
            button = with_variant(button, behavior.variant);
            if let Some(label) = behavior.label.clone() {
                button = button.label(label);
            }
            if let Some(tooltip) = behavior.tooltip.clone() {
                button = button.tooltip(tooltip);
            }
            if let Some(token) = behavior.on_click {
                let callbacks = *callbacks;
                let invalidate = invalidate.clone();
                button = button.on_click(move |_event, _window, cx| {
                    if let Some(click) = callbacks.click {
                        // SAFETY: managed callback; it copies anything it keeps.
                        unsafe {
                            let _ = click(session_id, token);
                        }
                    }
                    invalidate(cx);
                });
            }

            finish(button, refinement, children)
        }
    }
}

/// Applies the refinement and children, and turns the element into an
/// `AnyElement` — the shell's single exit point for a built node.
fn finish<E>(mut element: E, refinement: StyleRefinement, children: Vec<AnyElement>) -> AnyElement
where
    E: Styled + ParentElement + IntoElement + 'static,
{
    element.style().refine(&refinement);
    element.extend(children);
    element.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(component: u32, data: &str, ops: Vec<Op>) -> Node {
        Node {
            component,
            data: data.into(),
            ops,
            children: Vec::new(),
        }
    }

    #[test]
    fn a_button_node_decodes_to_its_component_and_identity() {
        let component = component_of(&node(COMPONENT_BUTTON, "save", Vec::new())).unwrap();
        assert_eq!(component, Component::Button("save".into()));
    }

    #[test]
    fn behavior_methods_accumulate_in_order() {
        let (_, behavior) = resolve_ops(&node(
            COMPONENT_BUTTON,
            "save",
            vec![
                Op::Method("label".into(), Some(StyleArg::String("First".into()))),
                Op::Method("label".into(), Some(StyleArg::String("Last".into()))),
                Op::Method("danger".into(), None),
                Op::Method("size".into(), Some(StyleArg::Number(3.0))),
                Op::Method("loading".into(), Some(StyleArg::Number(1.0))),
                Op::Method("disabled".into(), Some(StyleArg::Number(1.0))),
                Op::Callback("on_click".into(), 42),
            ],
        ));
        assert_eq!(behavior.label.as_deref(), Some("Last"));
        assert_eq!(behavior.variant, Variant::Danger);
        assert_eq!(behavior.size, Some(3));
        assert!(behavior.loading);
        assert!(behavior.disabled);
        assert_eq!(behavior.on_click, Some(42));
    }

    #[test]
    fn style_calls_fold_into_the_refinement() {
        let (refinement, _) = resolve_ops(&node(
            COMPONENT_DIV,
            "",
            vec![
                Op::NullaryStyle("items_center".into()),
                Op::ParamStyle("p".into(), StyleArg::Number(16.0)),
            ],
        ));
        assert_eq!(refinement.align_items, Some(gpui::AlignItems::Center));
        assert_eq!(refinement.padding.top, Some(gpui::px(16.0).into()));
    }

    #[test]
    fn unknown_methods_are_inert() {
        let (_, behavior) = resolve_ops(&node(
            COMPONENT_BUTTON,
            "save",
            vec![Op::Method("not_a_method".into(), None)],
        ));
        assert_eq!(behavior.variant, Variant::Default);
        assert_eq!(behavior.on_click, None);
    }
}
