using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A styled text label. Styling is the shared surface in <see cref="StyleExtensions"/>.</summary>
public sealed class LabelElement : Element
{
    internal LabelElement(RenderContext ui, int index)
        : base(ui, index) { }
}
