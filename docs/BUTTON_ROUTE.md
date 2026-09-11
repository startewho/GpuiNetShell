# The Button route

This is the first complete vertical slice: a C# `Button` declaration becomes a
real `gpui-component` button, and activating it calls back into managed code.
The native walk mirrors `gpui-shell`'s `materialize.rs`.

## 1. Declaration (C#)

```csharp
ui.Button("increment")
    .Label("Increment")
    .Primary()
    .OnClick(() => _count++);
```

- `RenderContext.Button` (`src/GpuiNetShell/Rendering/RenderContext.cs`) adds a
  node with `ComponentButton` and the id (`"increment"`) as node data.
- `ButtonElement` (`src/GpuiNetShell/Elements/ButtonElement.cs`) records generic
  operations: `Method("label", "Increment")`, `Method("primary")`, and
  `Callback("on_click", token)`.
- `OnClick` registers the handler in the `EventRegistry` and writes the returned
  token into the callback operation.

## 2. Arena (C#)

`RenderArena` (`src/GpuiNetShell/Rendering/RenderArena.cs`) collects the nodes,
operations, edges, and UTF-8, then `Publish()` copies them into reused unmanaged
buffers and returns a `NativeArena` descriptor. Every operation's `a` is the
packed UTF-8 range of a method name; `flags` classifies the argument in `b`.

## 3. Callback and decode (Rust)

The native host is dirty, so `ShellView::refresh`
(`crates/gpui-net-shell/src/host.rs`) calls the managed `render` callback,
receives a nonzero revision, and calls `Snapshot::decode`
(`src/snapshot.rs`):

- record flags and argument kinds must be valid;
- every operation's node index must be in range;
- name/argument ranges must be in bounds and valid UTF-8;
- numeric arguments must be finite;
- callback tokens must be nonzero;
- the graph must be acyclic.

The result is an owned `Snapshot` whose ops are `NullaryStyle`, `ParamStyle`,
`Method`, or `Callback` — no per-style or per-property operation code. On
success the host acknowledges with `render_completed(session, revision, 0)`.

## 4. Materialization (Rust)

`materialize` (`src/materialize.rs`) walks the snapshot the way the shell does:

- `resolve_ops(node)` makes one pass:
  - `NullaryStyle`/`ParamStyle` fold into a `StyleRefinement` through
    `style::apply_nullary_name` / `style::apply_param`;
  - `Method` accumulates into a `Behavior` through `apply_behavior`;
  - `Callback` sets the behavior's event token through `apply_callback`.
- `materialize_node` builds the children first.
- `materialize_component` matches the `Component`:

  ```text
  Component::Div          -> finish(div(), refinement, children)
  Component::Text(value)  -> finish(div().child(value), refinement, children)
  Component::Button(id)   -> Button::new(id)
                               .loading(behavior.loading)
                               .disabled(behavior.disabled)
                               .selected(behavior.selected)
                               // variant, size, label, tooltip, on_click
                               -> finish(button, refinement, children)
  ```

- `finish(element, refinement, children)` applies the refinement, extends the
  children, and produces the `AnyElement`.

Styles are applied only through `StyleRefinement`, so the same `finish` serves
every component. The `Button` arm reads the shared `Behavior`, exactly as the
shell's `Component::Button` arm does.

## 5. Activation

Clicking (or Enter/Space on) the button runs the closure. It calls the managed
`click(session, token)` callback, then marks the owning view dirty and notifies
it. The view re-renders, calls managed `Render` again, and publishes a new
revision and snapshot.

The click crosses the ABI as one `(session, token)` pair; the handler lookup and
all UI state stay managed.

## Tests covering the route

- Rust `snapshot::tests` — decoding styles/methods/callbacks, argument kinds,
  cycles, unknown components/operations.
- Rust `materialize::tests` — component identity, behavior accumulation, style
  folding, and that unknown methods are inert.
- Rust `style::tests` — the reflection table, parameter binding, and color
  parsing.
- C# `RenderArenaTests` / `RenderContextTests` — arena encoding and the Button
  op sequence.
- C# `EventRegistryTests` — token registration, dispatch, and reset.
- `GpuiNetShell.Sample -- --check` — loads the native host and negotiates
  ABI/schema.
