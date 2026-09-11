using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A plain container. Styling is the shared surface in <see cref="StyleExtensions"/>.</summary>
public sealed class DivElement : Element
{
    internal DivElement(RenderContext ui, int index)
        : base(ui, index) { }

    public DivElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }

    public DivElement Children(IEnumerable<Element> children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
