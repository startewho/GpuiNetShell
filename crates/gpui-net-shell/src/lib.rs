//! `gpui-net-shell`: a C#-hosted shell runtime for GPUI.
//!
//! The managed host owns the application model and publishes an element
//! description into a flat arena. Rust owns the GPUI application, window, event
//! loop, validation, and materialization, and never exposes a GPUI type across
//! the C ABI.
//!
//! The component vocabulary in [`schema`] is mirrored in
//! `src/GpuiNetShell/Interop/NativeProtocol.cs`. Adding a component is a
//! descriptor in [`registry`] plus a materializer in [`components`].

mod abi;
mod components;
mod ffi;
mod host;
mod materialize;
mod registry;
mod schema;
mod snapshot;
mod style;

pub use abi::{
    GpuiNetArena, GpuiNetCallbacks, GpuiNetChild, GpuiNetNode, GpuiNetOp, GpuiNetShellApi,
};
pub use ffi::{gpui_net_shell_abi_version, gpui_net_shell_get_api, gpui_net_shell_schema_hash};
pub use registry::ComponentRegistry;
pub use snapshot::{Node, Op, Snapshot};
