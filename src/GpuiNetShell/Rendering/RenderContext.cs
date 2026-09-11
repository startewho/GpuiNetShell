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

    /// <summary>Declares a count or dot badge configured through methods.</summary>
    public BadgeElement Badge() =>
        new BadgeElement(this, _arena.AddNode(NativeProtocol.ComponentBadge));

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

    /// <summary>Declares a radio option. <paramref name="id"/> is its stable identity.</summary>
    public RadioElement Radio(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentRadio);
        _arena.SetNodeData(index, id);
        return new RadioElement(this, index);
    }

    /// <summary>Declares a tab bar. <paramref name="id"/> is its stable identity.</summary>
    public TabsElement Tabs(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentTabs);
        _arena.SetNodeData(index, id);
        return new TabsElement(this, index);
    }

    /// <summary>Declares a scrollable area. <paramref name="id"/> is its stable identity.</summary>
    public ScrollElement Scroll(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentScroll);
        _arena.SetNodeData(index, id);
        return new ScrollElement(this, index);
    }

    /// <summary>Declares a scrollbar that drives the area named <paramref name="target"/>.</summary>
    public ScrollbarElement Scrollbar(string target)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentScrollbar);
        _arena.SetNodeData(index, target);
        return new ScrollbarElement(this, index);
    }

    /// <summary>Declares resizable panels. <paramref name="id"/> is its stable identity.</summary>
    public ResizableElement Resizable(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentResizable);
        _arena.SetNodeData(index, id);
        return new ResizableElement(this, index);
    }

    /// <summary>
    /// Declares an anchored popover. <paramref name="id"/> is its stable
    /// identity; <paramref name="label"/> is the trigger's label.
    /// </summary>
    public PopoverElement Popover(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentPopover);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + label);
        return new PopoverElement(this, index);
    }

    /// <summary>Declares a cycling loading spinner.</summary>
    public SpinnerElement Spinner() =>
        new SpinnerElement(this, _arena.AddNode(NativeProtocol.ComponentSpinner));

    /// <summary>Declares a horizontal separator.</summary>
    public SeparatorElement Separator() => Separator("Separator");

    /// <summary>Declares a vertical separator.</summary>
    public SeparatorElement VerticalSeparator() => Separator("VerticalSeparator");

    /// <summary>Declares a dashed horizontal separator.</summary>
    public SeparatorElement DashedSeparator() => Separator("DashedSeparator");

    /// <summary>Declares a dashed vertical separator.</summary>
    public SeparatorElement VerticalDashedSeparator() => Separator("VerticalDashedSeparator");

    /// <summary>Declares an animated loading placeholder.</summary>
    public SkeletonElement Skeleton() =>
        new SkeletonElement(this, _arena.AddNode(NativeProtocol.ComponentSkeleton));

    /// <summary>Declares a compact semantic status tag.</summary>
    public TagElement Tag() =>
        new TagElement(this, _arena.AddNode(NativeProtocol.ComponentTag));

    private SeparatorElement Separator(string export)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSeparator);
        _arena.SetNodeData(index, export);
        return new SeparatorElement(this, index);
    }

    /// <summary>Declares an external-resource link. <paramref name="id"/> is its identity.</summary>
    public LinkElement Link(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentLink);
        _arena.SetNodeData(index, id);
        return new LinkElement(this, index);
    }

    /// <summary>Declares a keyboard shortcut keycap from a keystroke string.</summary>
    public KbdElement Kbd(string keystroke)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentKbd);
        _arena.SetNodeData(index, keystroke);
        return new KbdElement(this, index);
    }

    /// <summary>Declares a circular avatar with a name-derived fallback.</summary>
    public AvatarElement Avatar() =>
        new AvatarElement(this, _arena.AddNode(NativeProtocol.ComponentAvatar));

    /// <summary>Declares an SVG icon from a relative asset path.</summary>
    public IconElement Icon(string path)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentIcon);
        _arena.SetNodeData(index, path);
        return new IconElement(this, index);
    }

    /// <summary>A column container.</summary>
    public DivElement VStack(params Element[] children) => Div(children).FlexColumn();

    /// <summary>A row container.</summary>
    public DivElement HStack(params Element[] children) => Div(children).FlexRow();
}
