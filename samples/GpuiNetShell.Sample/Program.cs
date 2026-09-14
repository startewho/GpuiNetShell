using System.Reflection.Metadata;
using GpuiNetShell;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;
using GpuiNetShell.Sample.Pages;

// The hot-reload handler lives in GpuiNetShell.dll, but the runtime only sees
// handlers declared by the assembly being edited. Declaring the attribute here
// (the app assembly) is what makes an edit trigger a repaint.
[assembly: MetadataUpdateHandler(typeof(GpuiNetShell.HotReload))]

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

var initialPage = 0;
var pageArgument = args.FirstOrDefault(argument =>
    argument.StartsWith("--page=", StringComparison.Ordinal)
);
if (
    pageArgument is not null
    && int.TryParse(pageArgument["--page=".Length..], out var parsedPage)
)
{
    initialPage = parsedPage;
}

GpuiApplication? application = null;
GalleryView? gallery = null;
application = new GpuiApplication(() => gallery!);
gallery = new GalleryView(application, initialPage);
application.UseCustomTitlebar = true;
application.AlwaysShowScrollbars = true;

// A dev affordance for hunting retention leaks: cycle through the pages on a
// timer, forcing a full GC and printing managed/working-set memory every N
// switches. `--cycle-pages=150` sets the interval in milliseconds.
var cycleArgument = args.FirstOrDefault(argument =>
    argument.StartsWith("--cycle-pages", StringComparison.Ordinal)
);
if (cycleArgument is not null)
{
    var interval = 150;
    var separator = cycleArgument.IndexOf('=');
    if (
        separator >= 0
        && int.TryParse(cycleArgument[(separator + 1)..], out var parsed)
        && parsed > 0
    )
    {
        interval = parsed;
    }
    var cycleSetArgument = args.FirstOrDefault(argument =>
        argument.StartsWith("--cycle-set=", StringComparison.Ordinal)
    );
    if (cycleSetArgument is not null)
    {
        var indices = cycleSetArgument["--cycle-set=".Length..]
            .Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Select(value => int.TryParse(value, out var parsed) ? parsed : -1)
            .Where(value => value >= 0)
            .ToArray();
        gallery.SetCycleSet(indices);
    }
    var view = gallery;
    _ = Task.Run(async () =>
    {
        var switches = 0;
        while (true)
        {
            await Task.Delay(interval);
            GpuiNetShell.Entities.UiDispatcher.Post(() =>
            {
                switches++;
                view.CyclePage();
            });
            if (switches % 25 == 0)
            {
                GC.Collect();
                GC.WaitForPendingFinalizers();
                GC.Collect();
                Console.WriteLine(
                    $"[cycle {switches}] managed={GC.GetTotalMemory(true) / (1024 * 1024)}MB "
                        + $"ws={Environment.WorkingSet / (1024 * 1024)}MB"
                );
            }
        }
    });
}

// A dev affordance for verifying multi-window deterministically: open N child
// windows before the event loop starts. Every window is its own session.
var openWindowsArgument = args.FirstOrDefault(argument =>
    argument.StartsWith("--open-windows=", StringComparison.Ordinal)
);
if (
    openWindowsArgument is not null
    && int.TryParse(openWindowsArgument["--open-windows=".Length..], out var openCount)
)
{
    for (var i = 1; i <= openCount; i++)
    {
        var ordinal = i;
        WindowHandle? handle = null;
        var child = new SecondaryWindowView(
            application,
            ordinal,
            $"Child window {ordinal}",
            () => handle?.Close()
        );
        handle = application.OpenWindow(() => child);
    }
}

// A dev affordance for verifying per-window title bars: open one window that
// inherits the parent, one with a custom title bar, and one with the system
// title bar.
if (args.Contains("--open-mixed", StringComparer.Ordinal))
{
    OpenChild(1, "Inherit", null);
    OpenChild(2, "Custom", true);
    OpenChild(3, "System", false);

    void OpenChild(int ordinal, string label, bool? useCustomTitlebar)
    {
        WindowHandle? handle = null;
        var child = new SecondaryWindowView(
            application!,
            ordinal,
            $"{label} titlebar",
            () => handle?.Close()
        );
        handle = application!.OpenWindow(() => child, new WindowOptions(useCustomTitlebar));
    }
}

// A dev affordance for verifying programmatic close: close the first child
// window after a delay, exercising `WindowHandle.Close` -> native `close_window`.
var closeAfterArgument = args.FirstOrDefault(argument =>
    argument.StartsWith("--close-after=", StringComparison.Ordinal)
);
if (
    closeAfterArgument is not null
    && int.TryParse(closeAfterArgument["--close-after=".Length..], out var closeAfterMs)
)
{
    var target = application.OpenWindow(() => new SecondaryWindowView(
        application,
        99,
        "Auto-close window",
        () => { }
    ));
    _ = Task.Run(async () =>
    {
        await Task.Delay(closeAfterMs);
        target.Close();
    });
}

application.Run();
return 0;

/// <summary>
/// The gallery shell: a left navigation sidebar and a right content area that
/// shows exactly one control page at a time.
/// </summary>
internal sealed class GalleryView : View
{
    private readonly IReadOnlyList<GalleryPage> _pages;
    private int _index;
    private bool _collapsed;

    public GalleryView(GpuiApplication application, int initialPage)
    {
        _pages = PageRegistry.Create();
        foreach (var page in _pages)
        {
            page.Host = this;
            page.Application = application;
        }
        _index = Math.Clamp(initialPage, 0, _pages.Count - 1);
    }

    /// <summary>Dev affordance: advance to the next page and request a repaint.</summary>
    internal void CyclePage()
    {
        if (_cycleSet is { Length: > 0 } set)
        {
            _cyclePosition = (_cyclePosition + 1) % set.Length;
            _index = Math.Clamp(set[_cyclePosition], 0, _pages.Count - 1);
        }
        else
        {
            _index = (_index + 1) % _pages.Count;
        }
        Invalidate();
    }

    private int[]? _cycleSet;
    private int _cyclePosition;

    /// <summary>Dev affordance: restrict page cycling to specific page indices.</summary>
    internal void SetCycleSet(int[] indices) => _cycleSet = indices;

    protected override Element Render(ref RenderContext ui)
    {
        var items = new List<Element>(_pages.Count);
        for (var i = 0; i < _pages.Count; i++)
        {
            var page = _pages[i];
            var index = i;
            items.Add(
                ui.SidebarMenuItem(page.Title)
                    .Icon(page.Icon)
                    .Selected(i == _index)
                    .OnClick(() =>
                    {
                        _index = index;
                        Invalidate();
                    })
            );
        }

        var sidebar = ui.Sidebar("gallery-nav")
            .Side(SideKind.Left)
            .Collapsible(SidebarCollapsibleKind.Icon)
            .Collapsed(_collapsed)
            .Header(ui.Label("gpui-net-shell").TextSize(16).FontSemibold())
            .Footer(
                ui.HStack(
                        ui.SidebarToggleButton()
                            .Collapsed(_collapsed)
                            .OnClick(() =>
                            {
                                _collapsed = !_collapsed;
                                Invalidate();
                            }),
                        ui.Label(_pages[_index].Title).TextSize(12)
                    )
                    .Gap(8)
                    .ItemsCenter()
            )
            .Add(ui.SidebarMenu().Add(items.ToArray()))
            .W(200)
            .HFull();

        var content = ui
            .Scroll("gallery-content")
            .FlexGrow(1.0)
            .MinH(0)
            .P(24)
            .Add(_pages[_index].Render(ref ui));

        return ui.HStack(ui.Div(sidebar).FlexShrink(0), content)
            .Gap(16)
            .P(16)
            .Full();
    }
}
