using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class RatingPage : GalleryPage
{
    private int _rating = 3;

    public override string Title => "Rating";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Rating",
            "Interactive star ratings; the clicked value is reported as a number.",
            ui.Rating("quality").Max(5).Value(_rating).OnChange(value =>
            {
                _rating = value;
                Invalidate();
            }),
            ui.Rating("color").Max(5).Value(4).Color("orange-500"),
            ui.Rating("small").Max(5).Value(2).Size(ControlSize.Small),
            ui.Label($"Rating: {_rating}/5")
        );
}
