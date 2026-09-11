using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>An interactive star rating.</summary>
public sealed class RatingElement : Element
{
    internal RatingElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the current number of active stars.</summary>
    public RatingElement Value(int value)
    {
        Arena.AddMethodNumber(Index, "value", value);
        return this;
    }

    /// <summary>Sets the maximum number of stars.</summary>
    public RatingElement Max(int max)
    {
        Arena.AddMethodNumber(Index, "max", max);
        return this;
    }

    /// <summary>Sets the active star color.</summary>
    public RatingElement Color(string color)
    {
        Arena.AddMethodString(Index, "color", color);
        return this;
    }

    /// <summary>Sets the rating's semantic size.</summary>
    public RatingElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    public RatingElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the star the reader clicked.</summary>
    public RatingElement OnChange(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler((int)value.Number));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}
