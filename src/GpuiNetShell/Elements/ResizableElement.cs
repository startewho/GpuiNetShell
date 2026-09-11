using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// Draggable panels in a row or column. Give the frame a width or height; sizes
/// are pixels with <c>*</c> for the flexible panel.
/// </summary>
public sealed class ResizableElement : Element
{
    internal ResizableElement(RenderContext ui, int index)
        : base(ui, index) { }

    public ResizableElement Axis(ResizeAxis axis)
    {
        Arena.AddMethodNumber(Index, "axis", (double)(int)axis);
        return this;
    }

    public ResizableElement Sizes(params string[] sizes)
    {
        Arena.AddMethodString(Index, "sizes", string.Join(',', sizes ?? []));
        return this;
    }

    public ResizableElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
