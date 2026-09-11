# The Button route

This is the first complete vertical slice: a C# `Button` declaration becomes a
real `gpui-component` button, and activating it calls back into managed code.

## 1. Declaration (C#)

```csharp
ui.Button("increment")
    .Label("Increment")
    .Primary()
    .OnClick(() => _count++);
```

- `RenderContext.Button` (`src/GpuiNetShell/Rendering/RenderContext.cs`) adds a
  node with `ComponentButton` and the id (`"increment"`) as node data.
- `ButtonElement` (`src/GpuiNetShell/Elements/ButtonElement.cs`) appends
  operations: `OpLabel`, `OpButtonVariant(Primary)`, and `OpOnClick(token)`.
- `OnClick` registers the handler in the `EventRegistry` and writes the returned
  token into the arena.

## 2. Arena (C#)

`RenderArena` (`src/GpuiNetShell/Rendering/RenderArena.cs`) collects the nodes,
operations, edges, and UTF-8, then `Publish()` copies them into reused unmanaged
buffers and returns a `NativeArena` descriptor.

## 3. Callback and decode (Rust)

The native host is dirty, so `ShellView::refresh`
(`crates/gpui-net-shell/src/host.rs`) calls the managed `render` callback. It
fills the arena, returns a nonzero revision, and the native host calls
`Snapshot::decode` (`src/snapshot.rs`):

- flags and `b` word must be zero;
- the node index of every operation must be in range;
- string ranges must be in bounds and valid UTF-8;
- `OpButtonVariant` must be a known operation;
- the graph must be acyclic.

On success the host acknowledges with `render_completed(session, revision, 0)`
and retains the owned snapshot.

## 4. Materialization (Rust)

`materialize_node` (`src/materialize.rs`) looks up `ComponentButton` in the
`ComponentRegistry` and calls `components::button::materialize`.

`button::plan` (`src/components/button.rs`) turns the node into a `ButtonPlan`:

```text
id          <- node data
label       <- OpLabel data
variant     <- OpButtonVariant
size        <- OpButtonSize
loading     <- OpLoading
compact     <- OpCompact
disabled    <- OpDisabled
selected    <- OpSelected
on_click    <- OpOnClick
```

`materialize` then builds a `gpui_component::button::Button`:

```rust
Button::new(plan.id)
    .loading(plan.loading)
    .disabled(plan.disabled)
    .selected(plan.selected)
    // ...variant, size, label, tooltip, shared style ops, children...
    .on_click(move |_event, _window, cx| { /* managed click + invalidate */ })
```

Children are materialized first and handed to `.children(...)`, matching GPUI's
consumed-element model.

## 5. Activation

Clicking (or Enter/Space on) the button runs the closure. It calls the managed
`click(session, token)` callback, then marks the owning view dirty and notifies
it. The view re-renders, calls managed `Render` again, and publishes a new
revision and snapshot.

The click crosses the ABI as one `(session, token)` pair; the handler lookup and
all UI state stay managed.

## Tests covering the route

- Rust `snapshot::tests` — arena decode, string ranges, cycles, unknown
  components.
- Rust `components::button::tests` — identity, operation interpretation, and
  that every variant constructs a real `Button`.
- Rust `registry::tests` — builtin descriptors and lookup.
- C# `RenderArenaTests` / `RenderContextTests` — arena encoding and the Button
  op sequence.
- C# `EventRegistryTests` — token registration, dispatch, and reset.
- `GpuiNetShell.Sample -- --check` — loads the native host and negotiates
  ABI/schema.
