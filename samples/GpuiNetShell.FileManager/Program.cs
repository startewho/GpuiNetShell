using System.Reflection.Metadata;
using GpuiNetShell;
using GpuiNetShell.FileManager;

// The hot-reload handler lives in GpuiNetShell.dll; declaring the attribute in
// this assembly is what makes an edit to it trigger a repaint under `dotnet watch`.
[assembly: MetadataUpdateHandler(typeof(GpuiNetShell.HotReload))]

// --check loads the native host and negotiates the ABI/schema without opening a
// window; it is the quickest end-to-end check of the C ABI boundary.
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

// --path=<dir> opens the file manager at a directory other than the default.
var pathArgument = args.FirstOrDefault(argument =>
    argument.StartsWith("--path=", StringComparison.Ordinal)
);
var initialPath = pathArgument is not null ? pathArgument["--path=".Length..] : null;

GpuiApplication? application = null;
FileManagerView? view = null;
application = new GpuiApplication(() => view!);
view = new FileManagerView(application, initialPath);
application.UseCustomTitlebar = true;
application.WindowTitle = "文件资源管理器";
application.AlwaysShowScrollbars = true;
application.Run();
return 0;
