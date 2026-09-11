using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A trigger container with an optional named <c>content</c> reveal. Ordinary
/// children are the trigger.
/// </summary>
public sealed class CollapsibleElement : Element
{
    internal CollapsibleElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Controls whether the content slot is revealed.</summary>
    public CollapsibleElement Open(bool open = true)
    {
        Arena.AddMethodNumber(Index, "open", open ? 1 : 0);
        return this;
    }

    /// <summary>Adds stable identity for a reversible measured reveal.</summary>
    public CollapsibleElement MotionId(string id)
    {
        Arena.AddMethodString(Index, "motion_id", id);
        return this;
    }

    /// <summary>Adds the trigger as ordinary children.</summary>
    public CollapsibleElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }

    /// <summary>Sets the revealed content slot.</summary>
    public CollapsibleElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }
}
