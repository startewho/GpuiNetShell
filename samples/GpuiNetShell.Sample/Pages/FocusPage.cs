using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class FocusPage : GalleryPage
{
    private string _status = "(none)";

    public override string Title => "Focus";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Focus",
            "The retained inputs report focus and blur; tab between them to see it.",
            ui.Input("focus-input")
                .Placeholder("Click or Tab to me")
                .OnFocus(() => Set("Input gained focus"))
                .OnBlur(() => Set("Input lost focus")),
            ui.Textarea("focus-area")
                .Placeholder("Then Tab to me")
                .H(96)
                .OnFocus(() => Set("Textarea gained focus"))
                .OnBlur(() => Set("Textarea lost focus")),
            ui.Label($"Last focus event: {_status}")
        );

    private void Set(string status)
    {
        _status = status;
        Invalidate();
    }
}
