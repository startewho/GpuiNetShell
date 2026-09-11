using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A horizontal or vertical, solid or dashed separator. Orientation and dash are
/// chosen by the factory method on <c>RenderContext</c>.
/// </summary>
public sealed class SeparatorElement : Element
{
    internal SeparatorElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Displays text centered over the separator line.</summary>
    public SeparatorElement Label(string label)
    {
        Arena.AddMethodString(Index, "label", label);
        return this;
    }

    /// <summary>Sets the separator line color.</summary>
    public SeparatorElement Color(string color)
    {
        Arena.AddMethodString(Index, "color", color);
        return this;
    }

    /// <summary>Uses a dashed separator line.</summary>
    public SeparatorElement Dashed()
    {
        Arena.AddMethod(Index, "dashed");
        return this;
    }
}
