using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class MenuBarPage : GalleryPage
{
    private string _last = "(none)";

    public override string Title => "MenuBar";

    public override Element Render(ref RenderContext ui) =>
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
            ui.Label($"Last command: {_last}")
        );

    private void Choose(string command)
    {
        _last = command;
        Invalidate();
    }
}
