using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class PaginationPage : GalleryPage
{
    private int _page = 1;

    public override string Title => "Pagination";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Pagination",
            "Controlled page navigation; selection is reported through a callback.",
            ui.Pagination("pages")
                .TotalPages(10)
                .CurrentPage(_page)
                .VisiblePages(5)
                .OnChange(page =>
                {
                    _page = page;
                    Invalidate();
                }),
            ui.Pagination("compact").TotalPages(4).Compact(),
            ui.Label($"Current page: {_page}")
        );
}
