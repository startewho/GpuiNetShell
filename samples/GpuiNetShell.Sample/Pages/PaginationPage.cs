using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class PaginationPage : GalleryPage<PaginationPage.State>
{
    internal sealed class State
    {
        public int Page { get; set; } = 1;
    }

    public override string Title => "Pagination";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Pagination",
            "Controlled page navigation; selection is reported through a callback.",
            ui.Pagination("pages")
                .TotalPages(10)
                .CurrentPage(state.Page)
                .VisiblePages(5)
                .OnChange(page =>
                {
                    Update((s, c) => { s.Page = page; c.Notify(); });
                }),
            ui.Pagination("compact").TotalPages(4).Compact(),
            ui.Label($"Current page: {state.Page}")
        );
}
