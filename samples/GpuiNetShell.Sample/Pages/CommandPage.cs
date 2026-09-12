using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class CommandPage : GalleryPage
{
    private string _query = "";
    private string _selected = "";

    public override string Title => "Command";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Command",
            "A retained command palette reporting the query and the selected path.",
            ui.Command("palette")
                .Searchable()
                .Filterable()
                .Bordered()
                .Placeholder("Type a command…")
                .MaxHeight(320)
                .OnQuery(query =>
                {
                    _query = query;
                    Invalidate();
                })
                .OnSelect(path =>
                {
                    _selected = path;
                    Invalidate();
                })
                .Add(
                    ui.CommandGroup("General")
                        .Add(
                            ui.CommandItem("New file").Keyword("create new"),
                            ui.CommandItem("Open file").Keyword("open"),
                            ui.CommandItem("Save").Checked()
                        ),
                    ui.CommandSeparator(),
                    ui.CommandGroup("Danger")
                        .Add(ui.CommandItem("Delete").Disabled())
                ),
            ui.Label($"Query: {_query}   Selected: {_selected}")
        );
}
