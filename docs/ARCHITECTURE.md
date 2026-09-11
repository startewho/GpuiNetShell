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
first mount, by an event binding, or by `View.Invalidate()` / `RenderContext.Notify()`.

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

The built-in catalog is `Div`, `Text`, `Button`, `Label`, `Badge`, `Progress`,
and `Combobox`; each is one file in `src/components/`.

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
    int32_t (*open_dialog)(uint64_t, const uint8_t* title, uint32_t, const uint8_t* body, uint32_t);
    int32_t (*close_dialog)(uint64_t session);
    int32_t (*open_sheet)(uint64_t, uint32_t placement, const uint8_t* title, uint32_t, const uint8_t* body, uint32_t);
    int32_t (*close_sheet)(uint64_t session);
    int32_t (*push_notification)(uint64_t, const uint8_t* message, uint32_t, uint32_t level);
    uint64_t reserved;
};
```

`run_application` blocks in the GPUI event loop. `invalidate` is the any-thread
request behind `View.Invalidate()`: it posts to the session's view, which marks
itself dirty and repaints. `open_dialog`, `close_dialog`, `open_sheet`,
`close_sheet`, and `push_notification` post overlay commands to the same
ingress; `Root` applies them on the GPUI thread with the window handle it
captured when the window opened.

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

- rich overlay content (dialogs and notifications currently carry plain text);
- retained controls beyond Button (Input, Slider, Scroll, List/Table);
- multi-window support and window options over the ABI;
- a theme payload from C# (the native host uses the gpui-component default);
- hot reload of managed code;
- virtualized item batches.
