using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A message banner created with a semantic variant. The variant is chosen by
/// the factory method on <c>RenderContext</c>.
/// </summary>
public sealed class AlertElement : Element
{
    internal AlertElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the alert title.</summary>
    public AlertElement Title(string title)
    {
        Arena.AddMethodString(Index, "title", title);
        return this;
    }

    /// <summary>Uses the full-width banner presentation.</summary>
    public AlertElement Banner()
    {
        Arena.AddMethod(Index, "banner");
        return this;
    }

    /// <summary>Controls whether the alert is rendered.</summary>
    public AlertElement Visible(bool visible = true)
    {
        Arena.AddMethodNumber(Index, "visible", visible ? 1 : 0);
        return this;
    }

    /// <summary>Sets the alert's semantic size.</summary>
    public AlertElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }
}
