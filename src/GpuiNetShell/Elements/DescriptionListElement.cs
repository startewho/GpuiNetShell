using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A structured label/value list accepting <see cref="DescriptionItemElement"/> children.</summary>
public sealed class DescriptionListElement : Element
{
    internal DescriptionListElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Uses the vertical label/value layout.</summary>
    public DescriptionListElement Vertical()
    {
        Arena.AddMethod(Index, "vertical");
        return this;
    }

    /// <summary>Controls the horizontal-layout border.</summary>
    public DescriptionListElement Bordered(bool bordered = true)
    {
        Arena.AddMethodNumber(Index, "bordered", bordered ? 1 : 0);
        return this;
    }

    /// <summary>Sets the column count from 1 through 10.</summary>
    public DescriptionListElement Columns(int columns)
    {
        Arena.AddMethodNumber(Index, "columns", columns);
        return this;
    }

    /// <summary>Sets the description-list density.</summary>
    public DescriptionListElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Adds the item children.</summary>
    public DescriptionListElement Add(params DescriptionItemElement[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}
