//! Panic-safe C entry points. This is the only FFI surface.
//!
//! Every exported function validates its pointers before dereferencing and
//! converts a Rust panic into a status code so an unwind never crosses the C
//! boundary.

use crate::abi::{GpuiNetCallbacks, GpuiNetShellApi};
use crate::schema::{
    ABI_VERSION, SCHEMA_HASH, STATUS_INVALID_ARGUMENT, STATUS_NULL_POINTER, STATUS_PANIC,
};

static API: GpuiNetShellApi = GpuiNetShellApi {
    struct_size: GpuiNetShellApi::struct_size(),
    abi_version: ABI_VERSION,
    schema_hash: SCHEMA_HASH,
    run_application: Some(run_application),
    _reserved: 0,
};

/// Returns the API table, or null when `requested_version` is unsupported.
///
/// The returned pointer has static lifetime.
#[no_mangle]
pub extern "C" fn gpui_net_shell_get_api(requested_version: u32) -> *const GpuiNetShellApi {
    if requested_version != ABI_VERSION {
        return std::ptr::null();
    }
    &API
}

#[no_mangle]
pub extern "C" fn gpui_net_shell_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub extern "C" fn gpui_net_shell_schema_hash() -> u64 {
    SCHEMA_HASH
}

unsafe extern "C" fn run_application(
    application_id: u64,
    callbacks: *const GpuiNetCallbacks,
) -> i32 {
    if callbacks.is_null() {
        return STATUS_NULL_POINTER;
    }
    // SAFETY: validated non-null; the caller keeps the table alive for the call.
    let callbacks = unsafe { *callbacks };
    if callbacks.struct_size < std::mem::size_of::<GpuiNetCallbacks>() as u32 {
        return STATUS_INVALID_ARGUMENT;
    }

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::host::run(application_id, callbacks)
    }));
    match outcome {
        Ok(status) => status,
        Err(_) => STATUS_PANIC,
    }
}
