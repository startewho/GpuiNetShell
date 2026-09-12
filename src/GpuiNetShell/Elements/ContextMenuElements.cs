using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A typed context-menu item consumed by a <see cref="ContextMenuElement"/>.</summary>
public sealed class ContextMenuItemElement : Element
{
    internal ContextMenuItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    public ContextMenuItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Sets the menu item checked state.</summary>
    public ContextMenuItemElement Checked(bool checkedValue = true)
    {
        Arena.AddMethodNumber(Index, "checked", checkedValue ? 1 : 0);
        return this;
    }

    /// <summary>Runs when the context-menu item is selected.</summary>
    public ContextMenuItemElement OnSelect(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_select", Events.Register(handler));
        return this;
    }
}

/// <summary>A typed context-menu separator.</summary>
public sealed class ContextMenuSeparatorElement : Element
{
    internal ContextMenuSeparatorElement(RenderContext ui, int index)
        : base(ui, index) { }
}

/// <summary>Attaches a right-click menu to its target children.</summary>
public sealed class ContextMenuElement : Element
{
    internal ContextMenuElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the element the menu attaches to.</summary>
    public ContextMenuElement Target(Element target)
    {
        ArgumentNullException.ThrowIfNull(target);
        Arena.AddChild(Index, target.Index);
        return this;
    }

    /// <summary>Adds menu entries (<see cref="ContextMenuItemElement"/> / separators).</summary>
    public ContextMenuElement Items(params Element[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}
