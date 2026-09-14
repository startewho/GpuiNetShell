# GpuiNetShell

GpuiNetShell is a C# host for a GPUI shell runtime. It replaces the JavaScript
application layer of `gpui-shell`/`component-shell` with C#: the managed host
owns state and describes an interface, and a Rust native host owns the GPUI
application, window, event loop, validation, and materialization behind a
versioned C ABI.

The first vertical slice is **Button**, end to end: a C# declaration becomes a
real `gpui-component` button that calls back into managed code on activation.

## How it fits together

```text
C# application  (View.Render -> Element builders)
        │  fills a flat render arena (nodes / ops / children / UTF-8)
        ▼
GpuiNetShell managed runtime  (RenderArena, EventRegistry, GpuiApplication)
        │  C ABI: API table + schema hash + cdecl callbacks
        ▼
gpui-net-shell native host  (Rust)
        ├─ app host: gpui::Application, window, event loop
        ├─ snapshot decode -> owned description
        ├─ style.rs: reflected GPUI style table
        └─ materialize.rs -> ComponentRegistry descriptors -> materializer
```

The split mirrors `gpui-shell`: a description is published only when state
moves, and clean repaints replay the retained snapshot in Rust without entering
managed code. Styling is not enumerated over the ABI: the managed side sends
GPUI style method names (`items_center`, `size_full`, `p`, `gap`, `bg`, …) and
Rust resolves them against GPUI's reflected style table, exactly as `gpui-shell`
does.

Each published render is a frozen `RenderSnapshot` that owns its generation.
The native `ShellView` keeps the current and previous snapshots; when one is
replaced its generation is retired and the managed host releases exactly that
generation's event handlers. `View.Invalidate()` and `RenderContext.Notify()`
are the managed equivalents of `cx.notify()`: both request a native re-render.
`Notify()` is refused during `Render`, exactly as gpui refuses a self-notify
while rendering.

A managed `Entity<T>` renders as a retained native `EntityHost` subtree via
`ui.Child(entity, render)`. Event callbacks inside such a subtree do **not**
force a full repaint; only an explicit `Context.Notify()` repaints that subtree
through the native `notify_entity` ingress. The sample gallery uses this for
every stateful page (`GalleryPage<TState>`), so an interaction rebuilds one page
subtree instead of the whole window. A `[GpuiCallback]` method with signature
`Element M(TState, RenderContext, Context<TState>)` is an **entity view**: the
source generator emits a `PageToken` and registers it with
`ui.RegisterEntityView`, which `GalleryPage<TState>.Render` renders through
`ui.Child(Entity, token)`.

The overlay host (`src/root.rs`) wraps the content view and paints one sheet, a
dialog stack, and a notification stack over it; the managed `GpuiApplication`
drives them with `OpenDialog`, `OpenSheet`, `CloseDialog`, `CloseSheet`, and
`PushNotification`, while a `Combobox` opens its option menu as a `Bottom` sheet
(one callback token per option). Tooltips ride the `gpui-component` window root.
Components are registered descriptors in `src/components/` (`Div`, `Text`,
`Button`, `Label`, `Badge`, `Progress`, `Combobox`, `Radio`, `Tabs`, `Scroll`,
`Scrollbar`, `Resizable`, `Popover`); adding one is a descriptor, a materializer,
and a managed builder. Components with named parts — a popover's `trigger` and
`content` — receive them as slots. Every overlay mutation takes the current
window and ends in `Context::notify`, and re-rendering the content is the
separate `View.Invalidate()` step.

## Repository layout

```text
Cargo.toml                     Rust workspace
crates/gpui-net-shell/         Native host (cdylib + rlib)
  src/schema.rs                  wire vocabulary (mirrored in C#)
  src/abi.rs                     C layouts and the API table
  src/snapshot.rs                arena decode + RenderSnapshot
  src/style.rs                   reflected GPUI style table (shell-style)
  src/registry.rs                ComponentRegistry / Descriptor / Materializer
  src/components/                Div, Text, Button, Label, Badge, Progress, Combobox, Radio, Tabs, Scroll, Scrollbar, Resizable, Popover
  src/context.rs                 HostContext: session, callbacks, invalidate
  src/materialize.rs             op resolution + registry dispatch
  src/view.rs                    ShellView: dirty/current/previous + rebuild
  src/root.rs                    Root: content + sheet + dialog + notification
  src/host.rs                    GPUI application, window, ingress
  src/ffi.rs                     panic-safe C entry points
src/GpuiNetShell/              Managed runtime library
  Interop/                       layouts, P/Invoke, managed callbacks
  Rendering/                     RenderArena, RenderContext
  Entities/                      Entity<T>, Context<T>, EntityRegistry, GlobalStore, UiDispatcher
  Elements/                      Element, ButtonElement, TextElement, DivElement
  Events/                        EventRegistry
  View.cs, GpuiApplication.cs
samples/GpuiNetShell.Sample/   Tabbed component gallery (one page per component)
tests/GpuiNetShell.Tests/      Managed contract tests
external/gpui-kit/             Pinned submodule (gpui-base/gpui-component)
docs/                          Architecture and the Button route
```

## Requirements

- .NET SDK 10+
- the stable Rust toolchain and Cargo
- the platform prerequisites GPUI needs

## Build and test

```sh
cargo test -p gpui-net-shell
dotnet test GpuiNetShell.slnx
dotnet build samples/GpuiNetShell.Sample/GpuiNetShell.Sample.csproj
```

Building the managed library also builds the native host (`cargo build -p
gpui-net-shell`) and copies the shared library next to the output.

On Windows the executing app must embed a Common Controls v6 manifest because
the native host imports `TaskDialogIndirect`; see
`samples/GpuiNetShell.Sample/app.manifest`.
`GpuiNativeHost.Verify()` is a quick way to check the load path without opening
a window.

## Run

```sh
dotnet run --project samples/GpuiNetShell.Sample            # opens the window
dotnet run --project samples/GpuiNetShell.Sample -- --check # ABI/schema check
```

## Small application

```csharp
using GpuiNetShell;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

var application = new GpuiApplication(() => new CounterView());
application.Run();

internal sealed class CounterView : View
{
    private int _count;

    protected override Element Render(ref RenderContext ui) =>
        ui.VStack(
                ui.Text($"Count: {_count}").TextSize(24),
                ui.Button("increment")
                    .Label("Increment")
                    .Primary()
                    .OnClick(() => _count++)
            )
            .Gap(12)
            .P(24)
            .Full()
            .ItemsCenter()
            .JustifyCenter();
}
```

`OnClick` runs on the native application thread; the host requests a re-render
after it returns, so the handler only mutates state.
