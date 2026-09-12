using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A typed accordion accepting only <see cref="AccordionItemElement"/> children.</summary>
public sealed class AccordionElement : Element
{
    internal AccordionElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Allows multiple items to remain open.</summary>
    public AccordionElement Multiple(bool multiple = true)
    {
        Arena.AddMethodNumber(Index, "multiple", multiple ? 1 : 0);
        return this;
    }

    /// <summary>Controls the joined outer border.</summary>
    public AccordionElement Bordered(bool bordered = true)
    {
        Arena.AddMethodNumber(Index, "bordered", bordered ? 1 : 0);
        return this;
    }

    /// <summary>Sets the semantic component size.</summary>
    public AccordionElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    public AccordionElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Adds the item children.</summary>
    public AccordionElement Add(params AccordionItemElement[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}
