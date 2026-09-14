using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class MenuBarPage : GalleryPage<MenuBarPage.State>
{
    internal sealed class State
    {
        public string Last { get; set; } = "(none)";
    }

    public override string Title => "MenuBar";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "MenuBar",
            "An in-window menu bar built from typed menu items.",
            ui.MenuBar("main")
                .Add(
                    ui.Menu("File")
                        .Add(
                            ui.MenuItem("New").OnSelect(() => Choose("New")),
                            ui.MenuItem("Open").OnSelect(() => Choose("Open")),
                            ui.MenuSeparator(),
                            ui.MenuItem("Quit").Disabled()
                        ),
                    ui.Menu("Edit")
                        .Add(
                            ui.MenuItem("Undo").Checked().OnSelect(() => Choose("Undo")),
                            ui.MenuItem("Redo").OnSelect(() => Choose("Redo"))
                        )
                ),
            ui.Label($"Last command: {state.Last}")
        );

    private void Choose(string command)
    {
        Update((s, c) => { s.Last = command; c.Notify(); });
    }
}
