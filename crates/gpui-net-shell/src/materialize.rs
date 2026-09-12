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
    ChildElement, ComponentArgument, ComponentDescriptor, ComponentPayload,
    FrozenComponentRegistry, MaterializeRequest, NodeFactory, RecordedComponentMethod,
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
    methods: Vec<(String, Vec<StyleArg>)>,
}

/// Materializes a frozen snapshot through the catalog.
pub fn materialize(
    registry: &FrozenComponentRegistry,
    snapshot: &RenderSnapshot,
    host: &HostContext,
    window: &mut Window,
    cx: &mut App,
) -> Result<AnyElement, String> {
    let factory = NodeFactory::new(registry, snapshot.snapshot(), host);
    materialize_node(&factory, snapshot.root(), window, cx)
}

/// Materializes one node, and recursively its ordinary children.
///
/// Named-slot children are left as node ids in the request so a component only
/// pays for them when it takes the slot.
pub(crate) fn materialize_node(
    factory: &NodeFactory,
    id: u32,
    window: &mut Window,
    cx: &mut App,
) -> Result<AnyElement, String> {
    let nodes = &factory.snapshot().nodes;
    let node = nodes
        .get(id as usize)
        .ok_or_else(|| format!("node {id} is outside the snapshot"))?;
    let descriptor = factory
        .registry()
        .descriptor(node.component)
        .ok_or_else(|| format!("component {} is not registered", node.component))?;

    let (refinement, behavior) = resolve_ops(node, descriptor);
    let payload = build_payload(descriptor, node)?;
    let methods = record_methods(descriptor, &behavior.methods);

    // A child referenced by a `Slot` op is delivered by name, not as an
    // ordinary child, and materializes only when the component takes it.
    let mut slots: Vec<(String, u32)> = Vec::new();
    for op in &node.ops {
        if let Op::Slot(name, child) = op {
            slots.push((name.clone(), *child));
        }
    }

    let mut children = Vec::with_capacity(node.children.len());
    for child in &node.children {
        if slots.iter().any(|(_, id)| id == child) {
            continue;
        }
        let element = materialize_node(factory, *child, window, cx)?;
        let component = nodes
            .get(*child as usize)
            .and_then(|child| factory.registry().descriptor(child.component))
            .map_or("Unknown", |descriptor| descriptor.name());
        children.push(ChildElement::new(component, element));
    }

    let request = MaterializeRequest::new(
        descriptor.name(),
        &payload,
        &methods,
        factory.clone(),
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
///
/// A descriptor with one constructor uses it directly, packing multiple string
/// arguments into the node's data separated by
/// [`crate::schema::CONSTRUCTOR_ARG_SEPARATOR`]. A descriptor with several named
/// constructors (such as `Separator`/`VerticalSeparator`, or the `Alert`
/// variants) encodes the export name first, then its arguments, all separated
/// by the same character.
fn build_payload(
    descriptor: &ComponentDescriptor,
    node: &Node,
) -> Result<ComponentPayload, String> {
    let constructors = descriptor.constructors();
    match constructors {
        [only] => {
            let arity = only.arguments().len();
            let arguments = if arity == 0 {
                Vec::new()
            } else if arity == 1 {
                vec![ComponentArgument::String(node.data.clone())]
            } else {
                split_arguments(&node.data, arity, descriptor.name())?
            };
            only.payload(&arguments)
        }
        many => {
            let mut parts = node.data.split(crate::schema::CONSTRUCTOR_ARG_SEPARATOR);
            let export = parts.next().unwrap_or_default();
            let constructor = many
                .iter()
                .find(|constructor| constructor.export() == export)
                .ok_or_else(|| {
                    format!("{} has no constructor named `{export}`", descriptor.name())
                })?;
            let arguments = parts
                .map(|part| ComponentArgument::String(part.to_string()))
                .collect::<Vec<_>>();
            if arguments.len() != constructor.arguments().len() {
                return Err(format!(
                    "{} constructor `{export}` expects {} arguments, got {}",
                    descriptor.name(),
                    constructor.arguments().len(),
                    arguments.len()
                ));
            }
            constructor.payload(&arguments)
        }
    }
}

fn split_arguments(
    encoded: &str,
    arity: usize,
    component: &str,
) -> Result<Vec<ComponentArgument>, String> {
    let parts = encoded
        .split(crate::schema::CONSTRUCTOR_ARG_SEPARATOR)
        .collect::<Vec<_>>();
    if parts.len() != arity {
        return Err(format!(
            "{component} expects {arity} constructor arguments, got {}",
            parts.len()
        ));
    }
    Ok(parts
        .into_iter()
        .map(|part| ComponentArgument::String(part.to_string()))
        .collect())
}

/// Turns recorded method names into owned payloads, dropping unknown names.
fn record_methods(
    descriptor: &ComponentDescriptor,
    methods: &[(String, Vec<StyleArg>)],
) -> Vec<RecordedComponentMethod> {
    let mut recorded = Vec::with_capacity(methods.len());
    for (name, arguments) in methods {
        let Some(method) = descriptor.method(name) else {
            continue;
        };
        if let Ok(payload) = method.record(&argument_list(arguments)) {
            recorded.push(RecordedComponentMethod::new(method.name(), payload));
        }
    }
    recorded
}

fn argument_list(arguments: &[StyleArg]) -> Vec<ComponentArgument> {
    arguments
        .iter()
        .map(|argument| match argument {
            StyleArg::Number(value) => ComponentArgument::Number(f64::from(*value)),
            StyleArg::String(value) => ComponentArgument::String(value.clone()),
            StyleArg::Enum(value) => ComponentArgument::Enum(value.clone()),
            StyleArg::Callback(token) => ComponentArgument::Callback(*token),
            StyleArg::Element(node) => ComponentArgument::Element(*node),
        })
        .collect()
}

/// Folds a node's ops into a style refinement and a behavior.
///
/// A method the component declares is always a component method; only a name the
/// descriptor does not declare falls back to shell behavior (`disabled`,
/// `selected`). Without this, a component whose own method is called `selected`
/// — `Tabs`, `Combobox` — would never receive it.
fn resolve_ops(node: &Node, descriptor: &ComponentDescriptor) -> (StyleRefinement, Behavior) {
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
            Op::Method(name, args) => {
                if descriptor.method(name).is_some() {
                    behavior.methods.push((name.clone(), args.clone()));
                } else {
                    match name.as_str() {
                        "disabled" => {
                            behavior.disabled =
                                args.first().map(StyleArg::is_truthy).unwrap_or(true)
                        }
                        "selected" => {
                            behavior.selected =
                                args.first().map(StyleArg::is_truthy).unwrap_or(true)
                        }
                        _ => behavior.methods.push((name.clone(), args.clone())),
                    }
                }
            }
            Op::Callback(name, token) => {
                if name == "on_click" {
                    behavior.on_click = Some(*token);
                } else {
                    // A callback passed as a component method argument, such as
                    // `Radio.on_change` or `Popover.on_open_change`.
                    behavior
                        .methods
                        .push((name.clone(), vec![StyleArg::Callback(*token)]));
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
        let frozen = components::catalog();
        let descriptor = frozen.descriptor(COMPONENT_BUTTON).unwrap();
        let (_, behavior) = resolve_ops(
            &node(
                COMPONENT_BUTTON,
                "save",
                vec![
                    Op::Method("disabled".into(), vec![StyleArg::Number(1.0)]),
                    Op::Method("label".into(), vec![StyleArg::String("Save".into())]),
                    Op::Method("primary".into(), Vec::new()),
                    Op::Callback("on_click".into(), 42),
                ],
            ),
            descriptor,
        );
        assert!(behavior.disabled);
        assert_eq!(behavior.on_click, Some(42));
        assert_eq!(
            behavior.methods,
            vec![
                ("label".into(), vec![StyleArg::String("Save".into())]),
                ("primary".into(), Vec::new()),
            ]
        );
    }

    #[test]
    fn a_declared_method_named_selected_reaches_the_component() {
        let frozen = components::catalog();
        let descriptor = frozen.descriptor(crate::schema::COMPONENT_TABS).unwrap();
        let (_, behavior) = resolve_ops(
            &node(
                crate::schema::COMPONENT_TABS,
                "pages",
                vec![Op::Method("selected".into(), vec![StyleArg::Number(1.0)])],
            ),
            descriptor,
        );
        assert!(!behavior.selected);
        assert_eq!(
            behavior.methods,
            vec![("selected".into(), vec![StyleArg::Number(1.0)])]
        );
    }

    #[test]
    fn style_calls_fold_into_the_refinement() {
        let frozen = components::catalog();
        let descriptor = frozen.descriptor(COMPONENT_DIV).unwrap();
        let (refinement, _) = resolve_ops(
            &node(
                COMPONENT_DIV,
                "",
                vec![
                    Op::NullaryStyle("items_center".into()),
                    Op::ParamStyle("p".into(), StyleArg::Number(16.0)),
                ],
            ),
            descriptor,
        );
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
                ("label".into(), vec![StyleArg::String("Save".into())]),
                ("not_a_method".into(), Vec::new()),
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
