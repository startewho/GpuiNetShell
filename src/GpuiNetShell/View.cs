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
    private Action? _invalidate;

    internal void AttachInvalidator(Action invalidate) => _invalidate = invalidate;

    /// <summary>
    /// Requests a re-render from any thread. This is the managed equivalent of
    /// shell's <c>cx.notify()</c>: the native view marks itself dirty and
    /// repaints, replaying this view's <see cref="Render"/>.
    /// </summary>
    public void Invalidate() => _invalidate?.Invoke();

    /// <summary>Describes this view's element tree for the current state.</summary>
    protected abstract Element Render(ref RenderContext ui);

    internal Element RenderRoot(ref RenderContext ui) => Render(ref ui);
}
