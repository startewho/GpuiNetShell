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
        │ dirty?  ── no ──▶ materialize from the prepared snapshot
        ▼ yes
render callback (session, generation)   C ABI
        │ managed View.Render writes nodes / ops / children / UTF-8
        ▼
RenderArena.Publish()                   C#
        │ NativeArena descriptor
        ▼
Snapshot::decode -> prepare -> RenderSnapshot   Rust
        │ render_completed(session, generation)
        ▼
materialize -> ComponentRegistry -> materializers -> elements
```

Materialization lives in a retained `ContentHost` entity, not in
`ShellView::render`. GPUI re-runs a view's `render` only when it is notified, so
the content subtree is reused unless a new description is pushed. `prepare`
folds each node's ops into a `PreparedNode` (style, payload, recorded methods,
child routing) once, when the description is built, and `materialize` only
borrows that. `Render` runs only when the view is dirty, which is set on first
mount, by an event binding, or by `View.Invalidate()` / `RenderContext.Notify()`.

## View and snapshots

`crates/gpui-net-shell/src/view.rs` mirrors `gpui-shell`'s `view.rs`. A
`ShellView` entity owns:

- `current` — the displayed `RenderSnapshot`;
- `previous` — the description `current` replaced, held one generation longer.
  The retained `ContentHost` re-materializes a frame after a swap, so the
  on-screen tree can still reference the replaced generation's callback tokens
  for one frame; `previous` keeps those tokens valid. Without it a click on the
  not-yet-rebuilt tree resolves against a retired generation and is dropped;
- `displayed_fingerprint` — the structural fingerprint of `current`. When a
  rebuild produces the same fingerprint, the displayed description (and its
  generation, and therefore its callbacks) is kept and the new generation is
  retired instead, so a redundant invalidation does no native materialization;
- a retained `content: Entity<ContentHost>` that owns the element tree;
- `dirty`, `retired`, and the last build `error`;
- the frozen component registry.

The fingerprint in `snapshot.rs` hashes components, data, style and method
codes, arguments, slots, and child edges, but **not** callback tokens: tokens
are new every generation by design and do not change the interface.

`rebuild` is transactional: the managed callback fills the arena, `Snapshot::decode`
validates it, `prepare` resolves it (including the fingerprint), and only then
is the new `RenderSnapshot` swapped in. A failed build leaves the previous
description and its callbacks untouched.

A `RenderSnapshot` (in `src/snapshot.rs`) is a cloneable `Rc` handle that owns
its generation. When it is dropped, it calls the managed `retire_callbacks`
with its generation, and the managed host releases exactly that generation's
event handlers. Because the retained `ContentHost` holds the displayed snapshot,
its generation — and its callbacks — stay alive for as long as it is shown.

Managed code requests a rebuild through `View.Invalidate()` or
`RenderContext.Notify()`, both of which post the native `invalidate` command.
`Notify()` throws when called during `Render`, mirroring gpui's rule that a
render may not request another render of itself; state changes belong in events
or tasks. The managed `RenderContext` is one stable instance per session, so a
handler can hold it and notify later.

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

Styling is a **closed opcode vocabulary**, not runtime reflection. A node
carries style *calls* — a `u16` opcode plus an optional argument — and
`crates/gpui-net-shell/src/style.rs` folds them into one `StyleRefinement`:

- **No-argument methods** (`items_center`, `size_full`, `flex_col`, `font_bold`,
  …) are declared in the `style_vocabulary!` macro as `(name, |style|
  style.method())` entries. The index is the wire opcode and each entry is a
  direct call.
- **Methods that take an argument** (`p`, `gap`, `w`, `bg`, `text_color`, …) are
  listed by the same macro and bound by hand in `apply_param`, because their
  argument type differs per method.

The order of both lists is the ABI vocabulary, mirrored by name in
`src/GpuiNetShell/Interop/StyleOps.cs`. The managed `RenderArena` maps a style
name to its opcode before publishing, so the arena carries no style strings;
`style.rs` has a `the_managed_vocabulary_matches` test that parses the managed
file and fails if either list drifts. Because nothing reflects over GPUI, the
native crate does **not** enable `gpui-base/inspector`, and the linker can drop
the style methods the vocabulary does not name.

`materialize::resolve_ops` applies the calls in order while it accumulates the
node's behavior. The managed `StyleExtensions` surface (`Style`, `StyleColor`,
and typed sugar) only names methods; it has no knowledge of how they are
applied. A name outside the vocabulary is dropped, exactly as a typo was before.

## Components and behavior

`crates/gpui-net-shell/src/materialize.rs` mirrors `gpui-shell`'s
`materialize.rs` and dispatches to a component registry. The work is split in
two so a repaint is cheap:

- `prepare` runs once per built description. For each node it performs one pass
  over the ops: style calls fold into a `StyleRefinement`, `Method` ops split
  into shell behavior (`disabled`, `selected`) and recorded component methods,
  `Callback` ops set the event tokens, and named slots are routed out of the
  ordinary children. The result is a `PreparedNode` — style, constructor
  payload, recorded methods, slots, child ids, behavior — stored on the
  snapshot.
- `materialize_node` runs on every repaint. It borrows the `PreparedNode`,
  materializes the children, and hands a `MaterializeRequest` to the
  descriptor's `ComponentMaterializer`. It does not re-resolve ops, rebuild
  payloads, or re-record methods.
- The materializer builds the element and calls `request.finish(element)`, which
  applies the `StyleRefinement` and the ordinary children and returns an
  `AnyElement`.

A component method's name does not cross the ABI either: the managed side sends
a `MethodOps` code (FNV-1a over the name), and the frozen registry resolves it
back to the name for `ComponentDescriptor::method`. `disabled` and `selected`
are reserved names in that table.

The registry (`src/registry.rs`) is the seam adapted from `gpui-shell`'s
`component_registry.rs`: a `ComponentDescriptor` owns its constructors, methods,
and materializer. Adding a component is:

1. a registry index constant in `schema.rs` and its `NativeProtocol` mirror;
2. a `ComponentMaterializer` and a `register` call in `components/`.

The runtime never names a concrete component: the decoder knows only an id, and
`materialize.rs` knows only a descriptor. Component methods are not enumerated
per method either: `ComponentDescriptor::method(name)` resolves a recorded name
to its recorder. `prepare`, `resolve_ops`, and the descriptor recorders are pure
and tested without a window.

The built-in catalog is `Div`, `Text`, `Button`, `Label`, `Badge`, `Progress`,
`Combobox`, `Radio`, `Tabs`, `Scroll`, `Scrollbar`, `Resizable`, and `Popover`;
each is one file in `src/components/`. A component with named parts receives
them as **slots**: `materialize_node` reads a node's `Slot` ops and delivers the
referenced children by name through `MaterializeRequest::take_slot` instead of as
ordinary children. Materializers also get the current `Window` and `App` through
`with_window_app`, which is what lets `Scroll` and `Scrollbar` share a
`ScrollHandle` in window element state by name.

## Root and overlays

`crates/gpui-net-shell/src/root.rs` mirrors `gpui-shell`'s `root.rs`. `Root`
owns the managed content view and paints the layers over it. Each layer is
deferred at its own priority, so the paint order is independent of construction
order:

1. **Content** — the `ShellView` the managed host describes.
2. **Sheet** (`8`) — at most one, anchored to a viewport edge. Replacing, not
   stacking, keeps the focus record honest: the incoming sheet inherits the
   outgoing one's restore target. The combobox popup is a sheet too: it opens a
   `Bottom` sheet whose content is the option menu, so there is no separate
   popup layer.
3. **Dialog stack** (`10 + index`) — a stack of `AnyView` cards. Only the
   topmost is interactive and only it draws a backdrop, so a stack of three
   dialogs dims the window once. Opening a dialog records the focused handle and
   focuses the dialog; closing it restores that handle.
4. **Notifications** (`100`) — a top-right stack that dismisses itself.

Open overlays use the same shapes as `gpui-shell`:

```text
struct ActiveDialog { content: AnyView, focus_handle: FocusHandle,
                      restore_focus: Option<WeakFocusHandle>, options: DialogOptions }
struct ActiveSheet  { content: AnyView, placement: gpui_base::Placement,
                      focus_handle: FocusHandle, restore_focus: Option<WeakFocusHandle> }
```

`DialogOptions` carries `escape_dismissable` and `backdrop_dismissable` (both
default `true`). Escape closes the topmost dismissable dialog, otherwise the
sheet (including a combobox popup); a backdrop press closes the topmost dialog
only when it allows it, and closes the sheet when no dialog is open. The root
installs the Escape key binding once per `App` and guards it with a global.

The managed host opens text dialogs/sheets through `open_message_dialog` /
`open_message_sheet`, which wrap a small `MessageView` as the `AnyView` content,
so the struct stays exactly the shell's.

The window is rooted at `gpui_component::Root` wrapping our `Root`, so
`gpui-component` tooltips (attached through component methods such as
`Button.Tooltip`) render on the window's own tooltip layer.

Overlays are native and window-scoped: the managed host opens them through the
command ingress (`OpenDialog`, `CloseDialog`, `PushNotification`, and a
component-triggered popup), and the ingress task holds the window handle so each
operation receives the current `&mut Window`. The content is built from the
request, so no overlay state crosses the ABI.

Every overlay mutation takes the current window and ends in `Context::notify`,
so the repaint is scheduled exactly like any other view change. Re-rendering the
*content* is deliberately separate: `Root::invalidate_view` calls the managed
`ShellView::refresh`, which marks the view dirty and notifies, so a clean content
snapshot is not rebuilt just because an overlay opened.

## Application and events

`GpuiApplication` owns one `View` and one native session. `View.Render` builds an
element tree; `ButtonElement.OnClick` registers a handler in the
`EventRegistry` and writes a callback token into the arena.

When the native control activates, the materialized closure calls the managed
callback with the token. The handler runs on the GPUI thread, and repaints are
explicit and scoped. A component callback or a discrete `Div` event marks the
whole view dirty after the handler returns; `OnMouseMove`/`OnScroll` do not.
Mutating `Entity<T>` state, by contrast, repaints nothing until the handler
calls `Context<T>.Notify()`: that routes through the `notify_entity` ingress and
repaints only the entity's retained subtree, because an entity host's element
render callback runs `without_invalidate`. No queue is required because
callbacks are delivered synchronously on the application thread.

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
    int32_t (*configure)(uint64_t session, uint32_t flags);
    int32_t (*set_theme)(uint64_t session, uint32_t mode, const uint8_t* colors, uint32_t);
    int64_t (*open_window)(uint64_t parent_session, uint32_t flags);
    int32_t (*close_window)(uint64_t session);
    uint64_t reserved;
};
```

`run_application` blocks in the GPUI event loop. `invalidate` is the any-thread
request behind `View.Invalidate()`: it posts to the session's view, which marks
itself dirty and repaints. `open_window`/`close_window` manage additional
top-level windows (see "Multi-window" below).

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

- A new **style** is one entry in the `style_vocabulary!` macro in `style.rs`
  (the nullary list for a `fn(self) -> Self` method, the param list plus one arm
  in `apply_param` otherwise), the matching name appended to `StyleOps.cs`, and
  the managed sugar in `StyleExtensions`. Append only: the index is the wire
  opcode, and `SCHEMA_HASH` must be bumped when the vocabulary changes.
- A new **component method** is a `MethodDescriptor` on that component's
  descriptor plus one arm in its materializer; the managed side only names the
  method.
- A new **component** is a descriptor and a materializer registered in
  `components/`, as above.

When a component id or payload rule changes, bump `SCHEMA_HASH` on both sides.
When a C record layout changes, bump `ABI_VERSION` and update both sides
together.

## Multi-window

Each top-level window is an independent **session**. The primary window uses the
managed application id; `open_window(parent, flags)` allocates a fresh session id
and asks the parent window's ingress task to run `cx.open_window` on the GPUI
thread. The child session renders through the same managed callback table, but
with its own `ShellView`, `Root`, arenas, and event registry, so the managed
`Session` is looked up per callback by session id.

- `GpuiApplication.OpenWindow(factory)` returns a `WindowHandle`; calling it
  before `Run` queues the window and it opens on the primary window's first
  frame, when the native ingress is live.
- `open_window`'s `flags` (custom title bar, always-show scrollbars) are set per
  window. `OpenWindow(factory, options)` takes a `WindowOptions` whose
  `UseCustomTitlebar` is `null` to inherit the primary window's mode, or an
  explicit `true`/`false` to override it, so one app can mix custom and system
  title bars across windows.
- `close_window(session)` posts a `Close` command; the owning window calls
  `Window::remove_window`, and GPUI quits once the last window closes.
- Native-menu actions carry their owning session, so one global action listener
  routes to the right managed callback.

## Windows apartment

GPUI's Windows platform initializes OLE as a single-threaded apartment, but a
managed entry thread is already MTA (`RPC_E_CHANGED_MODE`). `GpuiApplication.Run`
runs the native event loop on a dedicated STA thread on Windows, mirroring the
manifest requirement for Windows common controls.

## Not yet built

- rich overlay content (dialogs and notifications currently carry plain text);
- a theme payload from C# (the native host uses the gpui-component default);
- virtualized item batches;
- per-node incremental repaint. The content subtree is retained and a rebuild
  that produces the same fingerprint is skipped outright, and stateful pages
  already retain their subtree through `EntityHost`, so a change rebuilds one
  page. What is not built is diffing a *changed* description node by node to
  rebuild only the changed subtrees; that needs a stable structural key per
  node (React-style keys) that the current description does not carry.
