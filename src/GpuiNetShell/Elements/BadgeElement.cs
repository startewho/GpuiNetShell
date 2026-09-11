using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A small count or status badge.</summary>
public sealed class BadgeElement : Element
{
    internal BadgeElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Renders the badge as a dot rather than a count.</summary>
    public BadgeElement Dot()
    {
        Arena.AddMethod(Index, "dot");
        return this;
    }
}
