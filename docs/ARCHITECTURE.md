# Architecture

GpuiNetShell is a two-layer shell runtime. C# is the application/scripting host;
Rust is the native host. The boundary is a versioned C ABI carrying a flat,
managed-owned element description.

## Ownership boundary

C# owns:

- application and domain state;
- `View` instances and their `Render` method;
- the element declaration API (`RenderContext` and the element builders);
- the render arena it fills;
- event handlers and callback tokens.

Rust owns:

- the `gpui::Application`, native window, and event loop;
- the decoded, owned snapshot;
- validation of every arena record;
- component materialization into GPUI elements;
- the component dispatch and behavior model;
- the C ABI table.

The boundary rule is: cross for a state transition (a dirty render, an event),
never per builder call, per frame, or per property.

## Render path

```text
ShellView::render                       Rust
        │ dirty?  ── no ──▶ materialize the retained snapshot
        ▼ yes
render callback (session, generation)   C ABI
        │ managed View.Render writes nodes / ops / children / UTF-8
        ▼
RenderArena.Publish()                   C#
        │ NativeArena descriptor
        ▼
Snapshot::decode -> RenderSnapshot      Rust
        │ render_completed(session, generation)
        ▼
materialize -> ComponentRegistry -> materializers -> elements
```

A clean GPUI repaint re-materializes the retained snapshot without calling
managed `Render`. `Render` runs only when the view is dirty, which is set on
first mount, by an event binding, or by `View.Invalidate()`.

## View and snapshots

`crates/gpui-net-shell/src/view.rs` mirrors `gpui-shell`'s `view.rs`. A
`ShellView` entity owns:

- `current` and `previous` `RenderSnapshot`s — the previous is held one
  generation longer so an event dispatched against the last frame still
  resolves its callback tokens;
- `dirty`, `retired`, and the last build `error`;
- the frozen component registry.

`rebuild` is transactional: the managed callback fills the arena, `Snapshot::decode`
validates it, and only then is the new `RenderSnapshot` swapped in. A failed
build leaves the previous description and its callbacks untouched.

A `RenderSnapshot` (in `src/snapshot.rs`) is a cloneable `Rc` handle that owns
its generation. When it is dropped, it calls the managed `retire_callbacks`
with its generation, and the managed host releases exactly that generation's
event handlers. That is what keeps callback lifetime tied to a description
rather than to a frame or a global counter.

## The render arena

Four flat buffers describe one render:

- `GpuiNetNode[]`: component id, flags, and a UTF-8 data range (Button id or
  Text content);
- `GpuiNetOp[]`: operations addressed by node index;
- `GpuiNetChild[]`: parent/child edges;
- one UTF-8 byte buffer.

Every operation's `a` word packs the UTF-8 range of a method name; `flags`
classifies the argument in `b` (`None`, `Number` as an `f32` bit pattern, or
`String` as a packed range). `Snapshot::decode` validates record flags, index
bounds, UTF-8 ranges, argument kinds, known components, and acyclicity before
producing owned data. Rust never retains a managed pointer: decoding copies
every value it keeps.

## Styling

Styling is not enumerated in the ABI. A node carries style *method calls* — a
GPUI method name plus an optional argument — and
`crates/gpui-net-shell/src/style.rs` folds them into one `StyleRefinement`,
exactly as `gpui-shell` does:

- **No-argument methods** (`items_center`, `size_full`, `rounded_md`, `text_sm`,
  …) come from GPUI's inspector reflection
  (`gpui_base::styled_ext_reflection_methods` and
  `gpui::styled_reflection::methods`). Adding an upstream style needs no change
  here. This is why the native crate enables `gpui-base/inspector`.
- **Methods that take an argument** (`p`, `gap`, `w`, `bg`, `text_color`, …) are
  bound by hand in `apply_param`, because reflection reaches no-argument methods
  only.

`materialize::resolve_ops` applies the calls in order while it accumulates the
node's behavior. The managed `StyleExtensions` surface (`Style`, `StyleColor`,
and typed sugar) only names methods; it has no knowledge of how they are
applied.

## Components and behavior

`crates/gpui-net-shell/src/materialize.rs` mirrors `gpui-shell`'s
`materialize.rs` and dispatches to a component registry:

- `resolve_ops` performs one pass over a node's ops: style calls fold into a
  `StyleRefinement`, `Method` ops split into shell behavior (`disabled`,
  `selected`) and recorded component methods, and `Callback` ops set the event
  tokens.
- `materialize_node` builds children first, then asks
  `FrozenComponentRegistry::descriptor(id)` for the component, runs its
  constructor with the node identity, records its methods, and hands a
  `MaterializeRequest` to the descriptor's `ComponentMaterializer`.
- The materializer builds the element and calls `request.finish(element)`, which
  applies the `StyleRefinement` and the ordinary children and returns an
  `AnyElement`.

The registry (`src/registry.rs`) is the seam adapted from `gpui-shell`'s
`component_registry.rs`: a `ComponentDescriptor` owns its constructors, methods,
and materializer. Adding a component is:

1. a registry index constant in `schema.rs` and its `NativeProtocol` mirror;
2. a `ComponentMaterializer` and a `register` call in `components/`.

The runtime never names a concrete component: the decoder knows only an id, and
`materialize.rs` knows only a descriptor. Component methods are not enumerated
per method either: `ComponentDescriptor::method(name)` resolves a recorded name
to its recorder. `resolve_ops` and the descriptor recorders are pure and tested
without a window.

## Application and events

`GpuiApplication` owns one `View` and one native session. `View.Render` builds an
element tree; `ButtonElement.OnClick` registers a handler in the
`EventRegistry` and writes a callback token into the arena.

When the native button activates, the materialized closure calls the managed
`click` callback with the token. The handler runs on the GPUI thread; after it
returns, the host marks the view dirty and notifies it. No queue is required
because callbacks are delivered synchronously on the application thread.

Managed event handlers run inside a C callback and never let an exception cross
the boundary: each managed callback catches and returns an error status.

## C ABI

`gpui_net_shell_get_api(requested_version)` returns a static `GpuiNetShellApi`
table:

```c
struct GpuiNetShellApi {
    uint32_t struct_size;
    uint32_t abi_version;
    uint64_t schema_hash;
    int32_t (*run_application)(uint64_t, const GpuiNetCallbacks*);
    int32_t (*invalidate)(uint64_t session);
    uint64_t reserved;
};
```

`run_application` blocks in the GPUI event loop. `invalidate` is the
any-thread request behind `View.Invalidate()`: it posts to the session's view,
which marks itself dirty and repaints.

`GpuiNetCallbacks` carries the managed `application_started`, `window_closed`,
`render(session, generation, arena, root)`, `render_completed(session,
generation, status)`, `click(session, token)`, and `retire_callbacks(session,
generation)` function pointers, all cdecl.

`ABI_VERSION` covers the record layouts; `SCHEMA_HASH` covers the component and
operation vocabulary. A managed/native pair must agree on both. The constants
live in `crates/gpui-net-shell/src/schema.rs` and
`src/GpuiNetShell/Interop/NativeProtocol.cs`; a test pins each literal.

Every exported Rust function validates pointers before dereferencing and wraps
its body in `catch_unwind`, so a panic becomes a status code rather than an
unwind across the C boundary.

## Extending the surface

- A new **style** needs no native change at all: add sugar in
  `StyleExtensions` for a name the reflected table already knows, or add a
  name to `PARAM_STYLES` and one arm in `apply_param` for a method that takes
  an argument.
- A new **component method** is a `MethodDescriptor` on that component's
  descriptor plus one arm in its materializer; the managed side only names the
  method.
- A new **component** is a descriptor and a materializer registered in
  `components/`, as above.

When a component id or payload rule changes, bump `SCHEMA_HASH` on both sides.
When a C record layout changes, bump `ABI_VERSION` and update both sides
together.

## Windows apartment

GPUI's Windows platform initializes OLE as a single-threaded apartment, but a
managed entry thread is already MTA (`RPC_E_CHANGED_MODE`). `GpuiApplication.Run`
runs the native event loop on a dedicated STA thread on Windows, mirroring the
manifest requirement for Windows common controls.

## Not yet built

- retained controls beyond Button (Input, Slider, Scroll, List/Table);
- multi-window support and window options over the ABI;
- a theme payload from C# (the native host uses the gpui-component default);
- hot reload of managed code;
- virtualized item batches.
