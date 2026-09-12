using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A typed native tree data item with a Tree-wide unique id. Nested
/// <see cref="Add"/> children form the subtree.
/// </summary>
public sealed class TreeItemElement : Element
{
    internal TreeItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the initial expanded state.</summary>
    public TreeItemElement Expanded(bool expanded = true)
    {
        Arena.AddMethodNumber(Index, "expanded", expanded ? 1 : 0);
        return this;
    }

    /// <summary>Disables the item.</summary>
    public TreeItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Adds nested child items.</summary>
    public TreeItemElement Add(params TreeItemElement[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A retained tree accepting only <see cref="TreeItemElement"/> children.</summary>
public sealed class TreeElement : Element
{
    internal TreeElement(RenderContext ui, int index)
        : base(ui, index) { }

    public TreeElement Add(params TreeItemElement[] roots)
    {
        ArgumentNullException.ThrowIfNull(roots);
        foreach (var root in roots)
        {
            Arena.AddChild(Index, root.Index);
        }
        return this;
    }
}
