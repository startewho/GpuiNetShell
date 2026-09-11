using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A bar that drives the <see cref="ScrollElement"/> sharing its id.</summary>
public sealed class ScrollbarElement : Element
{
    internal ScrollbarElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Selects the native scroll axes.</summary>
    public ScrollbarElement Axis(ScrollAxis axis)
    {
        Arena.AddMethodEnum(Index, "scroll_axis", ScrollElement.AxisName(axis));
        return this;
    }

    /// <summary>Sets the native scrollbar visibility policy.</summary>
    public ScrollbarElement Mode(ScrollbarMode mode)
    {
        Arena.AddMethodEnum(Index, "mode", ModeName(mode));
        return this;
    }

    /// <summary>Uses this element's layout bounds as the native viewport.</summary>
    public ScrollbarElement ViewportFromLayout(bool enabled = true)
    {
        Arena.AddMethodNumber(Index, "viewport_from_layout", enabled ? 1 : 0);
        return this;
    }

    private static string ModeName(ScrollbarMode mode) =>
        mode switch
        {
            ScrollbarMode.Hover => "hover",
            ScrollbarMode.Always => "always",
            _ => "scrolling",
        };
}
