using GpuiNetShell.Events;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A plain container. Styling is the shared surface in <see cref="StyleExtensions"/>.</summary>
/// <remarks>
/// A Div can subscribe to GPUI element events. Only the subscribed events cross
/// the ABI; the native host binds a listener only for those, so an unsubscribed
/// event costs nothing. Events GPUI keys by element id (click, aux-click, hover)
/// require a stable <c>id</c>: it must be unique within the window and unchanged
/// for as long as the element is rendered.
/// </remarks>
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

    /// <summary>
    /// Blocks the mouse from elements painted behind this Div. On Windows the
    /// title bar is a single window-drag region; a control that does not occlude
    /// is treated as part of the caption and never receives a click. Occluding a
    /// control leaves the bar's gaps draggable.
    /// </summary>
    public DivElement Occlude()
    {
        Arena.AddMethod(Index, "occlude");
        return this;
    }

    /// <summary>
    /// Binds left-click activation (press and release). <paramref name="id"/> is
    /// a stable identity that GPUI keys the click's press/release state by, so a
    /// per-frame value would never complete a click. The handler runs on the
    /// native application thread; the host requests a re-render after it returns.
    /// </summary>
    public DivElement OnClick(string id, Action handler)
    {
        ArgumentException.ThrowIfNullOrEmpty(id);
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddMethodString(Index, "element_id", id);
        Arena.AddCallback(Index, "on_click", Events.Register(handler));
        return this;
    }

    /// <summary>Binds a non-primary click. <paramref name="id"/> must be stable.</summary>
    public DivElement OnAuxClick(string id, Action<ClickEvent> handler) =>
        Event("on_aux_click", id, value => handler(ClickEvent.Decode(value.String)));

    /// <summary>Binds hover start/end; the argument is true on enter. <paramref name="id"/> must be stable.</summary>
    public DivElement OnHover(string id, Action<bool> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        return Event("on_hover", id, value => handler(value.Boolean));
    }

    /// <summary>Binds a mouse-down on any button.</summary>
    public DivElement OnMouseDown(Action<PointerEvent> handler) =>
        Event("on_mouse_down", null, value => handler(PointerEvent.Decode(value.String)));

    /// <summary>Binds a mouse-up.</summary>
    public DivElement OnMouseUp(Action<PointerEvent> handler) =>
        Event("on_mouse_up", null, value => handler(PointerEvent.Decode(value.String)));

    /// <summary>Binds a mouse-down outside this element.</summary>
    public DivElement OnMouseDownOut(Action<PointerEvent> handler) =>
        Event("on_mouse_down_out", null, value => handler(PointerEvent.Decode(value.String)));

    /// <summary>Binds a mouse-up outside this element.</summary>
    public DivElement OnMouseUpOut(Action<PointerEvent> handler) =>
        Event("on_mouse_up_out", null, value => handler(PointerEvent.Decode(value.String)));

    /// <summary>Binds mouse movement over this element. Not auto-repainted; call <c>Notify</c> if state changes.</summary>
    public DivElement OnMouseMove(Action<PointerEvent> handler) =>
        Event("on_mouse_move", null, value => handler(PointerEvent.Decode(value.String)));

    /// <summary>Binds force-touch pressure changes.</summary>
    public DivElement OnMousePressure(Action<PressureEvent> handler) =>
        Event("on_mouse_pressure", null, value => handler(PressureEvent.Decode(value.String)));

    /// <summary>Binds scroll-wheel input. Not auto-repainted; call <c>Notify</c> if state changes.</summary>
    public DivElement OnScroll(Action<ScrollEvent> handler) =>
        Event("on_scroll_wheel", null, value => handler(ScrollEvent.Decode(value.String)));

    /// <summary>Binds a key-down on the focused element.</summary>
    public DivElement OnKeyDown(Action<KeyEvent> handler) =>
        Event("on_key_down", null, value => handler(KeyEvent.Decode(value.String)));

    /// <summary>Binds a key-up on the focused element.</summary>
    public DivElement OnKeyUp(Action<KeyEvent> handler) =>
        Event("on_key_up", null, value => handler(KeyEvent.Decode(value.String)));

    /// <summary>
    /// Subscribes to an element event with the raw <see cref="EventValue"/>.
    /// <paramref name="id"/> is required for click, aux-click, and hover.
    /// </summary>
    public DivElement On(string? id, DivEvent evt, Action<EventValue> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        return Event(WireName(evt), id, handler);
    }

    private DivElement Event(string name, string? id, Action<EventValue> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        if (!string.IsNullOrEmpty(id))
        {
            Arena.AddMethodString(Index, "element_id", id);
        }
        Arena.AddCallback(Index, name, Events.Register(handler));
        return this;
    }

    private static string WireName(DivEvent evt) =>
        evt switch
        {
            DivEvent.Click => "on_click",
            DivEvent.AuxClick => "on_aux_click",
            DivEvent.Hover => "on_hover",
            DivEvent.MouseDown => "on_mouse_down",
            DivEvent.MouseUp => "on_mouse_up",
            DivEvent.MouseDownOut => "on_mouse_down_out",
            DivEvent.MouseUpOut => "on_mouse_up_out",
            DivEvent.MouseMove => "on_mouse_move",
            DivEvent.MousePressure => "on_mouse_pressure",
            DivEvent.ScrollWheel => "on_scroll_wheel",
            DivEvent.KeyDown => "on_key_down",
            DivEvent.KeyUp => "on_key_up",
            _ => throw new ArgumentOutOfRangeException(nameof(evt)),
        };
}
