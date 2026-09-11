using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A retained single-value select backed by a row snapshot. Rows and the
/// selection handler come from the <c>Select</c> factory on <c>RenderContext</c>.
/// </summary>
public sealed class SelectElement : Element
{
    internal SelectElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the text shown while nothing is selected.</summary>
    public SelectElement Placeholder(string placeholder)
    {
        Arena.AddMethodString(Index, "placeholder", placeholder);
        return this;
    }

    /// <summary>Sets the popup menu width in pixels.</summary>
    public SelectElement MenuWidth(double pixels)
    {
        Arena.AddMethodNumber(Index, "menu_width", pixels);
        return this;
    }

    public SelectElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }
}
