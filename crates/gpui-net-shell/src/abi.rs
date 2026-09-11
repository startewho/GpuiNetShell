//! The C layouts exchanged with the managed host.
//!
//! These records are the only representation Rust accepts from C#. They are
//! validated before any field is trusted; see [`crate::snapshot`].

/// One element in a managed render arena.
///
/// `data_offset`/`data_len` address the arena's UTF-8 buffer and carry the
/// element's identity string: a Button id or Text content. A node with no data
/// has both fields zero.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuiNetNode {
    pub component: u32,
    pub flags: u32,
    pub data_offset: u32,
    pub data_len: u32,
}

/// One operation applied to an element.
///
/// `a`/`b` are interpreted by `code`. String operations pack
/// `(offset << 32) | len` into `a` with `b == 0`. Scalar operations use `a`
/// with `b == 0`. `flags` is reserved and must be zero.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuiNetOp {
    pub node: u32,
    pub code: u16,
    pub flags: u16,
    pub a: u64,
    pub b: u64,
}

/// A parent/child edge. `parent` and `child` index [`GpuiNetArena::nodes`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuiNetChild {
    pub parent: u32,
    pub child: u32,
}

/// A borrowed, managed-owned element description.
///
/// The host only reads these buffers during the `render` callback and copies
/// everything it keeps before returning. Every pointer/length pair is
/// validated; a zero length permits a null pointer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GpuiNetArena {
    pub nodes: *const GpuiNetNode,
    pub nodes_len: u32,
    pub _pad0: u32,
    pub ops: *const GpuiNetOp,
    pub ops_len: u32,
    pub _pad1: u32,
    pub children: *const GpuiNetChild,
    pub children_len: u32,
    pub _pad2: u32,
    pub utf8: *const u8,
    pub utf8_len: u32,
    pub _pad3: u32,
}

impl GpuiNetArena {
    pub const fn empty() -> Self {
        Self {
            nodes: std::ptr::null(),
            nodes_len: 0,
            _pad0: 0,
            ops: std::ptr::null(),
            ops_len: 0,
            _pad1: 0,
            children: std::ptr::null(),
            children_len: 0,
            _pad2: 0,
            utf8: std::ptr::null(),
            utf8_len: 0,
            _pad3: 0,
        }
    }
}

/// Managed entry points the host calls back into.
///
/// `render` fills `arena`/`root` for one generation. `render_completed`
/// acknowledges a decoded generation (or reports its failure). `click` delivers
/// a Button activation by callback token. `retire_callbacks` is called when a
/// published snapshot is dropped, letting the managed host release the event
/// handlers that generation registered. `struct_size` lets native code validate
/// the available prefix.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuiNetCallbacks {
    pub struct_size: u32,
    pub _reserved: u32,
    pub application_started: Option<unsafe extern "C" fn(application_id: u64) -> i32>,
    pub window_closed: Option<unsafe extern "C" fn(session_id: u64, status: i32) -> i32>,
    pub render: Option<
        unsafe extern "C" fn(
            session_id: u64,
            generation: u64,
            arena: *mut GpuiNetArena,
            root: *mut u32,
        ) -> i32,
    >,
    pub render_completed:
        Option<unsafe extern "C" fn(session_id: u64, generation: u64, status: i32) -> i32>,
    pub click: Option<unsafe extern "C" fn(session_id: u64, token: u64) -> i32>,
    pub retire_callbacks: Option<unsafe extern "C" fn(session_id: u64, generation: u64) -> i32>,
    /// Delivers a typed value to a callback token. `kind` is one of the
    /// `CALLBACK_VALUE_*` constants; `number` carries a boolean (`1.0`/`0.0`) or
    /// a number, and `data`/`data_len` carry a UTF-8 string when `kind` is
    /// [`crate::schema::CALLBACK_VALUE_STRING`].
    pub invoke: Option<
        unsafe extern "C" fn(
            session_id: u64,
            token: u64,
            kind: u32,
            number: f64,
            data: *const u8,
            data_len: u32,
        ) -> i32,
    >,
}

impl std::fmt::Debug for GpuiNetCallbacks {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GpuiNetCallbacks")
            .field("struct_size", &self.struct_size)
            .finish_non_exhaustive()
    }
}

/// The versioned API table returned by [`crate::ffi::gpui_net_shell_get_api`].
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuiNetShellApi {
    pub struct_size: u32,
    pub abi_version: u32,
    pub schema_hash: u64,
    pub run_application: Option<
        unsafe extern "C" fn(application_id: u64, callbacks: *const GpuiNetCallbacks) -> i32,
    >,
    /// Requests a re-render of one session from any thread.
    pub invalidate: Option<unsafe extern "C" fn(session_id: u64) -> i32>,
    /// Opens a dialog with a UTF-8 title and body.
    pub open_dialog: Option<
        unsafe extern "C" fn(
            session_id: u64,
            title: *const u8,
            title_len: u32,
            body: *const u8,
            body_len: u32,
        ) -> i32,
    >,
    /// Closes the topmost dialog.
    pub close_dialog: Option<unsafe extern "C" fn(session_id: u64) -> i32>,
    /// Opens a sheet on an edge (0 left, 1 right, 2 top, 3 bottom) with a
    /// UTF-8 title and body.
    pub open_sheet: Option<
        unsafe extern "C" fn(
            session_id: u64,
            placement: u32,
            title: *const u8,
            title_len: u32,
            body: *const u8,
            body_len: u32,
        ) -> i32,
    >,
    /// Closes the sheet.
    pub close_sheet: Option<unsafe extern "C" fn(session_id: u64) -> i32>,
    /// Posts a UTF-8 notification with a severity level.
    pub push_notification: Option<
        unsafe extern "C" fn(
            session_id: u64,
            message: *const u8,
            message_len: u32,
            level: u32,
        ) -> i32,
    >,
    pub _reserved: u64,
}

impl GpuiNetShellApi {
    pub const fn struct_size() -> u32 {
        std::mem::size_of::<Self>() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_layouts_are_stable() {
        assert_eq!(std::mem::size_of::<GpuiNetNode>(), 16);
        assert_eq!(std::mem::size_of::<GpuiNetChild>(), 8);
        // node(4) + code(2) + flags(2) = 8, then two aligned u64 words.
        assert_eq!(std::mem::size_of::<GpuiNetOp>(), 24);
    }

    #[test]
    fn arena_records_are_pointer_sized_and_aligned() {
        assert_eq!(
            std::mem::align_of::<GpuiNetArena>(),
            std::mem::align_of::<*const u8>()
        );
        assert_eq!(std::mem::align_of::<GpuiNetOp>(), 8);
    }
}
