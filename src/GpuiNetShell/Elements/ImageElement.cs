using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// An image or SVG loaded from the asset source by path (a plain filesystem
/// path or a bundled asset path). Size and radius come from style.
/// </summary>
public sealed class ImageElement : Element
{
    internal ImageElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the object-fit: cover, contain, fill, none, or scale_down.</summary>
    public ImageElement Fit(string fit)
    {
        Arena.AddMethodEnum(Index, "fit", fit);
        return this;
    }
}
