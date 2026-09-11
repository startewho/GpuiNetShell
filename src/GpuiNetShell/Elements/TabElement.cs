using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A tab accepted only as a direct <see cref="TabBarElement"/> child. It carries
/// its native value to the parent and renders nothing on its own.
/// </summary>
public sealed class TabElement : Element
{
    internal TabElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the visible tab label.</summary>
    public TabElement Label(string label)
    {
        Arena.AddMethodString(Index, "label", label);
        return this;
    }

    /// <summary>Sets the accessible tab label.</summary>
    public TabElement AriaLabel(string label)
    {
        Arena.AddMethodString(Index, "aria_label", label);
        return this;
    }

    public TabElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    public TabElement Selected(bool selected = true)
    {
        Arena.AddMethodNumber(Index, "selected", selected ? 1 : 0);
        return this;
    }
}
