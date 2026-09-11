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

    public ScrollElement Axis(ScrollAxis axis)
    {
        Arena.AddMethodNumber(Index, "axis", (double)(int)axis);
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
}
