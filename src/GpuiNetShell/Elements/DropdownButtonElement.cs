using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A real split dropdown button with a labeled action half and optional
/// callback menu items. It accepts no children.
/// </summary>
public sealed class DropdownButtonElement : Element
{
    internal DropdownButtonElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Uses the outlined button treatment.</summary>
    public DropdownButtonElement Outline()
    {
        Arena.AddMethod(Index, "outline");
        return this;
    }

    /// <summary>Sets the size of both halves.</summary>
    public DropdownButtonElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Sets the semantic variant of both halves.</summary>
    public DropdownButtonElement Variant(DropdownVariant variant)
    {
        Arena.AddMethodEnum(Index, "variant", variant switch
        {
            DropdownVariant.Secondary => "secondary",
            DropdownVariant.Danger => "danger",
            DropdownVariant.Ghost => "ghost",
            _ => "primary",
        });
        return this;
    }

    /// <summary>Sets the popup menu anchor.</summary>
    public DropdownButtonElement MenuAnchor(DropdownAnchor anchor)
    {
        Arena.AddMethodEnum(Index, "menu_anchor", anchor switch
        {
            DropdownAnchor.BottomRight => "bottom_right",
            DropdownAnchor.BottomLeft => "bottom_left",
            DropdownAnchor.TopLeft => "top_left",
            _ => "top_right",
        });
        return this;
    }

    /// <summary>Appends a clickable popup-menu item in call order.</summary>
    public DropdownButtonElement MenuItem(string label, Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(handler);
        Arena.AddMethodStringCallback(Index, "menu_item", label, token);
        return this;
    }

    public DropdownButtonElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    public DropdownButtonElement Selected(bool selected = true)
    {
        Arena.AddMethodNumber(Index, "selected", selected ? 1 : 0);
        return this;
    }

    /// <summary>Invokes the callback when the labeled action half is activated.</summary>
    public DropdownButtonElement OnClick(Action handler)
    {
        var token = Events.Register(handler);
        Arena.AddCallback(Index, "on_click", token);
        return this;
    }
}
