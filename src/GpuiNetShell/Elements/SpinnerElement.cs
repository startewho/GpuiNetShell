using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A cycling loading spinner.</summary>
public sealed class SpinnerElement : Element
{
    internal SpinnerElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the spinner's semantic size.</summary>
    public SpinnerElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Selects the icon rotated by the spinner.</summary>
    public SpinnerElement Icon(SpinnerIcon icon)
    {
        Arena.AddMethodEnum(Index, "icon", icon switch
        {
            SpinnerIcon.LoaderCircle => "loader_circle",
            _ => "loader",
        });
        return this;
    }

    /// <summary>Sets the spinner icon color.</summary>
    public SpinnerElement Color(string color)
    {
        Arena.AddMethodString(Index, "color", color);
        return this;
    }

    /// <summary>Sets the spinner rotation easing curve.</summary>
    public SpinnerElement Ease(SpinnerEase ease)
    {
        Arena.AddMethodEnum(Index, "ease", ease switch
        {
            SpinnerEase.EaseInOut => "ease_in_out",
            SpinnerEase.EaseOutQuint => "ease_out_quint",
            _ => "linear",
        });
        return this;
    }
}
