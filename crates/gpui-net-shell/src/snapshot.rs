//! Decoding a borrowed managed arena into an owned snapshot.
//!
//! Rust must never retain a managed pointer. [`Snapshot::decode`] validates
//! every record and copies what it keeps, so a later managed callback or
//! teardown cannot observe a stale borrow.
//!
//! The decoded [`Op`] mirrors `gpui-shell`'s `SpecOp`: a style call, a component
//! behavior `Method`, or a `Callback`. Which style name, which behavior method,
//! and what it means are resolved during materialization, not here.

use std::rc::Rc;

use crate::abi::{GpuiNetArena, GpuiNetCallbacks};
use crate::schema::*;
use crate::style::StyleArg;

/// A decoded operation.
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    /// A no-argument style method, by name.
    NullaryStyle(String),
    /// A style method taking one argument.
    ParamStyle(String, StyleArg),
    /// A component behavior method with an optional argument.
    Method(String, Option<StyleArg>),
    /// An event binding by name and callback token.
    Callback(String, u64),
    /// A named slot pointing at one child node.
    Slot(String, u32),
}

/// A decoded element.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub component: u32,
    pub data: String,
    pub ops: Vec<Op>,
    pub children: Vec<u32>,
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
        detect_cycle(&nodes)?;

        Ok(Self { root, nodes })
    }
}

/// One frozen description of a view's interface, modeled on `gpui-shell`'s
/// `RenderSnapshot`.
///
/// Built by [`Snapshot::decode`] and read by the materializer. A replacement is
/// built beside the live one and swapped in whole, so a decode that fails leaves
/// the previous description untouched. The snapshot owns its generation: when it
/// is dropped it tells the managed host to retire the event handlers that
/// generation registered, which is what keeps callback lifetime tied to the
/// description rather than to a frame.
#[derive(Clone)]
pub struct RenderSnapshot {
    inner: Rc<RenderSnapshotInner>,
}

struct RenderSnapshotInner {
    session_id: u64,
    revision: u64,
    snapshot: Snapshot,
    callbacks: GpuiNetCallbacks,
}

impl RenderSnapshot {
    pub(crate) fn new(
        session_id: u64,
        revision: u64,
        snapshot: Snapshot,
        callbacks: GpuiNetCallbacks,
    ) -> Self {
        Self {
            inner: Rc::new(RenderSnapshotInner {
                session_id,
                revision,
                snapshot,
                callbacks,
            }),
        }
    }

    /// The generation this description was built for.
    pub fn revision(&self) -> u64 {
        self.inner.revision
    }

    pub fn root(&self) -> u32 {
        self.inner.snapshot.root
    }

    pub fn nodes(&self) -> &[Node] {
        &self.inner.snapshot.nodes
    }

    pub fn node(&self, id: u32) -> Option<&Node> {
        self.inner.snapshot.nodes.get(id as usize)
    }

    pub fn len(&self) -> usize {
        self.inner.snapshot.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.snapshot.nodes.is_empty()
    }
}

impl Drop for RenderSnapshotInner {
    fn drop(&mut self) {
        if let Some(retire) = self.callbacks.retire_callbacks {
            // SAFETY: managed callback; retiring one generation touches no
            // native borrow.
            unsafe {
                let _ = retire(self.session_id, self.revision);
            }
        }
    }
}

fn decode_op(record: &crate::abi::GpuiNetOp, utf8: &[u8]) -> Result<Op, i32> {
    let name = read_word(utf8, record.a)?;
    if name.is_empty() {
        return Err(STATUS_INVALID_ARGUMENT);
    }

    match record.code {
        OP_NULLARY_STYLE => {
            if record.flags != ARG_NONE || record.b != 0 {
                return Err(STATUS_INVALID_ARGUMENT);
            }
            Ok(Op::NullaryStyle(name))
        }
        OP_PARAM_STYLE => Ok(Op::ParamStyle(name, read_arg(record, utf8)?)),
        OP_METHOD => match record.flags {
            ARG_NONE => {
                if record.b != 0 {
                    return Err(STATUS_INVALID_ARGUMENT);
                }
                Ok(Op::Method(name, None))
            }
            ARG_NUMBER => Ok(Op::Method(name, Some(StyleArg::Number(number(record.b)?)))),
            ARG_STRING => Ok(Op::Method(
                name,
                Some(StyleArg::String(read_word(utf8, record.b)?)),
            )),
            _ => Err(STATUS_INVALID_ARGUMENT),
        },
        OP_CALLBACK => {
            if record.flags != ARG_NONE || record.b == 0 {
                return Err(STATUS_INVALID_ARGUMENT);
            }
            Ok(Op::Callback(name, record.b))
        }
        OP_SLOT => {
            if record.flags != ARG_NUMBER {
                return Err(STATUS_INVALID_ARGUMENT);
            }
            Ok(Op::Slot(name, record.b as u32))
        }
        _ => Err(STATUS_INVALID_ARGUMENT),
    }
}

fn read_arg(record: &crate::abi::GpuiNetOp, utf8: &[u8]) -> Result<StyleArg, i32> {
    match record.flags {
        ARG_NUMBER => Ok(StyleArg::Number(number(record.b)?)),
        ARG_STRING => Ok(StyleArg::String(read_word(utf8, record.b)?)),
        _ => Err(STATUS_INVALID_ARGUMENT),
    }
}

fn number(bits: u64) -> Result<f32, i32> {
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
    fn decodes_styles_methods_and_callbacks() {
        let mut utf8 = String::new();
        let id = string_word("save", &mut utf8);
        let label_name = string_word("label", &mut utf8);
        let label_value = string_word("Save", &mut utf8);
        let nullary = string_word("items_center", &mut utf8);
        let click = string_word("on_click", &mut utf8);
        let bytes = utf8.into_bytes();

        let arena = RawArena {
            nodes: vec![GpuiNetNode {
                component: COMPONENT_BUTTON,
                flags: 0,
                data_offset: (id >> 32) as u32,
                data_len: (id & 0xffff_ffff) as u32,
            }],
            ops: vec![
                GpuiNetOp {
                    node: 0,
                    code: OP_METHOD,
                    flags: ARG_STRING,
                    a: label_name,
                    b: label_value,
                },
                GpuiNetOp {
                    node: 0,
                    code: OP_NULLARY_STYLE,
                    flags: ARG_NONE,
                    a: nullary,
                    b: 0,
                },
                GpuiNetOp {
                    node: 0,
                    code: OP_CALLBACK,
                    flags: ARG_NONE,
                    a: click,
                    b: 7,
                },
            ],
            children: Vec::new(),
            utf8: bytes,
        };

        let snapshot = Snapshot::decode(&arena.descriptor(), 0).expect("valid arena");
        assert_eq!(snapshot.nodes[0].data, "save");
        assert_eq!(
            snapshot.nodes[0].ops,
            vec![
                Op::Method("label".into(), Some(StyleArg::String("Save".into()))),
                Op::NullaryStyle("items_center".into()),
                Op::Callback("on_click".into(), 7),
            ]
        );
    }

    #[test]
    fn decodes_numeric_methods_and_param_styles() {
        let mut utf8 = String::new();
        let size = string_word("size", &mut utf8);
        let padding = string_word("p", &mut utf8);
        let bytes = utf8.into_bytes();

        let arena = RawArena {
            nodes: vec![GpuiNetNode {
                component: COMPONENT_BUTTON,
                ..Default::default()
            }],
            ops: vec![
                GpuiNetOp {
                    node: 0,
                    code: OP_METHOD,
                    flags: ARG_NUMBER,
                    a: size,
                    b: 2.0f32.to_bits() as u64,
                },
                GpuiNetOp {
                    node: 0,
                    code: OP_PARAM_STYLE,
                    flags: ARG_NUMBER,
                    a: padding,
                    b: 12.0f32.to_bits() as u64,
                },
            ],
            children: Vec::new(),
            utf8: bytes,
        };

        let snapshot = Snapshot::decode(&arena.descriptor(), 0).expect("valid arena");
        assert_eq!(
            snapshot.nodes[0].ops,
            vec![
                Op::Method("size".into(), Some(StyleArg::Number(2.0))),
                Op::ParamStyle("p".into(), StyleArg::Number(12.0)),
            ]
        );
    }

    #[test]
    fn rejects_out_of_range_child() {
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
        assert!(Snapshot::decode(&arena.descriptor(), 0).is_ok());
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

    #[test]
    fn dropping_a_snapshot_retires_its_generation() {
        use std::sync::atomic::{AtomicU64, Ordering};

        static RETIRED: AtomicU64 = AtomicU64::new(0);
        unsafe extern "C" fn retire(_session_id: u64, generation: u64) -> i32 {
            RETIRED.store(generation, Ordering::SeqCst);
            0
        }

        let callbacks = GpuiNetCallbacks {
            struct_size: std::mem::size_of::<GpuiNetCallbacks>() as u32,
            _reserved: 0,
            application_started: None,
            window_closed: None,
            render: None,
            render_completed: None,
            click: None,
            retire_callbacks: Some(retire),
        };
        let snapshot = Snapshot {
            root: 0,
            nodes: Vec::new(),
        };

        let frozen = RenderSnapshot::new(11, 7, snapshot, callbacks);
        assert_eq!(frozen.revision(), 7);
        drop(frozen);

        assert_eq!(RETIRED.load(Ordering::SeqCst), 7);
    }
}
