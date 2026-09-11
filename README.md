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
        ├─ arena decode -> owned ValidatedSnapshot
        ├─ component registry (Div, Text, Button)
        └─ materializers -> gpui-component / gpui-base elements
```

The split mirrors `gpui-shell`: a description is published only when state
moves, and clean repaints replay the retained snapshot in Rust without entering
managed code. Styling is not enumerated over the ABI: the managed side sends
GPUI style method names (`items_center`, `size_full`, `p`, `gap`, `bg`, …) and
Rust resolves them against GPUI's reflected style table, exactly as `gpui-shell`
does.

## Repository layout

```text
Cargo.toml                     Rust workspace
crates/gpui-net-shell/         Native host (cdylib + rlib)
  src/schema.rs                  wire vocabulary (mirrored in C#)
  src/abi.rs                     C layouts and the API table
  src/snapshot.rs                arena decode + validation
  src/style.rs                   reflected GPUI style table (shell-style)
  src/registry.rs                component descriptors
  src/components/                Div, Text, Button materializers
  src/materialize.rs             recursive dispatch
  src/host.rs                    GPUI application and window
  src/ffi.rs                     panic-safe C entry points
src/GpuiNetShell/              Managed runtime library
  Interop/                       layouts, P/Invoke, managed callbacks
  Rendering/                     RenderArena, RenderContext
  Elements/                      Element, ButtonElement, TextElement, DivElement
  Events/                        EventRegistry
  View.cs, GpuiApplication.cs
samples/GpuiNetShell.Sample/   Counter sample with a Button
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
