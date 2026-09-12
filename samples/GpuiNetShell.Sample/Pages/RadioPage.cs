using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class RadioPage : GalleryPage
{
    private int _index = 1;

    public override string Title => "Radio";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Radio",
            "Controlled radio options; selection is reported as a checked value.",
            ui.HStack(
                    ui.Radio("light")
                        .Label("Light")
                        .Checked(_index == 0)
                        .OnChange(_ =>
                        {
                            _index = 0;
                            Invalidate();
                        }),
                    ui.Radio("dark")
                        .Label("Dark")
                        .Checked(_index == 1)
                        .OnChange(_ =>
                        {
                            _index = 1;
                            Invalidate();
                        }),
                    ui.Radio("system")
                        .Label("System")
                        .Checked(_index == 2)
                        .OnChange(_ =>
                        {
                            _index = 2;
                            Invalidate();
                        })
                )
                .Gap(16)
                .ItemsCenter(),
            ui.Label($"Selected option: {_index}")
        );
}
