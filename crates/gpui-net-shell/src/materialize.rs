//! Recursive dispatch from a decoded snapshot to native elements.

use gpui::AnyElement;

use crate::abi::GpuiNetCallbacks;
use crate::registry::{ComponentRegistry, Invalidate, MaterializeContext};
use crate::snapshot::Snapshot;

/// Materializes `node_id` and its children. Children are built first so a
/// parent receives finished elements, matching GPUI's consumed-element model.
pub fn materialize_node(
    registry: &ComponentRegistry,
    snapshot: &Snapshot,
    node_id: u32,
    session_id: u64,
    callbacks: &GpuiNetCallbacks,
    invalidate: &Invalidate,
) -> Result<AnyElement, String> {
    let node = snapshot
        .nodes
        .get(node_id as usize)
        .ok_or_else(|| format!("node {node_id} is outside the snapshot"))?;

    let mut children = Vec::with_capacity(node.children.len());
    for child in &node.children {
        children.push(materialize_node(
            registry, snapshot, *child, session_id, callbacks, invalidate,
        )?);
    }

    let descriptor = registry
        .descriptor(node.component)
        .ok_or_else(|| format!("component {} has no materializer", node.component))?;

    let mut ctx = MaterializeContext {
        node,
        children,
        session_id,
        callbacks,
        invalidate: invalidate.clone(),
    };
    (descriptor.materialize)(&mut ctx)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use gpui::App;

    use super::*;
    use crate::registry::ComponentRegistry;
    use crate::schema::COMPONENT_DIV;
    use crate::snapshot::Node;

    fn callbacks() -> GpuiNetCallbacks {
        GpuiNetCallbacks {
            struct_size: std::mem::size_of::<GpuiNetCallbacks>() as u32,
            _reserved: 0,
            application_started: None,
            window_closed: None,
            render: None,
            render_completed: None,
            click: None,
        }
    }

    #[test]
    fn an_out_of_range_node_fails_before_touching_gpui() {
        let registry = ComponentRegistry::with_builtins();
        let snapshot = Snapshot {
            root: 0,
            nodes: vec![Node {
                component: COMPONENT_DIV,
                data: String::new(),
                ops: Vec::new(),
                children: Vec::new(),
            }],
        };
        let invalidate: Invalidate = Rc::new(|_: &mut App| {});
        let result = materialize_node(&registry, &snapshot, 7, 1, &callbacks(), &invalidate);
        assert!(result.is_err());
    }
}
