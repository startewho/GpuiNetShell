using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>Typed native Command palette item data.</summary>
public sealed class CommandItemElement : Element
{
    internal CommandItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the item search keyword.</summary>
    public CommandItemElement Keyword(string keyword)
    {
        Arena.AddMethodString(Index, "keyword", keyword);
        return this;
    }

    /// <summary>Sets the item checked state.</summary>
    public CommandItemElement Checked(bool checkedValue = true)
    {
        Arena.AddMethodNumber(Index, "checked", checkedValue ? 1 : 0);
        return this;
    }

    public CommandItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Sets a lazy content element for the row.</summary>
    public CommandItemElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }
}

/// <summary>Typed native Command palette group data.</summary>
public sealed class CommandGroupElement : Element
{
    internal CommandGroupElement(RenderContext ui, int index)
        : base(ui, index) { }

    public CommandGroupElement Add(params CommandItemElement[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}

/// <summary>Typed Command separator data.</summary>
public sealed class CommandSeparatorElement : Element
{
    internal CommandSeparatorElement(RenderContext ui, int index)
        : base(ui, index) { }
}

/// <summary>A retained native Command palette.</summary>
public sealed class CommandElement : Element
{
    internal CommandElement(RenderContext ui, int index)
        : base(ui, index) { }

    public CommandElement Searchable(bool value = true)
    {
        Arena.AddMethodNumber(Index, "searchable", value ? 1 : 0);
        return this;
    }

    public CommandElement Filterable(bool value = true)
    {
        Arena.AddMethodNumber(Index, "filterable", value ? 1 : 0);
        return this;
    }

    public CommandElement Bordered(bool value = true)
    {
        Arena.AddMethodNumber(Index, "bordered", value ? 1 : 0);
        return this;
    }

    /// <summary>Sets the command search placeholder.</summary>
    public CommandElement Placeholder(string placeholder)
    {
        Arena.AddMethodString(Index, "placeholder", placeholder);
        return this;
    }

    /// <summary>Sets the command results maximum height.</summary>
    public CommandElement MaxHeight(double pixels)
    {
        Arena.AddMethodNumber(Index, "max_height", pixels);
        return this;
    }

    /// <summary>Reports the current query.</summary>
    public CommandElement OnQuery(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_query", token);
        return this;
    }

    /// <summary>Reports the selected path as a <c>"section,row"</c> string.</summary>
    public CommandElement OnSelect(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_select", token);
        return this;
    }

    public CommandElement OnCancel(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(handler);
        Arena.AddCallback(Index, "on_cancel", token);
        return this;
    }

    /// <summary>Sets a lazy header element.</summary>
    public CommandElement Header(Element header)
    {
        ArgumentNullException.ThrowIfNull(header);
        Arena.AddSlot(Index, "header", header.Index);
        return this;
    }

    /// <summary>Sets a lazy footer element.</summary>
    public CommandElement Footer(Element footer)
    {
        ArgumentNullException.ThrowIfNull(footer);
        Arena.AddSlot(Index, "footer", footer.Index);
        return this;
    }

    public CommandElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
