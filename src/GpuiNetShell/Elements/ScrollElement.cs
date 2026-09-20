using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A scrollable area. A <see cref="ScrollbarElement"/> that names the same id
/// drives its position.
/// </summary>
public sealed class ScrollElement : Element
{
    internal ScrollElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Selects the native scroll axes.</summary>
    public ScrollElement Axis(ScrollAxis axis)
    {
        Arena.AddMethodEnum(Index, "scroll_axis", AxisName(axis));
        return this;
    }

    /// <summary>
    /// Nudges the scroll position horizontally: a positive amount reveals later
    /// content, a negative amount reveals earlier content. Emit it only on the
    /// frame a nudge is requested; the offset persists on the retained handle.
    /// </summary>
    public ScrollElement ScrollBy(int pixels)
    {
        Arena.AddMethodNumber(Index, "scroll_by", pixels);
        return this;
    }

    public ScrollElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }

    internal static string AxisName(ScrollAxis axis) =>
        axis switch
        {
            ScrollAxis.Horizontal => "horizontal",
            ScrollAxis.Both => "both",
            _ => "vertical",
        };
}
