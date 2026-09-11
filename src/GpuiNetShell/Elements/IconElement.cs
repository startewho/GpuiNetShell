using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// An SVG icon loaded from a relative path beneath the application's asset root.
/// </summary>
public sealed class IconElement : Element
{
    internal IconElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the semantic icon size.</summary>
    public IconElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Sets the icon color from a supported color token.</summary>
    public IconElement Color(string color)
    {
        Arena.AddMethodString(Index, "color", color);
        return this;
    }

    /// <summary>Rotates the icon by a finite number of radians.</summary>
    public IconElement Rotate(double radians)
    {
        Arena.AddMethodNumber(Index, "rotate", radians);
        return this;
    }
}
