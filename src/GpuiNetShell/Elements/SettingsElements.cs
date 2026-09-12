using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>Label/field axis of a <see cref="SettingItemElement"/>.</summary>
public enum SettingLayout : uint
{
    Horizontal = 0,
    Vertical = 1,
}

/// <summary>A typed setting item requiring a lazy <see cref="Content"/> element.</summary>
public sealed class SettingItemElement : Element
{
    internal SettingItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the supporting description.</summary>
    public SettingItemElement Description(string text)
    {
        Arena.AddMethodString(Index, "description", text);
        return this;
    }

    /// <summary>Lays the label and field out along the given axis.</summary>
    public SettingItemElement Layout(SettingLayout layout)
    {
        Arena.AddMethodEnum(Index, "layout", layout == SettingLayout.Vertical ? "vertical" : "horizontal");
        return this;
    }

    /// <summary>Adds search keywords that match this item.</summary>
    public SettingItemElement Keywords(params string[] keywords)
    {
        Arena.AddMethodString(Index, "keywords", string.Join('\n', keywords ?? []));
        return this;
    }

    /// <summary>Disables the item's field.</summary>
    public SettingItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Sets the lazy content element.</summary>
    public SettingItemElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }
}

/// <summary>A styled setting group accepting <see cref="SettingItemElement"/> children.</summary>
public sealed class SettingGroupElement : Element
{
    internal SettingGroupElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SettingGroupElement Title(string title)
    {
        Arena.AddMethodString(Index, "title", title);
        return this;
    }

    public SettingGroupElement Description(string text)
    {
        Arena.AddMethodString(Index, "description", text);
        return this;
    }

    public SettingGroupElement Add(params SettingItemElement[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}

/// <summary>A typed setting page accepting <see cref="SettingGroupElement"/> children.</summary>
public sealed class SettingPageElement : Element
{
    internal SettingPageElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SettingPageElement Description(string text)
    {
        Arena.AddMethodString(Index, "description", text);
        return this;
    }

    public SettingPageElement DefaultOpen(bool open = true)
    {
        Arena.AddMethodNumber(Index, "default_open", open ? 1 : 0);
        return this;
    }

    public SettingPageElement Resettable(bool resettable = true)
    {
        Arena.AddMethodNumber(Index, "resettable", resettable ? 1 : 0);
        return this;
    }

    /// <summary>Sets a lazy title-suffix element.</summary>
    public SettingPageElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }

    public SettingPageElement Add(params SettingGroupElement[] groups)
    {
        ArgumentNullException.ThrowIfNull(groups);
        foreach (var group in groups)
        {
            Arena.AddChild(Index, group.Index);
        }
        return this;
    }
}

/// <summary>A settings container accepting <see cref="SettingPageElement"/> children.</summary>
public sealed class SettingsElement : Element
{
    internal SettingsElement(RenderContext ui, int index)
        : base(ui, index) { }

    public SettingsElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Sets the sidebar's width in pixels.</summary>
    public SettingsElement SidebarWidth(double pixels)
    {
        Arena.AddMethodNumber(Index, "sidebar_width", pixels);
        return this;
    }

    /// <summary>Selects the page shown when the surface first opens.</summary>
    public SettingsElement DefaultSelectedPage(int index)
    {
        Arena.AddMethodNumber(Index, "default_selected_page", index);
        return this;
    }

    public SettingsElement Add(params SettingPageElement[] pages)
    {
        ArgumentNullException.ThrowIfNull(pages);
        foreach (var page in pages)
        {
            Arena.AddChild(Index, page.Index);
        }
        return this;
    }
}
