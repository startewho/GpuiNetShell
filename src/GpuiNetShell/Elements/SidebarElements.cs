using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A typed navigation row accepted by <see cref="SidebarMenuElement"/>.</summary>
public sealed class SidebarMenuItemElement : Element
{
    internal SidebarMenuItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the initial submenu disclosure state.</summary>
    public SidebarMenuItemElement DefaultOpen(bool open = true)
    {
        Arena.AddMethodNumber(Index, "default_open", open ? 1 : 0);
        return this;
    }

    /// <summary>Lets a row click open its submenu.</summary>
    public SidebarMenuItemElement ClickToOpen(bool value = true)
    {
        Arena.AddMethodNumber(Index, "click_to_open", value ? 1 : 0);
        return this;
    }

    /// <summary>Lets a row click toggle its submenu.</summary>
    public SidebarMenuItemElement ClickToToggle(bool value = true)
    {
        Arena.AddMethodNumber(Index, "click_to_toggle", value ? 1 : 0);
        return this;
    }

    /// <summary>Sets the navigation icon.</summary>
    public SidebarMenuItemElement Icon(SidebarIcon icon)
    {
        Arena.AddMethodEnum(Index, "icon", icon switch
        {
            SidebarIcon.Components => "components",
            SidebarIcon.Settings => "settings",
            SidebarIcon.Archive => "archive",
            SidebarIcon.Account => "account",
            _ => "home",
        });
        return this;
    }

    public SidebarMenuItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    public SidebarMenuItemElement Selected(bool selected = true)
    {
        Arena.AddMethodNumber(Index, "selected", selected ? 1 : 0);
        return this;
    }

    /// <summary>Invokes the callback when the control is activated.</summary>
    public SidebarMenuItemElement OnClick(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(handler);
        Arena.AddCallback(Index, "on_click", token);
        return this;
    }

    /// <summary>Adds nested submenu items.</summary>
    public SidebarMenuItemElement Add(params SidebarMenuItemElement[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}

/// <summary>A typed Sidebar menu accepting <see cref="SidebarMenuItemElement"/> children.</summary>
public sealed class SidebarMenuElement : Element
{
    internal SidebarMenuElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SidebarMenuElement Add(params Element[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}

/// <summary>A styled sidebar header accepting ordinary children.</summary>
public sealed class SidebarHeaderElement : Element
{
    internal SidebarHeaderElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SidebarHeaderElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A styled sidebar footer accepting ordinary children.</summary>
public sealed class SidebarFooterElement : Element
{
    internal SidebarFooterElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SidebarFooterElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A typed application sidebar accepting <see cref="SidebarMenuElement"/> children.</summary>
public sealed class SidebarElement : Element
{
    internal SidebarElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SidebarElement Side(SideKind side)
    {
        Arena.AddMethodEnum(Index, "side", side == SideKind.Right ? "right" : "left");
        return this;
    }

    /// <summary>Sets the sidebar collapse behavior.</summary>
    public SidebarElement Collapsible(SidebarCollapsibleKind mode)
    {
        Arena.AddMethodEnum(Index, "collapsible", mode switch
        {
            SidebarCollapsibleKind.Offcanvas => "offcanvas",
            SidebarCollapsibleKind.None => "none",
            _ => "icon",
        });
        return this;
    }

    /// <summary>Sets the controlled collapsed state.</summary>
    public SidebarElement Collapsed(bool collapsed = true)
    {
        Arena.AddMethodNumber(Index, "collapsed", collapsed ? 1 : 0);
        return this;
    }

    /// <summary>Sets the named header slot.</summary>
    public SidebarElement Header(Element header)
    {
        ArgumentNullException.ThrowIfNull(header);
        Arena.AddSlot(Index, "header", header.Index);
        return this;
    }

    /// <summary>Sets the named footer slot.</summary>
    public SidebarElement Footer(Element footer)
    {
        ArgumentNullException.ThrowIfNull(footer);
        Arena.AddSlot(Index, "footer", footer.Index);
        return this;
    }

    public SidebarElement Add(params SidebarMenuElement[] menus)
    {
        ArgumentNullException.ThrowIfNull(menus);
        foreach (var menu in menus)
        {
            Arena.AddChild(Index, menu.Index);
        }
        return this;
    }
}

/// <summary>A button that reflects sidebar side and collapsed state.</summary>
public sealed class SidebarToggleButtonElement : Element
{
    internal SidebarToggleButtonElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SidebarToggleButtonElement Side(SideKind side)
    {
        Arena.AddMethodEnum(Index, "side", side == SideKind.Right ? "right" : "left");
        return this;
    }

    public SidebarToggleButtonElement Collapsed(bool collapsed = true)
    {
        Arena.AddMethodNumber(Index, "collapsed", collapsed ? 1 : 0);
        return this;
    }

    /// <summary>Invokes the callback when activated.</summary>
    public SidebarToggleButtonElement OnClick(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(handler);
        Arena.AddCallback(Index, "on_click", token);
        return this;
    }
}
