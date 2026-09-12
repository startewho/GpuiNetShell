using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>Shared plumbing for the button-triggered native window effects.</summary>
public abstract class WindowEffectElement : Element
{
    internal WindowEffectElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Registers an effect-error reporter.</summary>
    protected void RegisterEffectError(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_effect_error", token);
    }
}

/// <summary>A button that opens a native dialog with a custom content slot.</summary>
public sealed class DialogElement : WindowEffectElement
{
    internal DialogElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the dialog title.</summary>
    public DialogElement Title(string title)
    {
        Arena.AddMethodString(Index, "title", title);
        return this;
    }

    public DialogElement OnOk(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_ok", Events.Register(handler));
        return this;
    }

    public DialogElement OnCancel(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_cancel", Events.Register(handler));
        return this;
    }

    public DialogElement OnClose(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_close", Events.Register(handler));
        return this;
    }

    public DialogElement OnEffectError(Action<string> handler)
    {
        RegisterEffectError(handler);
        return this;
    }

    /// <summary>Sets the lazy dialog content element.</summary>
    public DialogElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }
}

/// <summary>A button that opens a native alert dialog.</summary>
public sealed class AlertDialogElement : WindowEffectElement
{
    internal AlertDialogElement(RenderContext ui, int index)
        : base(ui, index) { }

    public AlertDialogElement Title(string title)
    {
        Arena.AddMethodString(Index, "title", title);
        return this;
    }

    public AlertDialogElement Description(string description)
    {
        Arena.AddMethodString(Index, "description", description);
        return this;
    }

    public AlertDialogElement ShowCancel(bool showCancel = true)
    {
        Arena.AddMethodNumber(Index, "show_cancel", showCancel ? 1 : 0);
        return this;
    }

    public AlertDialogElement OnOk(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_ok", Events.Register(handler));
        return this;
    }

    public AlertDialogElement OnCancel(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_cancel", Events.Register(handler));
        return this;
    }

    public AlertDialogElement OnClose(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_close", Events.Register(handler));
        return this;
    }

    public AlertDialogElement OnEffectError(Action<string> handler)
    {
        RegisterEffectError(handler);
        return this;
    }
}

/// <summary>A button that opens a native sheet with a custom content slot.</summary>
public sealed class SheetElement : WindowEffectElement
{
    internal SheetElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SheetElement Title(string title)
    {
        Arena.AddMethodString(Index, "title", title);
        return this;
    }

    /// <summary>Sets the edge: top, right, bottom, or left.</summary>
    public SheetElement Placement(string placement)
    {
        Arena.AddMethodEnum(Index, "placement", placement);
        return this;
    }

    public SheetElement OnClose(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_close", Events.Register(handler));
        return this;
    }

    public SheetElement OnEffectError(Action<string> handler)
    {
        RegisterEffectError(handler);
        return this;
    }

    /// <summary>Sets the lazy sheet content element.</summary>
    public SheetElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }
}

/// <summary>A button that posts a native notification.</summary>
public sealed class NotificationElement : WindowEffectElement
{
    internal NotificationElement(RenderContext ui, int index)
        : base(ui, index) { }

    public NotificationElement Title(string title)
    {
        Arena.AddMethodString(Index, "title", title);
        return this;
    }

    public NotificationElement Message(string message)
    {
        Arena.AddMethodString(Index, "message", message);
        return this;
    }

    /// <summary>Sets the severity: info, success, warning, or error.</summary>
    public NotificationElement Type(string type)
    {
        Arena.AddMethodEnum(Index, "type", type);
        return this;
    }

    public NotificationElement Autohide(bool autohide = true)
    {
        Arena.AddMethodNumber(Index, "autohide", autohide ? 1 : 0);
        return this;
    }

    public NotificationElement OnClick(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_click", Events.Register(handler));
        return this;
    }

    public NotificationElement OnClose(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_close", Events.Register(handler));
        return this;
    }

    public NotificationElement OnEffectError(Action<string> handler)
    {
        RegisterEffectError(handler);
        return this;
    }
}
