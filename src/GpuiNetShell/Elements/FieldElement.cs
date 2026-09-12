using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A typed form field containing ordinary control children. It carries its
/// native value to a <see cref="FormElement"/>.
/// </summary>
public sealed class FieldElement : Element
{
    internal FieldElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the field label.</summary>
    public FieldElement Label(string label)
    {
        Arena.AddMethodString(Index, "label", label);
        return this;
    }

    /// <summary>Sets supporting text below the control.</summary>
    public FieldElement Description(string description)
    {
        Arena.AddMethodString(Index, "description", description);
        return this;
    }

    /// <summary>Marks the field as required.</summary>
    public FieldElement Required(bool required = true)
    {
        Arena.AddMethodNumber(Index, "required", required ? 1 : 0);
        return this;
    }

    /// <summary>Controls field visibility.</summary>
    public FieldElement Visible(bool visible = true)
    {
        Arena.AddMethodNumber(Index, "visible", visible ? 1 : 0);
        return this;
    }

    /// <summary>Keeps unlabeled horizontal fields aligned with labeled fields.</summary>
    public FieldElement LabelIndent(bool indent = true)
    {
        Arena.AddMethodNumber(Index, "label_indent", indent ? 1 : 0);
        return this;
    }

    /// <summary>Aligns the label and control within the field.</summary>
    public FieldElement Align(FieldAlign align)
    {
        Arena.AddMethodEnum(Index, "align", align switch
        {
            FieldAlign.Center => "center",
            FieldAlign.End => "end",
            _ => "start",
        });
        return this;
    }

    /// <summary>Sets the field's grid-column span.</summary>
    public FieldElement ColSpan(int span)
    {
        Arena.AddMethodNumber(Index, "col_span", span);
        return this;
    }

    /// <summary>Adds the field's control content.</summary>
    public FieldElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
