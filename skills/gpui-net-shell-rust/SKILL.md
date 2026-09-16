---
name: gpui-net-shell-rust
description: Use when working in the Rust native host of GpuiNetShell (crates/gpui-net-shell, "gpui_net_shell", native host, GPUI materialization, ABI, schema hash, components/materializers, style opcodes, snapshot/prepare, ShellView, HostContext, C ABI in ffi.rs/abi.rs/schema.rs). Covers architecture, invariants, how to add a style or component, and the exact test/build/lint commands.
---

# GpuiNetShell — Rust native host

The crate `crates/gpui-net-shell` (cdylib name `gpui_net_shell`) is the native
half: it owns the `gpui::Application`, the window and event loop, decoding of
the managed arena, validation, and materialization into GPUI elements. It never
exposes a GPUI type across the C ABI, and it never retains a managed pointer.

The managed half lives in `src/GpuiNetShell` (C#). See the sibling skill
`gpui-net-shell-csharp`.

## Module map (`crates/gpui-net-shell/src/`)

| File | Responsibility |
| --- | --- |
| `lib.rs` | Module list and the crate's public re-exports (`Node`, `Op`, `RenderSnapshot`, `Snapshot`, `StyleArg`). |
| `abi.rs` | `#[repr(C)]` layouts: `GpuiNetNode`, `GpuiNetOp`, `GpuiNetChild`, `GpuiNetArena`, `GpuiNetCallbacks`, `GpuiNetShellApi`. |
| `schema.rs` | Wire vocabulary: `ABI_VERSION`, `SCHEMA_HASH`, component ids, op codes, arg codes, `method_code()`. Mirrored in C# `NativeProtocol.cs`. |
| `snapshot.rs` | `Node`, `Op`, `Snapshot` (root/nodes/prepared/fingerprint), `RenderSnapshot` (generation-owning handle), `Snapshot::decode`, `compute_fingerprint`. |
| `style.rs` | Closed style vocabulary: the `style_vocabulary!` macro, `NULLARY`/`PARAM`, `apply_nullary`, `apply_param`, `StyleArg`. |
| `registry.rs` | `ComponentRegistry`/`FrozenComponentRegistry`, `ComponentDescriptor`, `MethodDescriptor`, `ConstructorDescriptor`, `ComponentPayload`, `PreparedNode`, `NodeFactory`, `MaterializeRequest`. |
| `materialize.rs` | `prepare` (once per description) and `materialize`/`materialize_node` (per repaint). |
| `context.rs` | `HostContext` (session, callbacks, invalidate, entity hosts, row cache/scratch) and its type aliases. |
| `view.rs` | `ShellView` (dirty/current/fingerprint + retained `ContentHost`) and `ContentHost`. |
| `root.rs` | Window root: content plus sheet, dialog stack, and notification layers. |
| `host.rs` | `gpui::Application`, window creation, ingress commands, per-window sessions, the frozen catalog. |
| `ffi.rs` | Panic-safe C entry points: `gpui_net_shell_get_api`, `gpui_net_shell_abi_version`, `gpui_net_shell_schema_hash`. |
| `typed_child.rs`, `menu_action.rs` | Typed child hand-off and native-menu action routing. |
| `components/` | One file per component (descriptor + materializer + `register`); `mod.rs` builds the catalog. |

## Core invariants — do not break these

1. **The ABI is the only representation Rust accepts from C#.** Everything is
   validated and copied in `Snapshot::decode` before use. Never cache a raw
   pointer past the `render` callback.
2. **Schema and ABI are versioned and mirrored.** `schema.rs` and
   `src/GpuiNetShell/Interop/NativeProtocol.cs` must agree. Tests pin the
   literals on both sides (`schema::tests::schema_hash_is_pinned`,
   `NativeProtocolTests.SchemaHashMatchesTheNativeLiteral`).
   - Changing a component id, an op code, or a payload rule → bump
     `SCHEMA_HASH` on both sides.
   - Changing a `#[repr(C)]` record layout → bump `ABI_VERSION` and update both
     sides together.
3. **Styling is a closed opcode vocabulary, not reflection.** The native crate
   must **not** enable `gpui-base/inspector`. `style.rs` declares every style
   name once via `style_vocabulary!`; the index is the wire opcode and each
   entry is a direct GPUI method call.
4. **Component method names never cross the ABI.** They travel as
   `schema::method_code(name)` (FNV-1a 64) and the registry resolves the code
   back to a name via `FrozenComponentRegistry::method_name`.
   `disabled`/`selected` are reserved names in that table.
5. **`prepare` resolves once; `materialize` borrows.** A repaint must not
   re-resolve ops, rebuild payloads, or re-record methods.
6. **Callback lifetime is tied to a description, not a frame.**
   `RenderSnapshot`'s `Drop` calls the managed `retire_callbacks` for its
   generation. There is no `previous` snapshot: replacing `current` retires the
   old generation, and an unchanged description keeps its generation alive.

## The render path, end to end

```text
ShellView::render
  dirty?  ── no ──▶ return the retained ContentHost (GPUI reuses its element tree)
  yes
  render callback (session, generation) ──▶ managed fills the arena
  Snapshot::decode ──▶ prepare (folds ops, sets fingerprint)
      fingerprint equal? ── yes ──▶ keep `current`, retire the new generation
                           no  ──▶ swap `current`, push into ContentHost, notify it
```

- `prepare(registry, &mut Snapshot)` sets `PreparedNode` per node and
  `Snapshot::fingerprint`. `materialize::materialize_node` reads
  `snapshot.prepared[id]` and only builds GPUI elements + children.
- `compute_fingerprint` hashes components, data, style/method codes, args,
  slots, and child edges, but **masks callback tokens** (they are new every
  generation and do not change the interface).
- The fingerprint-skip is what makes a redundant `View.Invalidate()` free.

## How to add a style

Styles are declared in Rust and mirrored in C#.

1. Append the name (never reorder) to the correct list in
   `crates/gpui-net-shell/src/style.rs`:
   - no-argument `fn(self) -> Self` → the `nullary { ... }` list (dispatched by
     a direct call);
   - one argument → the `param { ... }` list **and** one arm in `apply_param`.
2. Append the same name to the matching array in
   `src/GpuiNetShell/Interop/StyleOps.cs`.
3. Expose managed sugar in
   `src/GpuiNetShell/Elements/StyleExtensions.cs`.
4. Bump `SCHEMA_HASH` (both `schema.rs` and `NativeProtocol.cs`).
5. `cargo test -p gpui-net-shell` — `style::tests::the_managed_vocabulary_matches`
   parses `StyleOps.cs` and fails if the two lists (or their order) drift, and
   `the_vocabulary_is_closed_and_pinned` pins the exact order.

The order of both lists **is** the wire contract; a reorder is a silent wire
break, which is why it is pinned.

## How to add a component

1. Add a component id constant in `schema.rs` **and** `NativeProtocol.cs`
   (append at the end; ids are registry indices).
2. Add `src/components/<name>.rs` with a `ComponentMaterializer` and a
   `pub(super) fn register(registry: &mut ComponentRegistry)`.
   - Register it from `src/components/mod.rs` (`catalog()`).
   - Constructors/methods are `ConstructorDescriptor`/`MethodDescriptor`; a
     method recorder returns a `ComponentPayload`.
3. Add the managed builder in `src/GpuiNetShell/Elements/` and a factory in
   `RenderContext`.
4. Bump `SCHEMA_HASH`. Component behavior method names need no schema change
   (they are method codes), but a **new** method name only resolves if its
   `method_code` is in the frozen registry table — that happens automatically
   from the descriptor.
5. `cargo test -p gpui-net-shell` and add a sample page under
   `samples/GpuiNetShell.Sample/Pages/`.

Slots: a component with named parts reads them from a node's `Slot` ops via
`MaterializeRequest::take_slot`/`take_slot_factory`; `materialize_node` already
routes them out of the ordinary children in `prepare`.

## Components and materialization details

- `ComponentPayload` wraps `Rc<dyn Any>` (not `Arc`): payloads are built and
  consumed on the GPUI thread within one generation.
- `NodeFactory` holds `Rc<FrozenComponentRegistry>`, `Rc<Snapshot>`, and a
  `HostContext`; cloning it is cheap (Rc bumps). Bound the number of clones per
  node.
- `MaterializeRequest` exposes: `payload`, `methods`, `style`, `host`,
  `disabled`, `selected`, `on_click`, `resolve_callback`,
  `resolve_element_callback`, `resolve_element`, `resolve_rows`,
  `take_children*`, `take_slot*`, `use_keyed_state`, `update_entity`,
  `with_window_app`, `finish`.
- `resolve_rows` reuses `HostContext.row_scratch` and caches `Rc<Vec<Row>>` per
  callback token in `HostContext.row_cache` (cleared when a description is
  built). Do not allocate a fresh buffer per call.

## HostContext

`HostContext { session_id, callbacks, invalidate, entity_hosts, row_scratch, row_cache }`
is `Clone` and shared into payloads, factories, and callbacks. The
`invalidate` closure is created once per `ShellView` (in `ShellView::new`), not
per render. `without_invalidate()` returns a copy whose invalidate is a no-op
(used by entity subtrees).

## Build, test, lint

Run from the repository root:

```sh
cargo build -p gpui-net-shell            # debug host
cargo build -p gpui-net-shell --release  # release host (LTO fat, stripped)
cargo test -p gpui-net-shell             # unit tests
cargo fmt --check
cargo clippy -p gpui-net-shell --tests -- -D warnings
```

The release profile in the workspace `Cargo.toml` sets `lto = "fat"`,
`codegen-units = 1`, `strip = "symbols"`. Do not enable
`gpui-base/inspector`; the release size was hard-won.

## Pitfalls

- Do not add backward-compatible "name" fallbacks to the wire. Styles are u16
  opcodes and methods are u64 codes; unknown codes are dropped.
- `Snapshot`/`Node`/`Op` are not `Send`; keep them on the GPUI thread.
- `RenderSnapshot` retirement is a managed callback in `Drop`; never construct
  one without intent to retire.
- `ffi.rs` wraps every entry point in `catch_unwind`; a panic becomes a status
  code. Keep `panic = unwind` (the release profile does not set `abort`).
- After changing a `SCHEMA_HASH`/`ABI_VERSION`, rebuild **both** sides; the
  sample's `--check` verifies the negotiated pair.
