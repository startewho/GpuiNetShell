//! The component registry: the one place a native component kind maps to its
//! materializer.
//!
//! This mirrors `gpui-shell`'s descriptor seam in a smaller form. Adding a
//! component is a descriptor plus a materializer; the host, decoder, and FFI
//! never name a concrete component.

use std::rc::Rc;

use gpui::{AnyElement, App};

use crate::abi::GpuiNetCallbacks;
use crate::schema::{COMPONENT_BUTTON, COMPONENT_DIV, COMPONENT_TEXT};
use crate::snapshot::Node;

/// Requests one managed re-render from inside a native callback that only has
/// `&mut App`. Produced by the owning view and captured by event closures.
pub type Invalidate = Rc<dyn Fn(&mut App)>;

/// Everything a materializer reads for one node plus the callbacks it may bind.
pub struct MaterializeContext<'a> {
    pub node: &'a Node,
    pub children: Vec<AnyElement>,
    pub session_id: u64,
    pub callbacks: &'a GpuiNetCallbacks,
    pub invalidate: Invalidate,
}

pub type Materializer = fn(&mut MaterializeContext<'_>) -> Result<AnyElement, String>;

pub struct ComponentDescriptor {
    pub id: u32,
    pub name: &'static str,
    pub materialize: Materializer,
}

pub struct ComponentRegistry {
    descriptors: Vec<ComponentDescriptor>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

impl ComponentRegistry {
    pub fn with_builtins() -> Self {
        Self {
            descriptors: vec![
                ComponentDescriptor {
                    id: COMPONENT_DIV,
                    name: "Div",
                    materialize: crate::components::div::materialize,
                },
                ComponentDescriptor {
                    id: COMPONENT_TEXT,
                    name: "Text",
                    materialize: crate::components::text::materialize,
                },
                ComponentDescriptor {
                    id: COMPONENT_BUTTON,
                    name: "Button",
                    materialize: crate::components::button::materialize,
                },
            ],
        }
    }

    pub fn descriptor(&self, id: u32) -> Option<&ComponentDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.id == id)
    }

    pub fn descriptors(&self) -> &[ComponentDescriptor] {
        &self.descriptors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_are_registered_in_stable_order() {
        let registry = ComponentRegistry::with_builtins();
        assert_eq!(
            registry
                .descriptors()
                .iter()
                .map(|descriptor| descriptor.name)
                .collect::<Vec<_>>(),
            ["Div", "Text", "Button"]
        );
    }

    #[test]
    fn every_builtin_id_resolves_and_unknown_ids_do_not() {
        let registry = ComponentRegistry::with_builtins();
        for descriptor in registry.descriptors() {
            assert_eq!(
                registry.descriptor(descriptor.id).map(|d| d.name),
                Some(descriptor.name)
            );
        }
        assert!(registry.descriptor(9_999).is_none());
    }
}
