using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// An accordion part accepted only as a direct <see cref="AccordionElement"/>
/// child. It carries its native value to the parent and renders nothing on its
/// own.
/// </summary>
public sealed class AccordionItemElement : Element
{
    internal AccordionItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the interactive title row element.</summary>
    public AccordionItemElement Title(Element title)
    {
        ArgumentNullException.ThrowIfNull(title);
        Arena.AddMethodElement(Index, "title", title.Index);
        return this;
    }

    /// <summary>Controls expanded state.</summary>
    public AccordionItemElement Open(bool open = true)
    {
        Arena.AddMethodNumber(Index, "open", open ? 1 : 0);
        return this;
    }

    /// <summary>Disables this item.</summary>
    public AccordionItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Adds the revealed content.</summary>
    public AccordionItemElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
