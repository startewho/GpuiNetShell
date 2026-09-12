using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TextareaPage : GalleryPage
{
    private string _value = "";

    public override string Title => "Textarea";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Textarea",
            "A retained multi-line text field.",
            ui.Textarea("notes")
                .Placeholder("Write a note…")
                .OnChange(value =>
                {
                    _value = value;
                    Invalidate();
                }),
            ui.Label($"Length: {_value.Length} characters")
        );
}
