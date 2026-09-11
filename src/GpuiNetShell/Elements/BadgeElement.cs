using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A count or dot badge positioned over its ordinary children. The badge is
/// configured entirely through methods, matching `component-shell`.
/// </summary>
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

    /// <summary>Sets the displayed count; zero hides a numeric badge.</summary>
    public BadgeElement Count(int count)
    {
        Arena.AddMethodNumber(Index, "count", count);
        return this;
    }

    /// <summary>Sets the largest count displayed before the plus suffix.</summary>
    public BadgeElement Max(int max)
    {
        Arena.AddMethodNumber(Index, "max", max);
        return this;
    }

    /// <summary>Sets the badge background from a supported color token.</summary>
    public BadgeElement Color(string color)
    {
        Arena.AddMethodString(Index, "color", color);
        return this;
    }

    /// <summary>Sets the semantic component size.</summary>
    public BadgeElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }
}
