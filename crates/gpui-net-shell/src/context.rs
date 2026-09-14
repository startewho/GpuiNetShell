//! Host capabilities handed to component materializers.
//!
//! A materializer often needs to reach back into the host: bind a click to the
//! managed callback table, or request a repaint. [`HostContext`] is that one
//! dependency, kept out of the component registry so the registry holds only
//! component descriptors.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::App;

use crate::abi::GpuiNetCallbacks;

/// Requests one managed re-render from inside a native callback that only has
/// `&mut App`. The managed view's `refresh` drives `Context::notify`.
pub type Invalidate = Rc<dyn Fn(&mut App)>;

/// Notifies one retained native entity subtree: a closure that calls
/// `cx.notify()` on the entity. Boxed so callers need not name the component
/// type.
pub type EntityNotifier = Rc<dyn Fn(&mut App)>;

/// Retained native entity subtrees for one window, keyed by managed entity id.
///
/// A managed `Context::notify` reaches native code as a command; the command
/// looks the entity up here and notifies it in place, so only that subtree
/// repaints. The map is rebuilt during each render (the `EntityHost`
/// materializer registers its notifier), so an entity that leaves the tree is
/// dropped.
pub type EntityHosts = Rc<RefCell<HashMap<u64, EntityNotifier>>>;

/// The native capabilities a materializer needs.
#[derive(Clone)]
pub struct HostContext {
    pub session_id: u64,
    pub callbacks: GpuiNetCallbacks,
    pub invalidate: Invalidate,
    /// The window's retained entity subtrees.
    pub entity_hosts: EntityHosts,
}

impl HostContext {
    /// Records how to notify one retained entity subtree.
    pub fn register_entity_host(&self, entity_id: u64, notifier: EntityNotifier) {
        if let Ok(mut hosts) = self.entity_hosts.try_borrow_mut() {
            hosts.insert(entity_id, notifier);
        }
    }

    /// A copy that never requests a full repaint. An entity subtree repaints
    /// only through `notify_entity`, so its callbacks must not force a window
    /// rebuild.
    pub fn without_invalidate(mut self) -> Self {
        self.invalidate = Rc::new(|_| {});
        self
    }
}
