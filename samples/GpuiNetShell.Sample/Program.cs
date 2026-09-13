using GpuiNetShell;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;
using GpuiNetShell.Sample.Pages;

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
application = new GpuiApplication(() => new GalleryView(application!, initialPage));
application.UseCustomTitlebar = true;
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
            .Add(ui.Div(_pages[_index].Render(ref ui)).P(24).WFull());

        return ui.HStack(ui.Div(sidebar).FlexShrink(0), content)
            .Gap(16)
            .P(16)
            .Full();
    }
}
