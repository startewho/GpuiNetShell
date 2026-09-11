using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A titled container for grouping related content.</summary>
public sealed class GroupBoxElement : Element
{
    internal GroupBoxElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the group title.</summary>
    public GroupBoxElement Title(string title)
    {
        Arena.AddMethodString(Index, "title", title);
        return this;
    }

    /// <summary>Sets the normal, fill, or outline presentation.</summary>
    public GroupBoxElement Variant(GroupBoxVariant variant)
    {
        Arena.AddMethodEnum(Index, "variant", variant switch
        {
            GroupBoxVariant.Fill => "fill",
            GroupBoxVariant.Outline => "outline",
            _ => "normal",
        });
        return this;
    }

    /// <summary>Adds the grouped body content.</summary>
    public GroupBoxElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
