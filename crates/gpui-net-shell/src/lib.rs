//! `gpui-net-shell`: a C#-hosted shell runtime for GPUI.
//!
//! The managed host owns the application model and publishes an element
//! description into a flat arena. Rust owns the GPUI application, window, event
//! loop, validation, and materialization, and never exposes a GPUI type across
//! the C ABI.
//!
//! The shape follows `gpui-shell`: [`snapshot`] decodes the description,
//! [`style`] resolves reflected style method names, and [`materialize`]
//! dispatches one component at a time. The wire vocabulary in [`schema`] is
//! mirrored in `src/GpuiNetShell/Interop/NativeProtocol.cs`.

mod abi;
mod components;
mod context;
mod ffi;
mod host;
mod materialize;
// The registry exposes a descriptor API that the built-in catalog uses a subset
// of today; the rest is the seam components are added through.
#[allow(dead_code)]
mod registry;
mod root;
mod schema;
mod snapshot;
mod style;
mod view;

pub use abi::{
    GpuiNetArena, GpuiNetCallbacks, GpuiNetChild, GpuiNetNode, GpuiNetOp, GpuiNetShellApi,
};
pub use ffi::{gpui_net_shell_abi_version, gpui_net_shell_get_api, gpui_net_shell_schema_hash};
pub use snapshot::{Node, Op, RenderSnapshot, Snapshot};
pub use style::StyleArg;
