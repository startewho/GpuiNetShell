using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A retained single-line text field.</summary>
public sealed class InputElement : Element
{
    internal InputElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the placeholder shown while empty.</summary>
    public InputElement Placeholder(string placeholder)
    {
        Arena.AddMethodString(Index, "placeholder", placeholder);
        return this;
    }

    /// <summary>Sets the initial text.</summary>
    public InputElement Value(string value)
    {
        Arena.AddMethodString(Index, "value", value);
        return this;
    }

    public InputElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the new text after each edit.</summary>
    public InputElement OnChange(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}
