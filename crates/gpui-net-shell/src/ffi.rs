//! Panic-safe C entry points. This is the only FFI surface.
//!
//! Every exported function validates its pointers before dereferencing and
//! converts a Rust panic into a status code so an unwind never crosses the C
//! boundary.

use crate::abi::{GpuiNetCallbacks, GpuiNetShellApi};
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
    configure: Some(configure),
    set_theme: Some(set_theme),
    open_window: Some(open_window),
    close_window: Some(close_window),
    notify_entity: Some(notify_entity),
    set_window_title: Some(set_window_title),
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

unsafe extern "C" fn configure(session_id: u64, flags: u32) -> i32 {
    guard(|| Ok(crate::host::configure(session_id, flags)))
}

unsafe extern "C" fn set_theme(
    session_id: u64,
    mode: u32,
    colors: *const u8,
    colors_len: u32,
) -> i32 {
    guard(|| {
        let colors = read_utf8(colors, colors_len)?;
        Ok(crate::host::set_theme(session_id, mode, colors))
    })
}

unsafe extern "C" fn open_window(
    parent_session: u64,
    flags: u32,
    title: *const u8,
    title_len: u32,
) -> i64 {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let title = match read_utf8(title, title_len) {
            Ok(title) => title,
            Err(status) => return status as i64,
        };
        crate::host::open_window(parent_session, flags, title)
    }));
    match outcome {
        Ok(session_id) => session_id,
        Err(_) => crate::schema::STATUS_PANIC as i64,
    }
}

unsafe extern "C" fn close_window(session_id: u64) -> i32 {
    guard(|| Ok(crate::host::close_window(session_id)))
}

unsafe extern "C" fn notify_entity(session_id: u64, entity_id: u64) -> i32 {
    guard(|| Ok(crate::host::notify_entity(session_id, entity_id)))
}

unsafe extern "C" fn set_window_title(session_id: u64, title: *const u8, title_len: u32) -> i32 {
    guard(|| {
        let title = read_utf8(title, title_len)?;
        Ok(crate::host::set_window_title(session_id, title))
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
