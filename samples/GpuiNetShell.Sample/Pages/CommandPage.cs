using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class CommandPage : GalleryPage<CommandPage.State>
{
    internal sealed class State
    {
        public string Query { get; set; } = "";
        public string Selected { get; set; } = "";
    }

    public override string Title => "Command";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
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
                    Update(
                        (s, c) =>
                        {
                            s.Query = query;
                            c.Notify();
                        }
                    )
                )
                .OnSelect(path =>
                    Update(
                        (s, c) =>
                        {
                            s.Selected = path;
                            c.Notify();
                        }
                    )
                )
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
            ui.Label($"Query: {state.Query}   Selected: {state.Selected}")
        );
}
