using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class RadioGroupPage : GalleryPage
{
    private int _index;

    public override string Title => "Radio Group";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Radio Group",
            "A controlled radio set; selection is reported as a zero-based index.",
            ui.RadioGroup("theme-group")
                .SelectedIndex(_index)
                .OnChange(index =>
                {
                    _index = index;
                    Invalidate();
                })
                .Add(
                    ui.Radio("group-light").Label("Light"),
                    ui.Radio("group-dark").Label("Dark"),
                    ui.Radio("group-system").Label("System")
                ),
            ui.Label($"Selected index: {_index}")
        );
}
