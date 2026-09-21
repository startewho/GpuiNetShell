using GpuiNetShell.Events;
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

    /// <summary>Sets the control size; it also fixes the field height.</summary>
    public InputElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
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

    /// <summary>Runs when the field gains focus.</summary>
    public InputElement OnFocus(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_focus", Events.Register(handler));
        return this;
    }

    /// <summary>Runs when the field loses focus.</summary>
    public InputElement OnBlur(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_blur", Events.Register(handler));
        return this;
    }
}
