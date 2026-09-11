//! Host capabilities handed to component materializers.
//!
//! A materializer often needs to reach back into the host: bind a click to the
//! managed callback table, or request a repaint. [`HostContext`] is that one
//! dependency, kept out of the component registry so the registry holds only
//! component descriptors.

use std::rc::Rc;

use gpui::App;

use crate::abi::GpuiNetCallbacks;

/// Requests one managed re-render from inside a native callback that only has
/// `&mut App`. The managed view's `refresh` drives `Context::notify`.
pub type Invalidate = Rc<dyn Fn(&mut App)>;

/// The native capabilities a materializer needs.
#[derive(Clone)]
pub struct HostContext {
    pub session_id: u64,
    pub callbacks: GpuiNetCallbacks,
    pub invalidate: Invalidate,
}
