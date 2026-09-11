using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>An animated loading placeholder.</summary>
public sealed class SkeletonElement : Element
{
    internal SkeletonElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Uses the secondary skeleton color.</summary>
    public SkeletonElement Secondary()
    {
        Arena.AddMethod(Index, "secondary");
        return this;
    }
}
