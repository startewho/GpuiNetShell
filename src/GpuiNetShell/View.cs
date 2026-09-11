using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell;

/// <summary>
/// An application view. <see cref="Render"/> describes the current element tree
/// into the arena and must not perform I/O or start work; observable state
/// changes belong in event handlers.
/// </summary>
public abstract class View
{
    /// <summary>Describes this view's element tree for the current state.</summary>
    protected abstract Element Render(ref RenderContext ui);

    internal Element RenderRoot(ref RenderContext ui) => Render(ref ui);
}
