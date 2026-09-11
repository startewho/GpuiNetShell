using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A compact semantic status tag that renders ordinary children.
/// </summary>
public sealed class TagElement : Element
{
    internal TagElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the semantic tag variant.</summary>
    public TagElement Variant(TagVariant variant)
    {
        Arena.AddMethodEnum(Index, "variant", variant switch
        {
            TagVariant.Secondary => "secondary",
            TagVariant.Danger => "danger",
            TagVariant.Success => "success",
            TagVariant.Warning => "warning",
            TagVariant.Info => "info",
            _ => "primary",
        });
        return this;
    }

    /// <summary>Uses the outline presentation.</summary>
    public TagElement Outline()
    {
        Arena.AddMethod(Index, "outline");
        return this;
    }

    /// <summary>Uses pill-shaped corners.</summary>
    public TagElement RoundedFull()
    {
        Arena.AddMethod(Index, "rounded_full");
        return this;
    }

    /// <summary>Sets the semantic component size.</summary>
    public TagElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Adds ordinary children shown inside the tag.</summary>
    public TagElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
