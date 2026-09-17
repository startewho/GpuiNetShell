---
name: gpui-net-shell-csharp
description: Use when working in the C# managed half of GpuiNetShell (src/GpuiNetShell, "GpuiNetShell.dll", managed runtime, RenderArena, RenderContext, StyleExtensions, EventRegistry, Entity<T>, GpuiApplication, NativeProtocol, StyleOps, MethodOps, the GpuiCallback source generator). Covers the element-declaration API, the render arena, the event/callback model, the ABI mirror, and how to add an element/component page plus the exact build/test commands.
---

# GpuiNetShell — C# managed runtime

`src/GpuiNetShell` owns application state, `View` instances, the element
declaration API, the flat render arena, and event handlers. It publishes a
description across a versioned C ABI; the Rust host (`crates/gpui-net-shell`)
owns the window, event loop, and materialization. See the sibling skill
`gpui-net-shell-rust`.

The boundary rule: **cross for a state transition (a dirty render, an event),
never per builder call, per frame, or per property.**

## Project map (`src/GpuiNetShell/`)

| Path | Responsibility |
| --- | --- |
| `Interop/NativeProtocol.cs` | Mirrors `crates/gpui-net-shell/src/schema.rs`: `AbiVersion`, `SchemaHash`, component ids, op/arg codes, statuses. |
| `Interop/StyleOps.cs` | Mirrors the style vocabulary in `style.rs` (`Nullary`/`Param` arrays, `TryNullary`/`TryParam`). Index is the opcode. |
| `Interop/MethodOps.cs` | `Code(name)` — FNV-1a 64 of a component method name, matching `schema::method_code`. |
| `Interop/NativeLayouts.cs` | `[StructLayout]` mirrors of the `#[repr(C)]` records. |
| `Interop/NativeMethods.cs`, `ManagedCallbacks.cs` | P/Invoke and the managed `UnmanagedCallersOnly` entry points. |
| `Rendering/RenderArena.cs` | The flat arena (nodes/ops/children/UTF-8) and `Publish`. |
| `Rendering/RenderContext.cs` | The element-declaration surface passed to `View.Render`. |
| `Elements/` | `Element` base, one builder per component, `StyleExtensions`, `Length`, `PaintPrimitives`. |
| `Entities/` | `Entity<T>`, `Context<T>`, `EntityRegistry`, `GlobalStore`, `UiDispatcher`. |
| `Events/EventRegistry.cs` | Token → handler tables, generation retirement, stable/keyed registration. |
| `Events/ElementEvents.cs` | `DivEvent` plus the typed payloads (`PointerEvent`, `ScrollEvent`, `KeyEvent`, `ClickEvent`, `PressureEvent`) decoded from the native payload. |
| `View.cs` | Base view: `Render`, `RenderRoot`, `Invalidate`, `Notify`, `DispatchInput`, `AttachInvalidator`. |
| `GpuiApplication.cs` | `Session` per window, `RenderInto`, the native callbacks, multi-window, overlays. |
| `GpuiNativeHost.cs` | `AbiVersion`, `SchemaHash`, `Verify()`. |
| `HotReload.cs`, `WindowOptions.cs`, `ThemeMode.cs` | Hot reload hooks, per-window options, theme mode. |
| `../GpuiNetShell.SourceGen/GpuiCallbackGenerator.cs` | `[GpuiCallbacks]`/`[GpuiCallback]` → token properties + guarded registration. |

## Core invariants — do not break these

1. **The ABI mirror must stay in lockstep with `schema.rs`.** `SCHEMA_HASH` and
   `ABI_VERSION` are pinned by tests on both sides. When you change a component
   id, op code, or payload rule, update `NativeProtocol.cs` **and** `schema.rs`
   and bump `SCHEMA_HASH`. When a record layout changes, bump `ABI_VERSION`.
2. **Styles are u16 opcodes, component methods are u64 codes — never strings.**
   `RenderArena` maps a style name through `StyleOps` and a method name through
   `MethodOps` before publishing. Unknown names produce no op (a typo is inert,
   as before).
3. **`RenderArena` is reused every frame; do not allocate per string/op.**
   - `Reset()` keeps capacity; `Publish()` copies into unmanaged buffers.
   - Dynamic values encode straight into `_utf8` (no intermediate `byte[]`).
   - Invariant names (`PackName`) use a static UTF-8 cache.
   - `PublishBuffer` copies via `CollectionsMarshal.AsSpan` (no `ToArray()`).
   - Buffer capacity grows geometrically.
   - `RenderArena` is `IDisposable` **and** has a finalizer; `Session.OnWindowClosed`
     disposes both arenas. Do not drop unmanaged buffers on the floor.
4. **Callback tokens are generation-scoped unless registered stable.**
   `Register*` tokens are retired by `retire_callbacks(generation)` when their
   snapshot is replaced. `RegisterStable*` (keyed) and the keyed
   `RegisterEntityView`/`RegisterPersistentElement` (which **reuse** their token
   on re-registration) are not.
5. **The managed side never blocks the GPUI thread.** Handlers run on the
   application thread; after they return the host marks the view dirty. Do not
   call `Notify()` during `Render` (it throws, matching gpui).

## The render arena

`Session.RenderInto` (`GpuiApplication.cs`) runs once per dirty frame:

```csharp
_arena.Reset();
_events.BeginGeneration(generation);
_root ??= RootFactory();
_root.AttachInvalidator(() => Owner.InvalidateSession(SessionId));
_renderContext.BeginRender();
var element = _root.RenderRoot(ref _renderContext);
_renderContext.EndRender();
*arena = _arena.Publish();
*root  = (uint)element.Index;
```

Four flat buffers describe one render: `NativeNode[]` (component id + UTF-8 data
range), `NativeOp[]` (`a`/`b`/`c` words interpreted by `Code`), `NativeChild[]`
edges, and one UTF-8 byte buffer. Element callback subtrees render into a
separate scratch arena (`_elementArena`) from `OnRenderElement`.

`StyleExtensions` is only sugar over `Style(name)` / `Style(name, value)` /
`StyleString` / `StyleColor`; it names methods and nothing else. Style names are
mapped to opcodes in `RenderArena`, so `StyleExtensions` does not need to know
about opcodes.

## Callback registration

`EventRegistry` keeps three token→handler tables (handlers, row providers,
element renderers) plus generation and scope bookkeeping.

- `RegisterCallback(Action)`, `RegisterCallback(Action<EventValue>)`,
  `RegisterRows`, `RegisterElement` → **generation-scoped** tokens.
- `RegisterStableCallback`/`RegisterStableRows`/`RegisterStableElement`/
  `RegisterStableMeasure` → **stable** tokens keyed by name; re-registration
  replaces the handler and keeps the token.
- `RegisterEntityView(key, renderer)` and
  `RegisterPersistentElement(entityId, renderer)` → keyed, stable, and they
  **reuse** the token so a retained subtree that references it keeps working.
- `BeginCallbackScope(token)`/`EndCallbackScope()` bound the handlers an element
  callback registers; the previous invocation's handlers are retired when the
  next invocation starts. Generation and scope token lists are pooled.

## Source-generated callbacks

Mark a partial class `[GpuiCallbacks]` and a method `[GpuiCallback("Name")]`.
The generator infers the kind from the signature and emits a `NameToken`
property plus a `RegisterGeneratedCallbacks(ref RenderContext)` method that:

- registers **once** (guarded by `_gpuiCallbacksRegistered`), and
- uses the **stable** registration methods with a full-name key.

Call it at the top of `Render` and hand the token to the element that should
invoke the callback. Entity views use `RegisterEntityView` and render through
`ui.Child(entity, token)`.

## Div element events

`DivElement` can subscribe to GPUI element events: `OnClick`, `OnAuxClick`,
`OnHover`, `OnMouseDown/Up/Move/DownOut/UpOut`, `OnMousePressure`, `OnScroll`,
`OnKeyDown/Up`, plus a generic `On(id, DivEvent, Action<EventValue>)`.

- **Opt-in**: each method writes exactly one `Op::Callback(name, token)`; the
  native host binds a GPUI listener only for the subscribed events. Unsubscribed
  events cost nothing.
- **Stable id**: click, aux-click, and hover are keyed by GPUI on the element
  id, so those methods take an `id` that must be unique in the window and
  unchanged while rendered (the native side writes it as an `element_id` method
  op). Stateless events take no id.
- **Payloads** are decoded into `PointerEvent`/`ScrollEvent`/`KeyEvent`/
  `ClickEvent`/`PressureEvent` (`Events/ElementEvents.cs`).
- **Repaint**: discrete events repaint after the handler; `OnMouseMove` and
  `OnScroll` do not, so call `Notify()`/`Invalidate()` from those handlers if
  they change what `Render` reads.
- Only `Div` supports these; `AnyElement` has no event methods, so other
  components keep their dedicated callbacks.

## How to add a managed element / component page1. Add the component id and any method codes in `NativeProtocol.cs`, matching
   `schema.rs` (never reorder ids).
2. Add a builder in `Elements/` deriving from `Element`, and a factory in
   `RenderContext`.
3. Style with the existing `StyleExtensions` sugar; do not invent style names
   outside `StyleOps` (they are dropped).
4. Add a gallery page under `samples/GpuiNetShell.Sample/Pages/` and, if it has
   state, use `GalleryPage<TState>` so it renders as a retained entity subtree.

## Build, test, verify

Run from the repository root:

```sh
dotnet build GpuiNetShell.slnx
dotnet test GpuiNetShell.slnx
dotnet run --project samples/GpuiNetShell.Sample -- --check   # ABI/schema negotiation
```

`GpuiNativeHost.Verify()` loads the host and compares ABI version and schema
hash; `--check` runs that without opening a window. Building
`src/GpuiNetShell/GpuiNetShell.csproj` also builds the native host (set
`SkipNativeBuild=true` to use a prebuilt one).

### Allocation budget

`tests/GpuiNetShell.Tests/AllocationBaselineTests.cs` renders a ~400-node tree
for 200 frames and asserts the per-frame managed allocation stays under a
ceiling. Treat it as a regression guard: **raise the ceiling deliberately, never
to silence a leak.** A representative frame is ~21 KB/frame today.

## Pitfalls

- `params Element[]` overloads allocate an array at the call site; for hot trees
  prefer explicit small-arity overloads or build the array once.
- Every `RenderContext` factory allocates one `Element` object per node per
  frame; that is inherent to the fluent API.
- `string + char` for the constructor separator boxes the `char`; prefer the
  arena's span/join helpers when adding composite node data.
- Do not format per-frame keyed ids or repeat `Encoding.UTF8.GetBytes` for
  invariant names — both have cached paths.
- After changing `SCHEMA_HASH`/`ABI_VERSION`, rebuild both sides; a mismatch
  throws in `RunCore`.
