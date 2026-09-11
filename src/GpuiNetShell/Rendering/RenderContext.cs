using System.Globalization;
using GpuiNetShell.Elements;
using GpuiNetShell.Events;
using GpuiNetShell.Interop;

namespace GpuiNetShell.Rendering;

/// <summary>
/// The element declaration surface passed to <see cref="View.Render"/>. Every
/// factory records into the arena; nothing renders here.
/// </summary>
public sealed class RenderContext
{
    private readonly RenderArena _arena;
    private readonly EventRegistry _events;
    private readonly Action _notify;
    private bool _rendering;

    internal RenderContext(RenderArena arena, EventRegistry events, Action notify)
    {
        _arena = arena;
        _events = events;
        _notify = notify;
    }

    internal RenderArena Arena => _arena;

    internal EventRegistry Events => _events;

    /// <summary>
    /// Requests a native re-render, the managed equivalent of gpui's
    /// <c>cx.notify()</c>. Call it from an event or task after changing state
    /// that <see cref="View.Render"/> reads.
    /// </summary>
    /// <exception cref="InvalidOperationException">
    /// Thrown when called during <see cref="View.Render"/>, where requesting
    /// another render would loop, matching gpui's own rule.
    /// </exception>
    public void Notify()
    {
        if (_rendering)
        {
            throw new InvalidOperationException(
                "Cannot notify during Render(); change state from an event or task instead."
            );
        }
        _notify();
    }

    /// <summary>Marks that managed rendering has begun, so <see cref="Notify"/> is refused.</summary>
    internal void BeginRender() => _rendering = true;

    /// <summary>Marks that managed rendering has ended.</summary>
    internal void EndRender() => _rendering = false;

    /// <summary>Declares a button. <paramref name="id"/> is its stable identity.</summary>
    public ButtonElement Button(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentButton);
        _arena.SetNodeData(index, id);
        return new ButtonElement(this, index);
    }

    /// <summary>Declares a run of text.</summary>
    public TextElement Text(string content)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentText);
        _arena.SetNodeData(index, content);
        return new TextElement(this, index);
    }

    /// <summary>Declares a container with the given children.</summary>
    public DivElement Div(params Element[] children) =>
        new DivElement(this, _arena.AddNode(NativeProtocol.ComponentDiv)).Add(children);

    /// <summary>Declares a styled label.</summary>
    public LabelElement Label(string value)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentLabel);
        _arena.SetNodeData(index, value);
        return new LabelElement(this, index);
    }

    /// <summary>Declares a count badge.</summary>
    public BadgeElement Badge(int count)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentBadge);
        _arena.SetNodeData(index, count.ToString(CultureInfo.InvariantCulture));
        return new BadgeElement(this, index);
    }

    /// <summary>Declares a progress bar. <paramref name="id"/> is its stable identity.</summary>
    public ProgressElement Progress(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentProgress);
        _arena.SetNodeData(index, id);
        return new ProgressElement(this, index);
    }

    /// <summary>Declares a combobox. <paramref name="id"/> is its stable identity.</summary>
    public ComboboxElement Combobox(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentCombobox);
        _arena.SetNodeData(index, id);
        return new ComboboxElement(this, index);
    }

    /// <summary>A column container.</summary>
    public DivElement VStack(params Element[] children) => Div(children).FlexColumn();

    /// <summary>A row container.</summary>
    public DivElement HStack(params Element[] children) => Div(children).FlexRow();
}
