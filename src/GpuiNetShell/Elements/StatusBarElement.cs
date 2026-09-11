using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A three-region status bar. Ordinary children fill the center;
/// <see cref="LeftContent"/> and <see cref="RightContent"/> pin content to each
/// edge.
/// </summary>
public sealed class StatusBarElement : Element
{
    internal StatusBarElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Pins content to the leading edge.</summary>
    public StatusBarElement LeftContent(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddMethodElement(Index, "left_content", content.Index);
        return this;
    }

    /// <summary>Pins content to the trailing edge.</summary>
    public StatusBarElement RightContent(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddMethodElement(Index, "right_content", content.Index);
        return this;
    }

    /// <summary>Adds center content.</summary>
    public StatusBarElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
