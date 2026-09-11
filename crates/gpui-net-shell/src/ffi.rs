//! Panic-safe C entry points. This is the only FFI surface.
//!
//! Every exported function validates its pointers before dereferencing and
//! converts a Rust panic into a status code so an unwind never crosses the C
//! boundary.

use crate::abi::{GpuiNetCallbacks, GpuiNetShellApi};
use crate::root::NotificationLevel;
use crate::schema::{
    ABI_VERSION, SCHEMA_HASH, STATUS_BAD_UTF8, STATUS_INVALID_ARGUMENT, STATUS_NULL_POINTER,
    STATUS_PANIC,
};

static API: GpuiNetShellApi = GpuiNetShellApi {
    struct_size: GpuiNetShellApi::struct_size(),
    abi_version: ABI_VERSION,
    schema_hash: SCHEMA_HASH,
    run_application: Some(run_application),
    invalidate: Some(invalidate),
    open_dialog: Some(open_dialog),
    close_dialog: Some(close_dialog),
    push_notification: Some(push_notification),
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

    guard(|| Ok(crate::host::run(application_id, callbacks)))
}

unsafe extern "C" fn invalidate(session_id: u64) -> i32 {
    guard(|| Ok(crate::host::invalidate(session_id)))
}

unsafe extern "C" fn open_dialog(
    session_id: u64,
    title: *const u8,
    title_len: u32,
    body: *const u8,
    body_len: u32,
) -> i32 {
    guard(|| {
        let title = read_utf8(title, title_len)?;
        let body = read_utf8(body, body_len)?;
        Ok(crate::host::open_dialog(session_id, title, body))
    })
}

unsafe extern "C" fn close_dialog(session_id: u64) -> i32 {
    guard(|| Ok(crate::host::close_dialog(session_id)))
}

unsafe extern "C" fn push_notification(
    session_id: u64,
    message: *const u8,
    message_len: u32,
    level: u32,
) -> i32 {
    guard(|| {
        let message = read_utf8(message, message_len)?;
        Ok(crate::host::push_notification(
            session_id,
            message,
            NotificationLevel::from_wire(level),
        ))
    })
}

/// Runs a fallible body, turning a panic into [`STATUS_PANIC`].
fn guard(body: impl FnOnce() -> Result<i32, i32>) -> i32 {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    match outcome {
        Ok(Ok(status)) => status,
        Ok(Err(status)) => status,
        Err(_) => STATUS_PANIC,
    }
}

/// Reads a borrowed UTF-8 string. A zero length permits a null pointer.
unsafe fn read_utf8(pointer: *const u8, length: u32) -> Result<String, i32> {
    if length == 0 {
        return Ok(String::new());
    }
    if pointer.is_null() {
        return Err(STATUS_NULL_POINTER);
    }
    // SAFETY: the caller guarantees a valid buffer for `length` bytes.
    let bytes = unsafe { std::slice::from_raw_parts(pointer, length as usize) };
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| STATUS_BAD_UTF8)
}
