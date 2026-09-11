//! The wire vocabulary shared with the managed host.
//!
//! Every constant here is mirrored verbatim in
//! `src/GpuiNetShell/Interop/NativeProtocol.cs`. `SCHEMA_HASH` is the single
//! value both sides must agree on for a managed/native pair to run; a test in
//! each language pins its literal.
//!
//! Styling is deliberately *not* enumerated here. The managed host sends a
//! style method name and its argument; Rust resolves the name against GPUI's
//! reflected style table exactly as `gpui-shell` does. Only component behavior
//! (identity, state, activation) and the style-call envelope have codes.

/// Protocol version negotiated through [`crate::abi::gpui_net_shell_get_api`].
pub const ABI_VERSION: u32 = 1;

/// Identifies the component/operation vocabulary below. Bump whenever a
/// component id, operation code, or payload rule changes.
pub const SCHEMA_HASH: u64 = 0x6E65_7473_6865_6C6D;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

pub const COMPONENT_DIV: u32 = 1;
pub const COMPONENT_TEXT: u32 = 2;
pub const COMPONENT_BUTTON: u32 = 3;

// ---------------------------------------------------------------------------
// Component behavior operations
// ---------------------------------------------------------------------------

pub const OP_DISABLED: u16 = 20;
pub const OP_SELECTED: u16 = 21;
pub const OP_ON_CLICK: u16 = 22;

pub const OP_LABEL: u16 = 40;
pub const OP_TOOLTIP: u16 = 41;
pub const OP_LOADING: u16 = 42;
pub const OP_BUTTON_VARIANT: u16 = 43;
pub const OP_BUTTON_SIZE: u16 = 44;
pub const OP_COMPACT: u16 = 45;

// ---------------------------------------------------------------------------
// Generic style operations
// ---------------------------------------------------------------------------
//
// `a` is always the packed UTF-8 range of the style method name. `b` carries
// the argument, if the method takes one:
//
// * nullary: `b == 0`;
// * length/number: `b` is the low 32 bits of an IEEE-754 `f32`;
// * color/string: `b` is the packed UTF-8 range of the argument.

pub const OP_STYLE_NULLARY: u16 = 50;
pub const OP_STYLE_LENGTH: u16 = 51;
pub const OP_STYLE_NUMBER: u16 = 52;
pub const OP_STYLE_COLOR: u16 = 53;
pub const OP_STYLE_STRING: u16 = 54;

// ---------------------------------------------------------------------------
// Enumerations
// ---------------------------------------------------------------------------

pub const BUTTON_VARIANT_DEFAULT: u64 = 0;
pub const BUTTON_VARIANT_PRIMARY: u64 = 1;
pub const BUTTON_VARIANT_SECONDARY: u64 = 2;
pub const BUTTON_VARIANT_DANGER: u64 = 3;
pub const BUTTON_VARIANT_SUCCESS: u64 = 4;
pub const BUTTON_VARIANT_WARNING: u64 = 5;
pub const BUTTON_VARIANT_GHOST: u64 = 6;
pub const BUTTON_VARIANT_LINK: u64 = 7;

pub const BUTTON_SIZE_XSMALL: u64 = 0;
pub const BUTTON_SIZE_SMALL: u64 = 1;
pub const BUTTON_SIZE_MEDIUM: u64 = 2;
pub const BUTTON_SIZE_LARGE: u64 = 3;

// ---------------------------------------------------------------------------
// Status codes
// ---------------------------------------------------------------------------

pub const STATUS_OK: i32 = 0;
pub const STATUS_INVALID_ARGUMENT: i32 = -1;
pub const STATUS_NULL_POINTER: i32 = -2;
pub const STATUS_TRUNCATED: i32 = -3;
pub const STATUS_BAD_UTF8: i32 = -4;
pub const STATUS_BAD_INDEX: i32 = -5;
pub const STATUS_CYCLE: i32 = -6;
pub const STATUS_UNKNOWN_COMPONENT: i32 = -7;
pub const STATUS_STALE_REVISION: i32 = -8;
pub const STATUS_PANIC: i32 = -9;

#[cfg(test)]
mod tests {
    use super::*;

    /// The managed host mirrors this literal; keep them in lockstep.
    #[test]
    fn schema_hash_is_pinned() {
        assert_eq!(SCHEMA_HASH, 0x6E65_7473_6865_6C6D);
    }

    #[test]
    fn component_and_operation_ids_are_unique() {
        let components = [COMPONENT_DIV, COMPONENT_TEXT, COMPONENT_BUTTON];
        let mut sorted = components.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), components.len());

        let ops = [
            OP_DISABLED,
            OP_SELECTED,
            OP_ON_CLICK,
            OP_LABEL,
            OP_TOOLTIP,
            OP_LOADING,
            OP_BUTTON_VARIANT,
            OP_BUTTON_SIZE,
            OP_COMPACT,
            OP_STYLE_NULLARY,
            OP_STYLE_LENGTH,
            OP_STYLE_NUMBER,
            OP_STYLE_COLOR,
            OP_STYLE_STRING,
        ];
        let mut sorted = ops.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ops.len());
    }
}
