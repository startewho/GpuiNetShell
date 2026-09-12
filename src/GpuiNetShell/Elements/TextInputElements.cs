using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A retained numeric text field.</summary>
public sealed class NumberInputElement : Element
{
    internal NumberInputElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the placeholder shown while empty.</summary>
    public NumberInputElement Placeholder(string placeholder)
    {
        Arena.AddMethodString(Index, "placeholder", placeholder);
        return this;
    }

    /// <summary>Sets the initial text.</summary>
    public NumberInputElement Value(string value)
    {
        Arena.AddMethodString(Index, "value", value);
        return this;
    }

    public NumberInputElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the new text after each edit.</summary>
    public NumberInputElement OnChange(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}

/// <summary>A retained multi-line text field.</summary>
public sealed class TextareaElement : Element
{
    internal TextareaElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the placeholder shown while empty.</summary>
    public TextareaElement Placeholder(string placeholder)
    {
        Arena.AddMethodString(Index, "placeholder", placeholder);
        return this;
    }

    /// <summary>Sets the initial text.</summary>
    public TextareaElement Value(string value)
    {
        Arena.AddMethodString(Index, "value", value);
        return this;
    }

    public TextareaElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the new text after each edit.</summary>
    public TextareaElement OnChange(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}

/// <summary>A retained fixed-length one-time-password field.</summary>
public sealed class OtpInputElement : Element
{
    internal OtpInputElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the fixed number of digits.</summary>
    public OtpInputElement Length(int length)
    {
        Arena.AddMethodNumber(Index, "length", length);
        return this;
    }

    /// <summary>Splits the code into the requested number of visual groups.</summary>
    public OtpInputElement Groups(int groups)
    {
        Arena.AddMethodNumber(Index, "groups", groups);
        return this;
    }

    public OtpInputElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the entered code after each edit.</summary>
    public OtpInputElement OnChange(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}

/// <summary>A retained numeric slider.</summary>
public sealed class SliderElement : Element
{
    internal SliderElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the initial value.</summary>
    public SliderElement Value(double value)
    {
        Arena.AddMethodNumber(Index, "value", value);
        return this;
    }

    /// <summary>Sets the minimum value.</summary>
    public SliderElement Min(double value)
    {
        Arena.AddMethodNumber(Index, "min", value);
        return this;
    }

    /// <summary>Sets the maximum value.</summary>
    public SliderElement Max(double value)
    {
        Arena.AddMethodNumber(Index, "max", value);
        return this;
    }

    /// <summary>Uses a vertical track.</summary>
    public SliderElement Vertical()
    {
        Arena.AddMethod(Index, "vertical");
        return this;
    }

    /// <summary>Reverses the filled side.</summary>
    public SliderElement Reverse()
    {
        Arena.AddMethod(Index, "reverse");
        return this;
    }

    public SliderElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the new value.</summary>
    public SliderElement OnChange(Action<double> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.Number));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}
