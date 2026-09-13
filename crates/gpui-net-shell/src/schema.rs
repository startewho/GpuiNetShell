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
pub const ABI_VERSION: u32 = 6;

/// Identifies the component/operation vocabulary below. Bump whenever a
/// component id, operation code, or payload rule changes.
pub const SCHEMA_HASH: u64 = 0x6E65_7473_6865_6C51;

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
#[allow(dead_code)]
pub const COMPONENT_COLLAPSIBLE: u32 = 21;
#[allow(dead_code)]
pub const COMPONENT_PAGINATION: u32 = 22;
#[allow(dead_code)]
pub const COMPONENT_RATING: u32 = 23;
#[allow(dead_code)]
pub const COMPONENT_CLIPBOARD: u32 = 24;
#[allow(dead_code)]
pub const COMPONENT_BREADCRUMB: u32 = 25;
#[allow(dead_code)]
pub const COMPONENT_GROUP_BOX: u32 = 26;
#[allow(dead_code)]
pub const COMPONENT_STATUS_BAR: u32 = 27;
#[allow(dead_code)]
pub const COMPONENT_ALERT: u32 = 28;
#[allow(dead_code)]
pub const COMPONENT_TOOLTIP: u32 = 29;
#[allow(dead_code)]
pub const COMPONENT_HOVER_CARD: u32 = 30;
#[allow(dead_code)]
pub const COMPONENT_DROPDOWN_MENU: u32 = 31;
#[allow(dead_code)]
pub const COMPONENT_DROPDOWN_BUTTON: u32 = 32;
#[allow(dead_code)]
pub const COMPONENT_TAB: u32 = 33;
#[allow(dead_code)]
pub const COMPONENT_TAB_BAR: u32 = 34;
#[allow(dead_code)]
pub const COMPONENT_LIST: u32 = 35;
#[allow(dead_code)]
pub const COMPONENT_SELECT: u32 = 36;
#[allow(dead_code)]
pub const COMPONENT_DATA_TABLE: u32 = 37;
#[allow(dead_code)]
pub const COMPONENT_ACCORDION_ITEM: u32 = 38;
#[allow(dead_code)]
pub const COMPONENT_ACCORDION: u32 = 39;
#[allow(dead_code)]
pub const COMPONENT_STEPPER_ITEM: u32 = 40;
#[allow(dead_code)]
pub const COMPONENT_STEPPER: u32 = 41;
#[allow(dead_code)]
pub const COMPONENT_DESCRIPTION_ITEM: u32 = 42;
#[allow(dead_code)]
pub const COMPONENT_DESCRIPTION_LIST: u32 = 43;
#[allow(dead_code)]
pub const COMPONENT_FIELD: u32 = 44;
#[allow(dead_code)]
pub const COMPONENT_FORM: u32 = 45;
#[allow(dead_code)]
pub const COMPONENT_INPUT: u32 = 46;
#[allow(dead_code)]
pub const COMPONENT_NUMBER_INPUT: u32 = 47;
#[allow(dead_code)]
pub const COMPONENT_TEXTAREA: u32 = 48;
#[allow(dead_code)]
pub const COMPONENT_OTP_INPUT: u32 = 49;
#[allow(dead_code)]
pub const COMPONENT_SLIDER: u32 = 50;
#[allow(dead_code)]
pub const COMPONENT_COLOR_PICKER: u32 = 51;
#[allow(dead_code)]
pub const COMPONENT_CALENDAR: u32 = 52;
#[allow(dead_code)]
pub const COMPONENT_DATE_PICKER: u32 = 53;
#[allow(dead_code)]
pub const COMPONENT_MENU_ITEM: u32 = 54;
#[allow(dead_code)]
pub const COMPONENT_MENU_SEPARATOR: u32 = 55;
#[allow(dead_code)]
pub const COMPONENT_MENU: u32 = 56;
#[allow(dead_code)]
pub const COMPONENT_MENU_BAR: u32 = 57;
#[allow(dead_code)]
pub const COMPONENT_SIDEBAR_MENU_ITEM: u32 = 58;
#[allow(dead_code)]
pub const COMPONENT_SIDEBAR_MENU: u32 = 59;
#[allow(dead_code)]
pub const COMPONENT_SIDEBAR_HEADER: u32 = 60;
#[allow(dead_code)]
pub const COMPONENT_SIDEBAR_FOOTER: u32 = 61;
#[allow(dead_code)]
pub const COMPONENT_SIDEBAR: u32 = 62;
#[allow(dead_code)]
pub const COMPONENT_SIDEBAR_TOGGLE_BUTTON: u32 = 63;
#[allow(dead_code)]
pub const COMPONENT_SETTING_ITEM: u32 = 64;
#[allow(dead_code)]
pub const COMPONENT_SETTING_GROUP: u32 = 65;
#[allow(dead_code)]
pub const COMPONENT_SETTING_PAGE: u32 = 66;
#[allow(dead_code)]
pub const COMPONENT_SETTINGS: u32 = 67;
#[allow(dead_code)]
pub const COMPONENT_TREE_ITEM: u32 = 68;
#[allow(dead_code)]
pub const COMPONENT_TREE: u32 = 69;
#[allow(dead_code)]
pub const COMPONENT_TABLE_HEADER: u32 = 70;
#[allow(dead_code)]
pub const COMPONENT_TABLE_BODY: u32 = 71;
#[allow(dead_code)]
pub const COMPONENT_TABLE_FOOTER: u32 = 72;
#[allow(dead_code)]
pub const COMPONENT_TABLE_ROW: u32 = 73;
#[allow(dead_code)]
pub const COMPONENT_TABLE_HEAD: u32 = 74;
#[allow(dead_code)]
pub const COMPONENT_TABLE_CELL: u32 = 75;
#[allow(dead_code)]
pub const COMPONENT_TABLE_CAPTION: u32 = 76;
#[allow(dead_code)]
pub const COMPONENT_TABLE: u32 = 77;
#[allow(dead_code)]
pub const COMPONENT_COMMAND_ITEM: u32 = 78;
#[allow(dead_code)]
pub const COMPONENT_COMMAND_GROUP: u32 = 79;
#[allow(dead_code)]
pub const COMPONENT_COMMAND_SEPARATOR: u32 = 80;
#[allow(dead_code)]
pub const COMPONENT_COMMAND: u32 = 81;
#[allow(dead_code)]
pub const COMPONENT_ATTACHMENT: u32 = 82;
#[allow(dead_code)]
pub const COMPONENT_BUBBLE: u32 = 83;
#[allow(dead_code)]
pub const COMPONENT_MARKER: u32 = 84;
#[allow(dead_code)]
pub const COMPONENT_MESSAGE: u32 = 85;
#[allow(dead_code)]
pub const COMPONENT_SHIMMER_TEXT: u32 = 86;
#[allow(dead_code)]
pub const COMPONENT_MESSAGE_SCROLLER: u32 = 87;
#[allow(dead_code)]
pub const COMPONENT_RADIO_GROUP: u32 = 88;
#[allow(dead_code)]
pub const COMPONENT_DIALOG: u32 = 89;
#[allow(dead_code)]
pub const COMPONENT_ALERT_DIALOG: u32 = 90;
#[allow(dead_code)]
pub const COMPONENT_SHEET: u32 = 91;
#[allow(dead_code)]
pub const COMPONENT_NOTIFICATION: u32 = 92;
#[allow(dead_code)]
pub const COMPONENT_EDITOR: u32 = 93;
#[allow(dead_code)]
pub const COMPONENT_NATIVE_MENU_ITEM: u32 = 94;
#[allow(dead_code)]
pub const COMPONENT_NATIVE_MENU_SEPARATOR: u32 = 95;
#[allow(dead_code)]
pub const COMPONENT_NATIVE_MENU_TRIGGER: u32 = 96;
#[allow(dead_code)]
pub const COMPONENT_CONTEXT_MENU_ITEM: u32 = 97;
#[allow(dead_code)]
pub const COMPONENT_CONTEXT_MENU_SEPARATOR: u32 = 98;
#[allow(dead_code)]
pub const COMPONENT_CONTEXT_MENU: u32 = 99;

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
/// An element argument: `b` is the index of the child node to materialize.
pub const ARG_ELEMENT: u16 = 4;
/// A two-argument method: `b` is a packed string, `c` is a callback token.
pub const ARG_STRING_CALLBACK: u16 = 5;

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
// Input events
// ---------------------------------------------------------------------------
//
// The `kind` word of the `input_event` callback.

pub const INPUT_KEY_DOWN: u32 = 0;
pub const INPUT_KEY_UP: u32 = 1;
pub const INPUT_MOUSE_DOWN: u32 = 2;
pub const INPUT_MOUSE_UP: u32 = 3;
pub const INPUT_MOUSE_MOVE: u32 = 4;
pub const INPUT_SCROLL: u32 = 5;

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
        assert_eq!(SCHEMA_HASH, 0x6E65_7473_6865_6C51);
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
