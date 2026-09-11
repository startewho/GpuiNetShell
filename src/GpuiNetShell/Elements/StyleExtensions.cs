namespace GpuiNetShell.Elements;

/// <summary>
/// The shared style surface. Every method records a GPUI style method name and
/// its argument; the native host resolves the name against GPUI's reflected
/// style table, so a new style needs no native operation code and no change to
/// the C ABI.
/// </summary>
/// <remarks>
/// Names are the GPUI/Rust spelling (`items_center`, `size_full`, `p`, `gap`),
/// matching `gpui-shell`. A bare number is pixels; a string length is `"auto"`,
/// `"50%"`, `"12px"`, or `"1rem"`; a color is a `#rgb`, `#rrggbb`, or
/// `#rrggbbaa` literal.
/// </remarks>
public static class StyleExtensions
{
    /// <summary>Records an arbitrary no-argument style method.</summary>
    public static T Style<T>(this T element, string method)
        where T : Element
    {
        element.Arena.AddStyleNullary(element.Index, method);
        return element;
    }

    /// <summary>Records an arbitrary style method taking a pixel/number argument.</summary>
    public static T Style<T>(this T element, string method, double value)
        where T : Element
    {
        element.Arena.AddStyleLength(element.Index, method, value);
        return element;
    }

    /// <summary>Records an arbitrary style method taking a string argument.</summary>
    public static T StyleString<T>(this T element, string method, string value)
        where T : Element
    {
        element.Arena.AddStyleString(element.Index, method, value);
        return element;
    }

    /// <summary>Records an arbitrary style method taking a color argument.</summary>
    public static T StyleColor<T>(this T element, string method, string color)
        where T : Element
    {
        element.Arena.AddStyleColor(element.Index, method, color);
        return element;
    }

    // No-argument styles.
    public static T Full<T>(this T element)
        where T : Element => element.Style("size_full");

    public static T FlexColumn<T>(this T element)
        where T : Element => element.Style("flex_col");

    public static T FlexRow<T>(this T element)
        where T : Element => element.Style("flex_row");

    public static T ItemsCenter<T>(this T element)
        where T : Element => element.Style("items_center");

    public static T ItemsStart<T>(this T element)
        where T : Element => element.Style("items_start");

    public static T ItemsEnd<T>(this T element)
        where T : Element => element.Style("items_end");

    public static T JustifyCenter<T>(this T element)
        where T : Element => element.Style("justify_center");

    public static T JustifyBetween<T>(this T element)
        where T : Element => element.Style("justify_between");

    public static T JustifyEnd<T>(this T element)
        where T : Element => element.Style("justify_end");

    public static T FontSemibold<T>(this T element)
        where T : Element => element.Style("font_semibold");

    public static T FontBold<T>(this T element)
        where T : Element => element.Style("font_bold");

    public static T FontMedium<T>(this T element)
        where T : Element => element.Style("font_medium");

    // Length styles.
    public static T W<T>(this T element, double pixels)
        where T : Element => element.Style("w", pixels);

    public static T H<T>(this T element, double pixels)
        where T : Element => element.Style("h", pixels);

    public static T Size<T>(this T element, double pixels)
        where T : Element => element.Style("size", pixels);

    public static T MinW<T>(this T element, double pixels)
        where T : Element => element.Style("min_w", pixels);

    public static T MinH<T>(this T element, double pixels)
        where T : Element => element.Style("min_h", pixels);

    public static T MaxW<T>(this T element, double pixels)
        where T : Element => element.Style("max_w", pixels);

    public static T MaxH<T>(this T element, double pixels)
        where T : Element => element.Style("max_h", pixels);

    public static T P<T>(this T element, double pixels)
        where T : Element => element.Style("p", pixels);

    public static T Px<T>(this T element, double pixels)
        where T : Element => element.Style("px", pixels);

    public static T Py<T>(this T element, double pixels)
        where T : Element => element.Style("py", pixels);

    public static T M<T>(this T element, double pixels)
        where T : Element => element.Style("m", pixels);

    public static T Mx<T>(this T element, double pixels)
        where T : Element => element.Style("mx", pixels);

    public static T My<T>(this T element, double pixels)
        where T : Element => element.Style("my", pixels);

    public static T Gap<T>(this T element, double pixels)
        where T : Element => element.Style("gap", pixels);

    public static T GapX<T>(this T element, double pixels)
        where T : Element => element.Style("gap_x", pixels);

    public static T GapY<T>(this T element, double pixels)
        where T : Element => element.Style("gap_y", pixels);

    public static T Rounded<T>(this T element, double pixels)
        where T : Element => element.Style("rounded", pixels);

    public static T Border<T>(this T element, double pixels)
        where T : Element => element.Style("border", pixels);

    public static T TextSize<T>(this T element, double pixels)
        where T : Element => element.Style("text_size", pixels);

    public static T LineHeight<T>(this T element, double multiplier)
        where T : Element => element.Style("line_height", multiplier);

    // Number styles.
    public static T Opacity<T>(this T element, double value)
        where T : Element => element.Style("opacity", value);

    public static T FlexGrow<T>(this T element, double value)
        where T : Element => element.Style("flex_grow", value);

    public static T FlexShrink<T>(this T element, double value)
        where T : Element => element.Style("flex_shrink", value);

    public static T FontWeight<T>(this T element, double value)
        where T : Element => element.Style("font_weight", value);

    // Color styles.
    public static T Bg<T>(this T element, string color)
        where T : Element => element.StyleColor("bg", color);

    public static T TextColor<T>(this T element, string color)
        where T : Element => element.StyleColor("text_color", color);

    public static T TextBg<T>(this T element, string color)
        where T : Element => element.StyleColor("text_bg", color);

    public static T BorderColor<T>(this T element, string color)
        where T : Element => element.StyleColor("border_color", color);
}
