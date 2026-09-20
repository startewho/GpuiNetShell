using GpuiNetShell.Elements;
using GpuiNetShell.Events;
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
    private readonly List<Action<InputEvent>> _inputHandlers = [];

    internal void AttachInvalidator(Action invalidate) => _invalidate = invalidate;

    /// <summary>
    /// Requests a re-render from any thread. This is the managed equivalent of
    /// shell's <c>cx.notify()</c>: the native view marks itself dirty and
    /// repaints, replaying this view's <see cref="Render"/>.
    /// </summary>
    public void Invalidate() => _invalidate?.Invoke();

    /// <summary>
    /// Registers a handler for window input events (mouse, wheel, and keyboard).
    /// Handlers run on the native application thread; call <see cref="Invalidate"/>
    /// after changing state.
    /// </summary>
    public void OnInput(Action<InputEvent> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        _inputHandlers.Add(handler);
    }

    internal void DispatchInput(InputEvent input)
    {
        foreach (var handler in _inputHandlers)
        {
            handler(input);
        }
    }

    /// <summary>Describes this view's element tree for the current state.</summary>
    protected abstract Element Render(ref RenderContext ui);

    /// <summary>
    /// Describes the custom title bar content, or <see langword="null"/> to use
    /// the default title bar. Only called when the window was opened with a
    /// custom title bar; the native host still draws the window controls and
    /// handles dragging around the returned element.
    /// </summary>
    protected virtual Element? RenderTitleBar(ref RenderContext ui) => null;

    internal Element RenderRoot(ref RenderContext ui) => Render(ref ui);

    internal Element? RenderTitleBarRoot(ref RenderContext ui) =>
        RenderTitleBar(ref ui);
}
