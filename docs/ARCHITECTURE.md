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
- the component registry;
- the C ABI table.

The boundary rule is: cross for a state transition (a dirty render, an event),
never per builder call, per frame, or per property.

## Render path

```text
View.Render(ref RenderContext)         C#
        │ writes nodes / ops / children / UTF-8
        ▼
RenderArena.Publish()                  C#  (copies to unmanaged buffers)
        │ NativeArena descriptor
        ▼
render callback (C ABI)
        │ Snapshot::decode  -> owned ValidatedSnapshot
        ▼
render_completed(session, revision)    Rust -> C# acknowledgement
        │
        ▼
ShellView::render -> materialize_node  Rust
        │ component registry dispatch
        ▼
Div / Text / Button materializers -> gpui-component elements
```

A clean GPUI repaint re-materializes the retained snapshot without calling
managed `Render`. `Render` runs only when the view is dirty, which is set on
first mount and whenever an event binding requests a re-render.

## The render arena

Four flat buffers describe one render:

- `GpuiNetNode[]`: component id, flags, and a UTF-8 data range (Button id or
  Text content);
- `GpuiNetOp[]`: typed operations addressed by node index;
- `GpuiNetChild[]`: parent/child edges;
- one UTF-8 byte buffer.

String operations pack `(offset << 32) | length` into the operation's `a` word.
All other operations use `a` (and sometimes `b`) with their own meaning. A style
operation stores the GPUI method name in `a` and its argument in `b`.

`Snapshot::decode` validates record flags, index bounds, UTF-8 ranges, known
components, known operations, and acyclicity before producing owned data.
Rust never retains a managed pointer: decoding copies every value it keeps.

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

`components::build_refinement` applies the calls in order; `components::apply_style`
refines any `Styled` element with the result. The managed `StyleExtensions`
surface (`lib`-side `Style`, `StyleColor`, and typed sugar) only names methods;
it has no knowledge of how they are applied.

## Components and the registry

`crates/gpui-net-shell/src/registry.rs` maps a component id to a
`ComponentDescriptor { id, name, materializer }`. This is a smaller form of
`gpui-shell`'s descriptor seam. Adding a component is:

1. a `COMPONENT_*` constant and its `is_known_component` arm in `schema.rs`;
2. a `NativeProtocol.Component*` constant in C#;
3. a materializer in `components/`;
4. one entry in `ComponentRegistry::with_builtins`.

The host, decoder, FFI, and managed runtime never name a concrete component
beyond that registry.

Each materializer splits a pure *plan* from the element build so the
interpretation is testable without a window (see `button::ButtonPlan`).

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
    uint64_t reserved;
};
```

`run_application` blocks in the GPUI event loop. `GpuiNetCallbacks` carries the
managed `application_started`, `window_closed`, `render`, `render_completed`,
and `click` function pointers, all cdecl.

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
- A new **component behavior operation** is a constant in `schema.rs`, its
  `NativeProtocol` mirror, a decode arm, and the component's plan.
- A new **component** is a descriptor plus a materializer, as above.

When a component id, behavior operation code, or payload rule changes, bump
`SCHEMA_HASH` on both sides. When a C record layout changes, bump `ABI_VERSION`
and update both sides together.

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
