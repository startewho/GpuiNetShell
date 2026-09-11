using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// An external-resource link. Disabled state and activation are recorded like a
/// button; <see cref="Href"/> sets the URL opened when activated.
/// </summary>
public sealed class LinkElement : Element
{
    internal LinkElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the external URL opened when activated.</summary>
    public LinkElement Href(string href)
    {
        Arena.AddMethodString(Index, "href", href);
        return this;
    }

    public LinkElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Adds ordinary children shown as the link text.</summary>
    public LinkElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }

    /// <summary>Binds activation. The host requests a re-render after it returns.</summary>
    public LinkElement OnClick(Action handler)
    {
        var token = Events.Register(handler);
        Arena.AddCallback(Index, "on_click", token);
        return this;
    }
}
