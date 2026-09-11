using GpuiNetShell;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

// --check loads the native host and negotiates the ABI/schema without opening a
// window. It is the quickest end-to-end check of the C ABI boundary.
if (args.Contains("--check", StringComparer.Ordinal))
{
    var compatible = GpuiNativeHost.Verify();
    Console.WriteLine(
        compatible
            ? $"gpui-net-shell native host OK (abi {GpuiNativeHost.AbiVersion}, schema 0x{GpuiNativeHost.SchemaHash:X16})"
            : "gpui-net-shell native host is incompatible"
    );
    return compatible ? 0 : 1;
}

// The smallest end-to-end Button route: one window, one view, one event.
var application = new GpuiApplication(() => new CounterView());
application.Run();
return 0;

/// <summary>Owns the counter state; <see cref="Render"/> describes it.</summary>
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
