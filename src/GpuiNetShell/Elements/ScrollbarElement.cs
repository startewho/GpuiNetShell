using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A bar that drives the <see cref="ScrollElement"/> sharing its id.</summary>
public sealed class ScrollbarElement : Element
{
    internal ScrollbarElement(RenderContext ui, int index)
        : base(ui, index) { }

    public ScrollbarElement Axis(ScrollAxis axis)
    {
        Arena.AddMethodNumber(Index, "axis", (double)(int)axis);
        return this;
    }
}
