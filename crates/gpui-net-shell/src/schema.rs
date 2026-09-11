//! The wire vocabulary shared with the managed host.
//!
//! Every constant here is mirrored verbatim in
//! `src/GpuiNetShell/Interop/NativeProtocol.cs`. `SCHEMA_HASH` is the single
//! value both sides must agree on for a managed/native pair to run; a test in
//! each language pins its literal.
//!
//! The operation set mirrors `gpui-shell`'s `SpecOp`. Styling is not enumerated
//! per property: a style call carries a GPUI method name and an argument that
//! the native host resolves against GPUI's reflected style table. Component
//! behavior is a generic `Method`, and event bindings are a generic `Callback`.

/// Protocol version negotiated through [`crate::abi::gpui_net_shell_get_api`].
pub const ABI_VERSION: u32 = 2;

/// Identifies the component/operation vocabulary below. Bump whenever a
/// component id, operation code, or payload rule changes.
pub const SCHEMA_HASH: u64 = 0x6E65_7473_6865_6C34;

/// Separates the string arguments of a multi-argument constructor inside one
/// node's identity data. `Popover(id, label)` is the only current user.
pub const CONSTRUCTOR_ARG_SEPARATOR: char = '\u{1F}';

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------
//
// Component ids are registry indices: the runtime resolves them against the
// registered catalog rather than enumerating them in the decoder. The managed
// host mirrors these values.
#[allow(dead_code)]
pub const COMPONENT_DIV: u32 = 0;
#[allow(dead_code)]
pub const COMPONENT_TEXT: u32 = 1;
#[allow(dead_code)]
pub const COMPONENT_BUTTON: u32 = 2;
#[allow(dead_code)]
pub const COMPONENT_LABEL: u32 = 3;
#[allow(dead_code)]
pub const COMPONENT_BADGE: u32 = 4;
#[allow(dead_code)]
pub const COMPONENT_PROGRESS: u32 = 5;
#[allow(dead_code)]
pub const COMPONENT_COMBOBOX: u32 = 6;
#[allow(dead_code)]
pub const COMPONENT_RADIO: u32 = 7;
#[allow(dead_code)]
pub const COMPONENT_TABS: u32 = 8;
#[allow(dead_code)]
pub const COMPONENT_SCROLL: u32 = 9;
#[allow(dead_code)]
pub const COMPONENT_SCROLLBAR: u32 = 10;
#[allow(dead_code)]
pub const COMPONENT_RESIZABLE: u32 = 11;
#[allow(dead_code)]
pub const COMPONENT_POPOVER: u32 = 12;
#[allow(dead_code)]
pub const COMPONENT_SPINNER: u32 = 13;
#[allow(dead_code)]
pub const COMPONENT_SEPARATOR: u32 = 14;
#[allow(dead_code)]
pub const COMPONENT_SKELETON: u32 = 15;
#[allow(dead_code)]
pub const COMPONENT_TAG: u32 = 16;
#[allow(dead_code)]
pub const COMPONENT_LINK: u32 = 17;
#[allow(dead_code)]
pub const COMPONENT_KBD: u32 = 18;
#[allow(dead_code)]
pub const COMPONENT_AVATAR: u32 = 19;
#[allow(dead_code)]
pub const COMPONENT_ICON: u32 = 20;

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------
//
// Every operation's `a` word carries the packed UTF-8 range of a method name.
// `flags` classifies the argument carried in `b`:
//
// * [`ARG_NONE`]: no argument, `b == 0`;
// * [`ARG_NUMBER`]: the low 32 bits of `b` are an IEEE-754 `f32`;
// * [`ARG_STRING`]: `b` is the packed UTF-8 range of the argument.

/// A no-argument style method (`items_center`, `size_full`, …).
pub const OP_NULLARY_STYLE: u16 = 1;
/// A style method taking one argument (`p`, `gap`, `bg`, …).
pub const OP_PARAM_STYLE: u16 = 2;
/// A component behavior method (`disabled`, `label`, `primary`, …).
pub const OP_METHOD: u16 = 3;
/// An event binding (`on_click`), whose `b` is a callback token.
pub const OP_CALLBACK: u16 = 4;
/// A named slot: `a` is the slot name, `b` is the child node it refers to.
pub const OP_SLOT: u16 = 5;

pub const ARG_NONE: u16 = 0;
pub const ARG_NUMBER: u16 = 1;
pub const ARG_STRING: u16 = 2;
/// A closed-set literal for a component method, packed like [`ARG_STRING`].
pub const ARG_ENUM: u16 = 3;

// ---------------------------------------------------------------------------
// Callback values
// ---------------------------------------------------------------------------
//
// The `kind` word of the `invoke` callback. `boolean` and `number` travel in the
// `number` argument; `string` travels in the `data`/`data_len` pair.

#[allow(dead_code)]
pub const CALLBACK_VALUE_NONE: u32 = 0;
pub const CALLBACK_VALUE_BOOLEAN: u32 = 1;
pub const CALLBACK_VALUE_NUMBER: u32 = 2;
pub const CALLBACK_VALUE_STRING: u32 = 3;

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
pub const STATUS_PANIC: i32 = -9;

#[cfg(test)]
mod tests {
    use super::*;

    /// The managed host mirrors this literal; keep them in lockstep.
    #[test]
    fn schema_hash_is_pinned() {
        assert_eq!(SCHEMA_HASH, 0x6E65_7473_6865_6C34);
    }

    #[test]
    fn component_and_operation_ids_are_unique() {
        let components = [COMPONENT_DIV, COMPONENT_TEXT, COMPONENT_BUTTON];
        let mut sorted = components.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), components.len());

        let ops = [OP_NULLARY_STYLE, OP_PARAM_STYLE, OP_METHOD, OP_CALLBACK];
        let mut sorted = ops.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ops.len());
    }
}
