using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A vertical or horizontal form accepting <see cref="FieldElement"/> children.</summary>
public sealed class FormElement : Element
{
    internal FormElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the form grid's column count.</summary>
    public FormElement Columns(int columns)
    {
        Arena.AddMethodNumber(Index, "columns", columns);
        return this;
    }

    /// <summary>Sets the horizontal form label width in pixels.</summary>
    public FormElement LabelWidth(double pixels)
    {
        Arena.AddMethodNumber(Index, "label_width", pixels);
        return this;
    }

    /// <summary>Sets the form density.</summary>
    public FormElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Adds the field children.</summary>
    public FormElement Add(params FieldElement[] fields)
    {
        ArgumentNullException.ThrowIfNull(fields);
        foreach (var field in fields)
        {
            Arena.AddChild(Index, field.Index);
        }
        return this;
    }
}
