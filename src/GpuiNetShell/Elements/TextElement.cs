using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A run of text. Content is the constructor argument; styling is the
/// shared surface in <see cref="StyleExtensions"/>.</summary>
public sealed class TextElement : Element
{
    internal TextElement(RenderContext ui, int index)
        : base(ui, index) { }
}
