//! Decoding a borrowed managed arena into an owned snapshot.
//!
//! Rust must never retain a managed pointer. [`Snapshot::decode`] validates
//! every record and copies what it keeps, so a later managed callback or
//! teardown cannot observe a stale borrow.

use crate::abi::GpuiNetArena;
use crate::schema::*;

/// A decoded operation.
///
/// Component behavior is typed because the host must interpret it. Styling is
/// *not* enumerated: a style operation carries the GPUI method name and its
/// argument, resolved against the reflected style table at materialization
/// time (see [`crate::style`]).
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Disabled(bool),
    Selected(bool),
    OnClick(u64),
    Label(String),
    Tooltip(String),
    Loading(bool),
    ButtonVariant(u64),
    ButtonSize(u64),
    Compact,
    /// A no-argument style method, by name.
    StyleNullary(String),
    /// A style method taking a pixel/number argument.
    StyleLength(String, f32),
    /// A style method taking a bare number.
    StyleNumber(String, f32),
    /// A style method taking a color literal.
    StyleColor(String, String),
    /// A style method taking a string argument.
    StyleString(String, String),
}

/// A decoded element.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub component: u32,
    pub data: String,
    pub ops: Vec<Op>,
    pub children: Vec<u32>,
}

impl Node {
    /// The last declared activation token, if any.
    pub fn on_click(&self) -> Option<u64> {
        self.ops.iter().rev().find_map(|op| match op {
            Op::OnClick(token) => Some(*token),
            _ => None,
        })
    }
}

/// An owned element description for one window render.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub root: u32,
    pub nodes: Vec<Node>,
}

impl Snapshot {
    /// Validates and copies `arena`, returning the owned description.
    pub fn decode(arena: &GpuiNetArena, root: u32) -> Result<Self, i32> {
        // SAFETY: each slice borrows caller-owned memory for the duration of
        // this call only; bounds are validated before indexing.
        let node_records = unsafe { arena_slice(arena.nodes, arena.nodes_len) }?;
        let op_records = unsafe { arena_slice(arena.ops, arena.ops_len) }?;
        let child_records = unsafe { arena_slice(arena.children, arena.children_len) }?;
        let utf8 = unsafe { byte_slice(arena.utf8, arena.utf8_len) }?;

        let mut nodes = Vec::with_capacity(node_records.len());
        for record in node_records {
            if record.flags != 0 {
                return Err(STATUS_INVALID_ARGUMENT);
            }
            let data = read_string(utf8, record.data_offset, record.data_len)?;
            nodes.push(Node {
                component: record.component,
                data,
                ops: Vec::new(),
                children: Vec::new(),
            });
        }

        for record in op_records {
            let index = record.node as usize;
            if index >= nodes.len() {
                return Err(STATUS_BAD_INDEX);
            }
            let op = decode_op(record, utf8)?;
            nodes[index].ops.push(op);
        }

        for record in child_records {
            let child = record.child as usize;
            if child >= nodes.len() {
                return Err(STATUS_BAD_INDEX);
            }
            if record.parent == record.child {
                return Err(STATUS_CYCLE);
            }
            let parent = record.parent as usize;
            if parent >= nodes.len() {
                return Err(STATUS_BAD_INDEX);
            }
            nodes[parent].children.push(record.child);
        }

        if (root as usize) >= nodes.len() {
            return Err(STATUS_BAD_INDEX);
        }
        for node in &nodes {
            if !is_known_component(node.component) {
                return Err(STATUS_UNKNOWN_COMPONENT);
            }
        }
        detect_cycle(&nodes)?;

        Ok(Self { root, nodes })
    }
}

pub fn is_known_component(component: u32) -> bool {
    matches!(component, COMPONENT_DIV | COMPONENT_TEXT | COMPONENT_BUTTON)
}

fn decode_op(record: &crate::abi::GpuiNetOp, utf8: &[u8]) -> Result<Op, i32> {
    if record.flags != 0 {
        return Err(STATUS_INVALID_ARGUMENT);
    }
    let operand = record.a;
    let data = record.b;
    match record.code {
        OP_DISABLED => {
            require_b_zero(data)?;
            Ok(Op::Disabled(operand != 0))
        }
        OP_SELECTED => {
            require_b_zero(data)?;
            Ok(Op::Selected(operand != 0))
        }
        OP_LOADING => {
            require_b_zero(data)?;
            Ok(Op::Loading(operand != 0))
        }
        OP_ON_CLICK => {
            require_b_zero(data)?;
            if operand == 0 {
                return Err(STATUS_INVALID_ARGUMENT);
            }
            Ok(Op::OnClick(operand))
        }
        OP_BUTTON_VARIANT => {
            require_b_zero(data)?;
            Ok(Op::ButtonVariant(operand))
        }
        OP_BUTTON_SIZE => {
            require_b_zero(data)?;
            Ok(Op::ButtonSize(operand))
        }
        OP_COMPACT => {
            require_b_zero(data)?;
            if operand != 0 {
                return Err(STATUS_INVALID_ARGUMENT);
            }
            Ok(Op::Compact)
        }
        OP_LABEL => {
            require_b_zero(data)?;
            Ok(Op::Label(read_word(utf8, operand)?))
        }
        OP_TOOLTIP => {
            require_b_zero(data)?;
            Ok(Op::Tooltip(read_word(utf8, operand)?))
        }
        OP_STYLE_NULLARY => {
            require_b_zero(data)?;
            Ok(Op::StyleNullary(read_word(utf8, operand)?))
        }
        OP_STYLE_LENGTH => Ok(Op::StyleLength(read_word(utf8, operand)?, scalar(data)?)),
        OP_STYLE_NUMBER => Ok(Op::StyleNumber(read_word(utf8, operand)?, scalar(data)?)),
        OP_STYLE_COLOR => Ok(Op::StyleColor(
            read_word(utf8, operand)?,
            read_word(utf8, data)?,
        )),
        OP_STYLE_STRING => Ok(Op::StyleString(
            read_word(utf8, operand)?,
            read_word(utf8, data)?,
        )),
        _ => Err(STATUS_INVALID_ARGUMENT),
    }
}

fn require_b_zero(data: u64) -> Result<(), i32> {
    if data == 0 {
        Ok(())
    } else {
        Err(STATUS_INVALID_ARGUMENT)
    }
}

fn scalar(bits: u64) -> Result<f32, i32> {
    let value = f32::from_bits(bits as u32);
    if value.is_finite() {
        Ok(value)
    } else {
        Err(STATUS_INVALID_ARGUMENT)
    }
}

fn read_word(utf8: &[u8], word: u64) -> Result<String, i32> {
    read_string(utf8, (word >> 32) as u32, (word & 0xffff_ffff) as u32)
}

unsafe fn arena_slice<'a, T>(pointer: *const T, length: u32) -> Result<&'a [T], i32> {
    if length == 0 {
        return Ok(&[]);
    }
    if pointer.is_null() {
        return Err(STATUS_NULL_POINTER);
    }
    Ok(unsafe { std::slice::from_raw_parts(pointer, length as usize) })
}

unsafe fn byte_slice<'a>(pointer: *const u8, length: u32) -> Result<&'a [u8], i32> {
    if length == 0 {
        return Ok(&[]);
    }
    if pointer.is_null() {
        return Err(STATUS_NULL_POINTER);
    }
    Ok(unsafe { std::slice::from_raw_parts(pointer, length as usize) })
}

fn read_string(utf8: &[u8], offset: u32, length: u32) -> Result<String, i32> {
    let start = offset as usize;
    let end = start.checked_add(length as usize).ok_or(STATUS_TRUNCATED)?;
    let bytes = utf8.get(start..end).ok_or(STATUS_TRUNCATED)?;
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| STATUS_BAD_UTF8)
}

fn detect_cycle(nodes: &[Node]) -> Result<(), i32> {
    const UNVISITED: u8 = 0;
    const OPEN: u8 = 1;
    const CLOSED: u8 = 2;

    let mut color = vec![UNVISITED; nodes.len()];
    for start in 0..nodes.len() {
        if color[start] != UNVISITED {
            continue;
        }
        color[start] = OPEN;
        let mut stack = vec![(start, 0usize)];
        while let Some((node, next)) = stack.pop() {
            let children = &nodes[node].children;
            if next < children.len() {
                stack.push((node, next + 1));
                let child = children[next] as usize;
                match color[child] {
                    OPEN => return Err(STATUS_CYCLE),
                    UNVISITED => {
                        color[child] = OPEN;
                        stack.push((child, 0));
                    }
                    _ => {}
                }
            } else {
                color[node] = CLOSED;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::{GpuiNetChild, GpuiNetNode, GpuiNetOp};

    fn string_word(value: &str, host: &mut String) -> u64 {
        let offset = host.len() as u32;
        host.push_str(value);
        ((offset as u64) << 32) | value.len() as u64
    }

    struct RawArena {
        nodes: Vec<GpuiNetNode>,
        ops: Vec<GpuiNetOp>,
        children: Vec<GpuiNetChild>,
        utf8: Vec<u8>,
    }

    impl RawArena {
        fn descriptor(&self) -> GpuiNetArena {
            GpuiNetArena {
                nodes: self.nodes.as_ptr(),
                nodes_len: self.nodes.len() as u32,
                ops: self.ops.as_ptr(),
                ops_len: self.ops.len() as u32,
                children: self.children.as_ptr(),
                children_len: self.children.len() as u32,
                utf8: self.utf8.as_ptr(),
                utf8_len: self.utf8.len() as u32,
                ..GpuiNetArena::empty()
            }
        }
    }

    #[test]
    fn decodes_a_button_tree_with_identity_ops_and_children() {
        let mut utf8 = String::new();
        let id_word = string_word("save", &mut utf8);
        let label_word = string_word("Save", &mut utf8);
        let bytes = utf8.into_bytes();

        let arena = RawArena {
            nodes: vec![
                GpuiNetNode {
                    component: COMPONENT_DIV,
                    flags: 0,
                    data_offset: 0,
                    data_len: 0,
                },
                GpuiNetNode {
                    component: COMPONENT_BUTTON,
                    flags: 0,
                    data_offset: (id_word >> 32) as u32,
                    data_len: (id_word & 0xffff_ffff) as u32,
                },
            ],
            ops: vec![
                GpuiNetOp {
                    node: 1,
                    code: OP_LABEL,
                    flags: 0,
                    a: label_word,
                    b: 0,
                },
                GpuiNetOp {
                    node: 1,
                    code: OP_BUTTON_VARIANT,
                    flags: 0,
                    a: BUTTON_VARIANT_PRIMARY,
                    b: 0,
                },
                GpuiNetOp {
                    node: 1,
                    code: OP_ON_CLICK,
                    flags: 0,
                    a: 7,
                    b: 0,
                },
            ],
            children: vec![GpuiNetChild {
                parent: 0,
                child: 1,
            }],
            utf8: bytes,
        };

        let snapshot = Snapshot::decode(&arena.descriptor(), 0).expect("valid arena");
        assert_eq!(snapshot.nodes.len(), 2);
        assert_eq!(snapshot.nodes[0].children, vec![1]);
        let button = &snapshot.nodes[1];
        assert_eq!(button.component, COMPONENT_BUTTON);
        assert_eq!(button.data, "save");
        assert!(button.ops.contains(&Op::Label("Save".into())));
        assert!(button
            .ops
            .contains(&Op::ButtonVariant(BUTTON_VARIANT_PRIMARY)));
        assert_eq!(button.on_click(), Some(7));
    }

    #[test]
    fn decodes_generic_style_operations() {
        let mut utf8 = String::new();
        let padding = string_word("p", &mut utf8);
        let color_name = string_word("bg", &mut utf8);
        let color = string_word("#ff0000", &mut utf8);
        let nullary = string_word("items_center", &mut utf8);
        let bytes = utf8.into_bytes();

        let arena = RawArena {
            nodes: vec![GpuiNetNode {
                component: COMPONENT_DIV,
                ..Default::default()
            }],
            ops: vec![
                GpuiNetOp {
                    node: 0,
                    code: OP_STYLE_LENGTH,
                    flags: 0,
                    a: padding,
                    b: 12.0f32.to_bits() as u64,
                },
                GpuiNetOp {
                    node: 0,
                    code: OP_STYLE_COLOR,
                    flags: 0,
                    a: color_name,
                    b: color,
                },
                GpuiNetOp {
                    node: 0,
                    code: OP_STYLE_NULLARY,
                    flags: 0,
                    a: nullary,
                    b: 0,
                },
            ],
            children: Vec::new(),
            utf8: bytes,
        };

        let snapshot = Snapshot::decode(&arena.descriptor(), 0).expect("valid arena");
        assert_eq!(
            snapshot.nodes[0].ops,
            vec![
                Op::StyleLength("p".into(), 12.0),
                Op::StyleColor("bg".into(), "#ff0000".into()),
                Op::StyleNullary("items_center".into()),
            ]
        );
    }

    #[test]
    fn rejects_out_of_range_child_and_unknown_component() {
        let mut arena = RawArena {
            nodes: vec![GpuiNetNode {
                component: COMPONENT_DIV,
                ..Default::default()
            }],
            ops: Vec::new(),
            children: vec![GpuiNetChild {
                parent: 0,
                child: 4,
            }],
            utf8: Vec::new(),
        };
        assert_eq!(
            Snapshot::decode(&arena.descriptor(), 0),
            Err(STATUS_BAD_INDEX)
        );

        arena.children.clear();
        arena.nodes[0].component = 99;
        assert_eq!(
            Snapshot::decode(&arena.descriptor(), 0),
            Err(STATUS_UNKNOWN_COMPONENT)
        );
    }

    #[test]
    fn rejects_cycles() {
        let arena = RawArena {
            nodes: vec![
                GpuiNetNode {
                    component: COMPONENT_DIV,
                    ..Default::default()
                },
                GpuiNetNode {
                    component: COMPONENT_DIV,
                    ..Default::default()
                },
            ],
            ops: Vec::new(),
            children: vec![
                GpuiNetChild {
                    parent: 0,
                    child: 1,
                },
                GpuiNetChild {
                    parent: 1,
                    child: 0,
                },
            ],
            utf8: Vec::new(),
        };
        assert_eq!(Snapshot::decode(&arena.descriptor(), 0), Err(STATUS_CYCLE));
    }

    #[test]
    fn rejects_truncated_string_ranges() {
        let arena = RawArena {
            nodes: vec![GpuiNetNode {
                component: COMPONENT_TEXT,
                flags: 0,
                data_offset: 2,
                data_len: 8,
            }],
            ops: Vec::new(),
            children: Vec::new(),
            utf8: vec![b'a', b'b'],
        };
        assert_eq!(
            Snapshot::decode(&arena.descriptor(), 0),
            Err(STATUS_TRUNCATED)
        );
    }

    #[test]
    fn rejects_an_unknown_operation_code() {
        let arena = RawArena {
            nodes: vec![GpuiNetNode {
                component: COMPONENT_DIV,
                ..Default::default()
            }],
            ops: vec![GpuiNetOp {
                node: 0,
                code: 9_999,
                flags: 0,
                a: 0,
                b: 0,
            }],
            children: Vec::new(),
            utf8: Vec::new(),
        };
        assert_eq!(
            Snapshot::decode(&arena.descriptor(), 0),
            Err(STATUS_INVALID_ARGUMENT)
        );
    }

    #[test]
    fn zero_length_arena_is_an_empty_error_not_a_crash() {
        let arena = GpuiNetArena::empty();
        assert_eq!(Snapshot::decode(&arena, 0), Err(STATUS_BAD_INDEX));
    }
}
