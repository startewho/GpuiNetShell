//! Materialization, mirroring `gpui-shell`'s `materialize.rs`.
//!
//! One pass over a node's ops folds styles into a [`StyleRefinement`], collects
//! behavior (disabled, selected, on_click) and the recorded method calls, and
//! recursively materializes children. The node is then dispatched to its
//! registered [`ComponentDescriptor`], whose materializer produces the element.
//! The runtime never names a concrete component.

use gpui::{AnyElement, App, StyleRefinement, Window};

use crate::context::HostContext;
use crate::registry::{
    ComponentArgument, ComponentDescriptor, ComponentPayload, FrozenComponentRegistry,
    MaterializeRequest, RecordedComponentMethod,
};
use crate::snapshot::{Node, Op, RenderSnapshot};
use crate::style::{apply_nullary_name, apply_param, StyleArg};

/// Behavior collected from a node's ops, applied when the component is built.
#[derive(Default)]
struct Behavior {
    disabled: bool,
    selected: bool,
    on_click: Option<u64>,
    /// Recorded component methods, in declaration order.
    methods: Vec<(String, Option<StyleArg>)>,
}

/// Materializes a frozen snapshot through the catalog.
pub fn materialize(
    registry: &FrozenComponentRegistry,
    snapshot: &RenderSnapshot,
    host: &HostContext,
    window: &mut Window,
    cx: &mut App,
) -> Result<AnyElement, String> {
    materialize_node(
        registry,
        snapshot.nodes(),
        snapshot.root(),
        host,
        window,
        cx,
    )
}

fn materialize_node(
    registry: &FrozenComponentRegistry,
    nodes: &[Node],
    id: u32,
    host: &HostContext,
    window: &mut Window,
    cx: &mut App,
) -> Result<AnyElement, String> {
    let node = nodes
        .get(id as usize)
        .ok_or_else(|| format!("node {id} is outside the snapshot"))?;
    let descriptor = registry
        .descriptor(node.component)
        .ok_or_else(|| format!("component {} is not registered", node.component))?;

    let (refinement, behavior) = resolve_ops(node);
    let payload = build_payload(descriptor, node)?;
    let methods = record_methods(descriptor, &behavior.methods);

    // A child referenced by a `Slot` op is delivered by name, not as an
    // ordinary child.
    let mut slot_names: Vec<(String, u32)> = Vec::new();
    for op in &node.ops {
        if let Op::Slot(name, child) = op {
            slot_names.push((name.clone(), *child));
        }
    }

    let mut children = Vec::with_capacity(node.children.len());
    let mut slots = Vec::new();
    for child in &node.children {
        let element = materialize_node(registry, nodes, *child, host, window, cx)?;
        match slot_names.iter().find(|(_, id)| id == child) {
            Some((name, _)) => slots.push((name.clone(), element)),
            None => children.push(element),
        }
    }

    let request = MaterializeRequest::new(
        descriptor.name(),
        &payload,
        &methods,
        host,
        refinement,
        children,
        slots,
        behavior.disabled,
        behavior.selected,
        behavior.on_click,
        window,
        cx,
    );
    descriptor.materializer().materialize(request)
}

/// Runs the descriptor's constructor with the node's identity data.
fn build_payload(
    descriptor: &ComponentDescriptor,
    node: &Node,
) -> Result<ComponentPayload, String> {
    let constructor = descriptor
        .constructors()
        .first()
        .ok_or_else(|| format!("{} has no constructor", descriptor.name()))?;
    let arguments = if constructor.arguments().is_empty() {
        Vec::new()
    } else {
        vec![ComponentArgument::String(node.data.clone())]
    };
    constructor.payload(&arguments)
}

/// Turns recorded method names into owned payloads, dropping unknown names.
fn record_methods(
    descriptor: &ComponentDescriptor,
    methods: &[(String, Option<StyleArg>)],
) -> Vec<RecordedComponentMethod> {
    let mut recorded = Vec::with_capacity(methods.len());
    for (name, argument) in methods {
        let Some(method) = descriptor.method(name) else {
            continue;
        };
        if let Ok(payload) = method.record(&argument_list(argument.as_ref())) {
            recorded.push(RecordedComponentMethod::new(method.name(), payload));
        }
    }
    recorded
}

fn argument_list(argument: Option<&StyleArg>) -> Vec<ComponentArgument> {
    match argument {
        None => Vec::new(),
        Some(StyleArg::Number(value)) => vec![ComponentArgument::Number(f64::from(*value))],
        Some(StyleArg::String(value)) => vec![ComponentArgument::String(value.clone())],
    }
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
            Op::Method(name, arg) => match name.as_str() {
                "disabled" => {
                    behavior.disabled = arg.as_ref().map(StyleArg::is_truthy).unwrap_or(true)
                }
                "selected" => {
                    behavior.selected = arg.as_ref().map(StyleArg::is_truthy).unwrap_or(true)
                }
                _ => behavior.methods.push((name.clone(), arg.clone())),
            },
            Op::Callback(name, token) => {
                if name == "on_click" {
                    behavior.on_click = Some(*token);
                }
            }
            Op::Slot(..) => {}
        }
    }

    (refinement, behavior)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components;
    use crate::schema::{COMPONENT_BUTTON, COMPONENT_DIV};

    fn node(component: u32, data: &str, ops: Vec<Op>) -> Node {
        Node {
            component,
            data: data.into(),
            ops,
            children: Vec::new(),
        }
    }

    #[test]
    fn behavior_separates_shell_state_from_recorded_methods() {
        let (_, behavior) = resolve_ops(&node(
            COMPONENT_BUTTON,
            "save",
            vec![
                Op::Method("disabled".into(), Some(StyleArg::Number(1.0))),
                Op::Method("label".into(), Some(StyleArg::String("Save".into()))),
                Op::Method("primary".into(), None),
                Op::Callback("on_click".into(), 42),
            ],
        ));
        assert!(behavior.disabled);
        assert_eq!(behavior.on_click, Some(42));
        assert_eq!(
            behavior.methods,
            vec![
                ("label".into(), Some(StyleArg::String("Save".into()))),
                ("primary".into(), None),
            ]
        );
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
    fn the_catalog_records_known_methods_and_drops_unknown_ones() {
        let frozen = components::catalog();
        let descriptor = frozen.descriptor(COMPONENT_BUTTON).unwrap();
        let recorded = record_methods(
            descriptor,
            &[
                ("label".into(), Some(StyleArg::String("Save".into()))),
                ("not_a_method".into(), None),
            ],
        );
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].name(), "label");
    }

    #[test]
    fn the_constructor_payload_is_the_node_identity() {
        let frozen = components::catalog();
        let descriptor = frozen.descriptor(COMPONENT_BUTTON).unwrap();
        let payload =
            build_payload(descriptor, &node(COMPONENT_BUTTON, "save", Vec::new())).unwrap();
        assert!(payload
            .downcast_ref::<components::button::IdPayload>()
            .is_some());
    }
}
