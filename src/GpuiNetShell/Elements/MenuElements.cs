using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A typed menu item accepted only by a <see cref="MenuElement"/>.</summary>
public sealed class MenuItemElement : Element
{
    internal MenuItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Disables the item.</summary>
    public MenuItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Shows a check mark.</summary>
    public MenuItemElement Checked(bool checkedValue = true)
    {
        Arena.AddMethodNumber(Index, "checked", checkedValue ? 1 : 0);
        return this;
    }

    /// <summary>Runs when the item is chosen.</summary>
    public MenuItemElement OnSelect(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(handler);
        Arena.AddCallback(Index, "on_select", token);
        return this;
    }
}

/// <summary>A typed separator accepted only by a <see cref="MenuElement"/>.</summary>
public sealed class MenuSeparatorElement : Element
{
    internal MenuSeparatorElement(RenderContext ui, int index)
        : base(ui, index) { }
}

/// <summary>A typed top-level menu accepted only by a <see cref="MenuBarElement"/>.</summary>
public sealed class MenuElement : Element
{
    internal MenuElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Disables the whole menu.</summary>
    public MenuElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Adds the menu entries.</summary>
    public MenuElement Add(params Element[] entries)
    {
        ArgumentNullException.ThrowIfNull(entries);
        foreach (var entry in entries)
        {
            Arena.AddChild(Index, entry.Index);
        }
        return this;
    }
}

/// <summary>A row of <see cref="MenuElement"/> dropdowns.</summary>
public sealed class MenuBarElement : Element
{
    internal MenuBarElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Adds the menus.</summary>
    public MenuBarElement Add(params MenuElement[] menus)
    {
        ArgumentNullException.ThrowIfNull(menus);
        foreach (var menu in menus)
        {
            Arena.AddChild(Index, menu.Index);
        }
        return this;
    }
}
