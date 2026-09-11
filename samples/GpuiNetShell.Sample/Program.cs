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

// The application is captured by the view so its buttons can open overlays; the
// factory runs on the native thread after this assignment.
GpuiApplication? application = null;
application = new GpuiApplication(() => new GalleryView(application!));
application.Run();
return 0;

/// <summary>
/// A small gallery: labels and a badge, a progress bar, and buttons that
/// increment, reset, open a dialog, or post a notification. The increment
/// button carries a tooltip.
/// </summary>
internal sealed class GalleryView : View
{
    private readonly GpuiApplication _application;
    private readonly string[] _themes = ["Light", "Dark", "System"];
    private int _count;
    private int _themeIndex;

    public GalleryView(GpuiApplication application)
    {
        _application = application;
    }

    protected override Element Render(ref RenderContext ui) =>
        ui.VStack(
                ui.HStack(
                        ui.Label("Counter").TextSize(18).FontSemibold(),
                        ui.Badge(_count)
                    )
                    .Gap(8)
                    .ItemsCenter()
                   ,
                ui.Text($"Count: {_count}").TextSize(28),
                ui.Progress("progress").Value(_count).P(20),
                ui.HStack(
                        ui.Label("Theme"),
                        ui.Combobox("theme")
                            .Options(_themes)
                            .Selected(_themeIndex)
                            .OnChange(index =>
                            {
                                _themeIndex = index;
                                _application.PushNotification($"Theme: {_themes[index]}");
                            })
                    )
                    .Gap(8)
                    .ItemsCenter()
                    .P(8),
                ui.HStack(
                        ui.Button("increment")
                            .Label("Increment")
                            .Primary()
                            .Tooltip("Increments the counter")
                            .OnClick(() =>
                            {
                                _count++;
                                this.Invalidate();
                            }),

                        ui.Button("reset").Label("Reset").Secondary().OnClick(() => _count = 0),
                        ui.Button("dialog")
                            .Label("Open dialog")
                            .OnClick(() =>
                                _application.OpenDialog(
                                    "About GpuiNetShell",
                                    "A C#-hosted GPUI shell: managed state, native rendering."
                                )
                            ),
                        ui.Button("sheet")
                            .Label("Open sheet")
                            .OnClick(() =>
                                _application.OpenSheet(
                                    SheetPlacement.Right,
                                    "Details",
                                    "A sheet is a place in the window, below the dialog stack."
                                )
                            ),
                        ui.Button("notify")
                            .Label("Notify")
                            .Success()
                            .OnClick(() =>
                                _application.PushNotification(
                                    $"Count is {_count}",
                                    NotificationLevel.Success
                                )
                            )
                    )
                    .Gap(8)
                    .ItemsCenter()
            .MinH(0)
            .Full()
            )
            .Gap(16)
            .P(32)
            .Full()
            .ItemsCenter()
          ;
}
