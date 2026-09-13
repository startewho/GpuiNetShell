using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A retained managed entity subtree. The native host keeps one view per entity
/// id, so a <see cref="Entities.Context{T}.Notify"/> repaints only this subtree
/// rather than the whole window. Styling is the shared surface in
/// <see cref="StyleExtensions"/>.
/// </summary>
public sealed class EntityHostElement : Element
{
    internal EntityHostElement(RenderContext ui, int index)
        : base(ui, index) { }
}
