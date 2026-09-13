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
/// `a`/`b`/`c` are interpreted by `code`. String operations pack
/// `(offset << 32) | len` into `a` with `b == 0`. Scalar operations use `a`
/// with `b == 0`. A method taking two arguments uses `b` for the first and `c`
/// for the second. `flags` classifies the argument(s).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuiNetOp {
    pub node: u32,
    pub code: u16,
    pub flags: u16,
    pub a: u64,
    pub b: u64,
    pub c: u64,
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
    /// Fills a caller-owned buffer with newline-separated rows of tab-separated
    /// fields for a row-snapshot callback token. Reports the required byte
    /// count through `out_len`; returns [`crate::schema::STATUS_TRUNCATED`] when
    /// `capacity` was too small so the caller can retry with a larger buffer.
    pub resolve_rows: Option<
        unsafe extern "C" fn(
            session_id: u64,
            token: u64,
            buffer: *mut u8,
            capacity: u32,
            out_len: *mut u32,
        ) -> i32,
    >,
    /// Renders a subtree for an element callback (P7). `arguments` is a UTF-8
    /// buffer of newline-separated callback arguments. The managed side fills
    /// `out_arena`/`out_root`, which the host decodes and materializes
    /// synchronously before returning.
    pub render_element: Option<
        unsafe extern "C" fn(
            session_id: u64,
            token: u64,
            arguments: *const u8,
            arguments_len: u32,
            out_arena: *mut GpuiNetArena,
            out_root: *mut u32,
        ) -> i32,
    >,
    /// Delivers a window input event (mouse, wheel, keyboard) to the managed
    /// host. `kind` is one of the `INPUT_*` constants; `a`/`b`/`c` carry
    /// position or delta; `text` carries a key name.
    pub input_event: Option<
        unsafe extern "C" fn(
            session_id: u64,
            kind: u32,
            flags: u32,
            a: f32,
            b: f32,
            c: f32,
            text: *const u8,
            text_len: u32,
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
    /// Sets per-session window options before running. `flags` bit 0 = custom titlebar.
    pub configure: Option<unsafe extern "C" fn(session_id: u64, flags: u32) -> i32>,
    /// Applies a theme. `mode` 0 = light, 1 = dark, 2 = system; `colors` is an
    /// optional UTF-8, newline-separated `name=#rrggbb` override list.
    pub set_theme: Option<
        unsafe extern "C" fn(session_id: u64, mode: u32, colors: *const u8, colors_len: u32) -> i32,
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
        // node(4) + code(2) + flags(2) = 8, then three aligned u64 words.
        assert_eq!(std::mem::size_of::<GpuiNetOp>(), 32);
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
