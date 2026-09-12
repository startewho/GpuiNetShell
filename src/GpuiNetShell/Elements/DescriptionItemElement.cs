using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A typed label/value child for <see cref="DescriptionListElement"/>. It
/// carries its native value to the parent and accepts no children or style.
/// </summary>
public sealed class DescriptionItemElement : Element
{
    internal DescriptionItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the item's textual value.</summary>
    public DescriptionItemElement Value(string value)
    {
        Arena.AddMethodString(Index, "value", value);
        return this;
    }

    /// <summary>Sets how many description-list columns the item spans.</summary>
    public DescriptionItemElement Span(int span)
    {
        Arena.AddMethodNumber(Index, "span", span);
        return this;
    }
}
