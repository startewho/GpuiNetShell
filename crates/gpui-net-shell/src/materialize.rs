//! Materialization, mirroring `gpui-shell`'s `materialize.rs`.
//!
//! One pass over a node's ops folds styles into a [`StyleRefinement`], collects
//! behavior (disabled, selected, on_click) and the recorded method calls, and
//! recursively materializes children. The node is then dispatched to its
//! registered [`ComponentDescriptor`], whose materializer produces the element.
//! The runtime never names a concrete component.

use std::rc::Rc;

use gpui::{AnyElement, App, StyleRefinement, Window};

use crate::context::HostContext;
use crate::element_events::{ElementEvent, ElementEvents};
use crate::registry::{
    ArgumentSchema, ChildElement, ComponentArgument, ComponentDescriptor, ComponentPayload,
    FrozenComponentRegistry, MaterializeRequest, NodeFactory, PreparedNode,
    RecordedComponentMethod,
};
use crate::snapshot::{Node, Op, RenderSnapshot, Snapshot};
use crate::style::{apply_nullary, apply_param, StyleArg};

/// Behavior collected from a node's ops, applied when the component is built.
#[derive(Default)]
struct Behavior {
    disabled: bool,
    selected: bool,
    /// Element events the node subscribed to, resolved once.
    events: ElementEvents,
    /// Recorded component methods, in declaration order, as `(method code, args)`.
    methods: Vec<(u64, Vec<StyleArg>)>,
}

/// Resolves every node of `snapshot` once, storing a [`PreparedNode`] per node.
///
/// Called when a description is built — a dirty render or an element callback —
/// never on a clean repaint. Materialization then only borrows the result, so a
/// repaint does not re-resolve ops, rebuild payloads, or re-record methods.
pub fn prepare(registry: &FrozenComponentRegistry, snapshot: &mut Snapshot) -> Result<(), String> {
    snapshot.fingerprint = crate::snapshot::compute_fingerprint(&snapshot.nodes);
    let mut prepared = Vec::with_capacity(snapshot.nodes.len());
    for node in &snapshot.nodes {
        let descriptor = registry
            .descriptor(node.component)
            .ok_or_else(|| format!("component {} is not registered", node.component))?;
        let (style, behavior) = resolve_ops(node, descriptor, registry);
        let payload = build_payload(descriptor, node)?;
        let methods = record_methods(descriptor, &behavior.methods, registry);

        let mut slots: Vec<(String, u32)> = Vec::new();
        for op in &node.ops {
            if let Op::Slot(name, child) = op {
                slots.push((name.clone(), *child));
            }
        }
        let children = node
            .children
            .iter()
            .copied()
            .filter(|child| !slots.iter().any(|(_, id)| id == child))
            .collect();

        prepared.push(PreparedNode {
            style,
            payload,
            methods,
            slots,
            children,
            disabled: behavior.disabled,
            selected: behavior.selected,
            events: behavior.events,
        });
    }
    snapshot.prepared = prepared;
    Ok(())
}

/// Materializes a frozen snapshot through the catalog.
pub fn materialize(
    registry: &Rc<FrozenComponentRegistry>,
    snapshot: &RenderSnapshot,
    host: &HostContext,
    window: &mut Window,
    cx: &mut App,
) -> Result<AnyElement, String> {
    let factory = NodeFactory::new(registry, snapshot.snapshot(), host);
    materialize_node(&factory, snapshot.root(), window, cx)
}

/// Materializes one node, and recursively its ordinary children, from the
/// description resolved by [`prepare`].
pub(crate) fn materialize_node(
    factory: &NodeFactory,
    id: u32,
    window: &mut Window,
    cx: &mut App,
) -> Result<AnyElement, String> {
    let snapshot = factory.snapshot();
    let prepared = snapshot
        .prepared
        .get(id as usize)
        .ok_or_else(|| format!("node {id} was not prepared"))?;
    let node = snapshot
        .nodes
        .get(id as usize)
        .ok_or_else(|| format!("node {id} is outside the snapshot"))?;
    let descriptor = factory
        .registry()
        .descriptor(node.component)
        .ok_or_else(|| format!("component {} is not registered", node.component))?;

    // A child referenced by a `Slot` op is delivered by name, not as an
    // ordinary child; `prepared.children` already excludes them.
    let mut children = Vec::with_capacity(prepared.children.len());
    for child in &prepared.children {
        let element = materialize_node(factory, *child, window, cx)?;
        let component = snapshot
            .nodes
            .get(*child as usize)
            .and_then(|child| factory.registry().descriptor(child.component))
            .map_or("Unknown", |descriptor| descriptor.name());
        children.push(ChildElement::new(component, element));
    }

    let request = MaterializeRequest::new(
        descriptor.name(),
        &prepared.payload,
        &prepared.methods,
        factory.clone(),
        prepared.style.clone(),
        children,
        prepared.slots.clone(),
        prepared.disabled,
        prepared.selected,
        &prepared.events,
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

/// Turns recorded method codes into owned payloads, dropping unknown ones.
fn record_methods(
    descriptor: &ComponentDescriptor,
    methods: &[(u64, Vec<StyleArg>)],
    registry: &FrozenComponentRegistry,
) -> Vec<RecordedComponentMethod> {
    let mut recorded = Vec::with_capacity(methods.len());
    for (code, arguments) in methods {
        let Some(name) = registry.method_name(*code) else {
            continue;
        };
        let Some(method) = descriptor.method(name) else {
            continue;
        };
        // Coerce each argument to the kind the method declared. The managed
        // surface has no boolean channel, so a boolean arrives as a number (or
        // a string); without this a `ComponentArgument::Boolean` matcher would
        // reject it and the method would be dropped silently.
        let schemas = method.arguments();
        let list = arguments
            .iter()
            .enumerate()
            .map(|(index, argument)| {
                coerce_argument(
                    argument,
                    schemas.get(index).map(|descriptor| descriptor.schema()),
                )
            })
            .collect::<Vec<_>>();
        if let Ok(payload) = method.record(&list) {
            recorded.push(RecordedComponentMethod::new(method.name(), payload));
        }
    }
    recorded
}

/// Applies a method's declared schema to a recorded argument.
fn coerce_argument(argument: &StyleArg, schema: Option<ArgumentSchema>) -> ComponentArgument {
    match (argument, schema) {
        (StyleArg::Number(value), Some(ArgumentSchema::Boolean)) => {
            ComponentArgument::Boolean(*value != 0.0 && !value.is_nan())
        }
        (StyleArg::String(value), Some(ArgumentSchema::Boolean)) => {
            ComponentArgument::Boolean(!value.is_empty())
        }
        (StyleArg::Enum(value), Some(ArgumentSchema::Enum(_))) => {
            ComponentArgument::Enum(value.clone())
        }
        (StyleArg::String(value), Some(ArgumentSchema::Enum(_))) => {
            ComponentArgument::Enum(value.clone())
        }
        (StyleArg::Number(value), _) => ComponentArgument::Number(f64::from(*value)),
        (StyleArg::String(value), _) => ComponentArgument::String(value.clone()),
        (StyleArg::Enum(value), _) => ComponentArgument::Enum(value.clone()),
        (StyleArg::Callback(token), _) => ComponentArgument::Callback(*token),
        (StyleArg::Element(node), _) => ComponentArgument::Element(*node),
    }
}

/// Folds a node's ops into a style refinement and a behavior.
///
/// A method the component declares is always a component method; only a name the
/// descriptor does not declare falls back to shell behavior (`disabled`,
/// `selected`). Without this, a component whose own method is called `selected`
/// — `Tabs`, `Combobox` — would never receive it.
fn resolve_ops(
    node: &Node,
    descriptor: &ComponentDescriptor,
    registry: &FrozenComponentRegistry,
) -> (StyleRefinement, Behavior) {
    let mut refinement = StyleRefinement::default();
    let mut behavior = Behavior::default();

    for op in &node.ops {
        match op {
            Op::NullaryStyle(code) => {
                refinement = apply_nullary(*code, refinement);
            }
            Op::ParamStyle(code, arg) => {
                // Move the refinement through the call instead of cloning it.
                let current = std::mem::take(&mut refinement);
                if let Ok(next) = apply_param(*code, arg, current) {
                    refinement = next;
                }
            }
            Op::Method(code, args) => {
                let name = registry.method_name(*code);
                let declared = name.and_then(|name| descriptor.method(name)).is_some();
                if !declared {
                    match name {
                        Some("disabled") => {
                            behavior.disabled =
                                args.first().map(StyleArg::is_truthy).unwrap_or(true);
                            continue;
                        }
                        Some("selected") => {
                            behavior.selected =
                                args.first().map(StyleArg::is_truthy).unwrap_or(true);
                            continue;
                        }
                        _ => {}
                    }
                }
                behavior.methods.push((*code, args.clone()));
            }
            Op::Callback(name, token) => {
                if let Some(event) = ElementEvent::from_wire_name(name) {
                    // An element event the managed host subscribed to.
                    behavior.events.push(event, *token);
                } else {
                    // A callback passed as a component method argument, such as
                    // `Radio.on_change` or `Popover.on_open_change`.
                    behavior.methods.push((
                        crate::schema::method_code(name),
                        vec![StyleArg::Callback(*token)],
                    ));
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
    use crate::schema::{method_code, COMPONENT_BUTTON, COMPONENT_DIV};

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
                    Op::Method(method_code("disabled"), vec![StyleArg::Number(1.0)]),
                    Op::Method(method_code("label"), vec![StyleArg::String("Save".into())]),
                    Op::Method(method_code("primary"), Vec::new()),
                    Op::Callback("on_click".into(), 42),
                ],
            ),
            descriptor,
            &frozen,
        );
        assert!(behavior.disabled);
        assert_eq!(behavior.events.get(ElementEvent::Click), Some(42));
        assert_eq!(
            behavior.methods,
            vec![
                (method_code("label"), vec![StyleArg::String("Save".into())]),
                (method_code("primary"), Vec::new()),
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
                vec![Op::Method(
                    method_code("selected"),
                    vec![StyleArg::Number(1.0)],
                )],
            ),
            descriptor,
            &frozen,
        );
        assert!(!behavior.selected);
        assert_eq!(
            behavior.methods,
            vec![(method_code("selected"), vec![StyleArg::Number(1.0)])]
        );
    }

    #[test]
    fn a_div_carries_the_generic_on_click_behavior() {
        let frozen = components::catalog();
        let descriptor = frozen.descriptor(COMPONENT_DIV).unwrap();
        let (_, behavior) = resolve_ops(
            &node(COMPONENT_DIV, "", vec![Op::Callback("on_click".into(), 7)]),
            descriptor,
            &frozen,
        );
        assert_eq!(behavior.events.get(ElementEvent::Click), Some(7));
        assert!(behavior.methods.is_empty());
    }

    #[test]
    fn element_events_are_classified_and_other_callbacks_stay_methods() {
        let frozen = components::catalog();
        let descriptor = frozen.descriptor(COMPONENT_DIV).unwrap();
        let (_, behavior) = resolve_ops(
            &node(
                COMPONENT_DIV,
                "",
                vec![
                    Op::Callback("on_mouse_move".into(), 1),
                    Op::Callback("on_hover".into(), 2),
                    Op::Callback("on_change".into(), 3),
                ],
            ),
            descriptor,
            &frozen,
        );
        assert_eq!(behavior.events.get(ElementEvent::MouseMove), Some(1));
        assert_eq!(behavior.events.get(ElementEvent::Hover), Some(2));
        assert!(behavior.events.needs_element_id());
        // A non-event callback is recorded as a component method and dropped
        // later because `Div` declares no `on_change`.
        assert_eq!(behavior.methods.len(), 1);
    }

    #[test]
    fn a_div_records_its_stable_element_id() {
        let frozen = components::catalog();
        let descriptor = frozen.descriptor(COMPONENT_DIV).unwrap();
        let recorded = record_methods(
            descriptor,
            &[(
                method_code("element_id"),
                vec![StyleArg::String("row-1".into())],
            )],
            &frozen,
        );
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].name(), "element_id");
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
                    Op::NullaryStyle(crate::style::nullary_index("items_center").unwrap()),
                    Op::ParamStyle(
                        crate::style::param_index("p").unwrap(),
                        StyleArg::Number(16.0),
                    ),
                ],
            ),
            descriptor,
            &frozen,
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
                (method_code("label"), vec![StyleArg::String("Save".into())]),
                (method_code("not_a_method"), Vec::new()),
            ],
            &frozen,
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

    #[test]
    fn prepare_resolves_nodes_and_routes_slots() {
        let frozen = components::catalog();
        let mut snapshot = Snapshot {
            root: 0,
            nodes: vec![
                Node {
                    component: COMPONENT_DIV,
                    data: String::new(),
                    ops: vec![
                        Op::NullaryStyle(crate::style::nullary_index("items_center").unwrap()),
                        Op::Slot("content".into(), 1),
                    ],
                    children: vec![1],
                },
                Node {
                    component: crate::schema::COMPONENT_TEXT,
                    data: "hi".into(),
                    ops: Vec::new(),
                    children: Vec::new(),
                },
            ],
            prepared: Vec::new(),
            fingerprint: 0,
        };

        prepare(&frozen, &mut snapshot).unwrap();

        assert_ne!(snapshot.fingerprint(), 0);
        assert_eq!(snapshot.prepared.len(), 2);
        assert_eq!(
            snapshot.prepared[0].style.align_items,
            Some(gpui::AlignItems::Center)
        );
        assert_eq!(snapshot.prepared[0].slots, vec![("content".to_string(), 1)]);
        assert!(
            snapshot.prepared[0].children.is_empty(),
            "a slot child is not an ordinary child"
        );
    }
}
