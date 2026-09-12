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
        element.Arena.AddNullaryStyle(element.Index, method);
        return element;
    }

    /// <summary>Records an arbitrary style method taking a pixel/number argument.</summary>
    public static T Style<T>(this T element, string method, double value)
        where T : Element
    {
        element.Arena.AddParamStyle(element.Index, method, value);
        return element;
    }

    /// <summary>Records an arbitrary style method taking a string argument.</summary>
    public static T StyleString<T>(this T element, string method, string value)
        where T : Element
    {
        element.Arena.AddParamStyleString(element.Index, method, value);
        return element;
    }

    /// <summary>Records an arbitrary style method taking a length argument.</summary>
    public static T Style<T>(this T element, string method, Length length)
        where T : Element
    {
        element.Arena.AddParamStyleString(element.Index, method, length.Wire);
        return element;
    }

    /// <summary>Records an arbitrary style method taking a color argument.</summary>
    public static T StyleColor<T>(this T element, string method, string color)
        where T : Element
    {
        element.Arena.AddParamStyleString(element.Index, method, color);
        return element;
    }

    // No-argument styles.
    public static T Flex<T>(this T element)
        where T : Element => element.Style("flex");

    public static T WFull<T>(this T element)
        where T : Element => element.Style("w_full");

    public static T HFull<T>(this T element)
        where T : Element => element.Style("h_full");

    public static T Full<T>(this T element)
        where T : Element => element.Style("size_full");

    public static T FlexColumn<T>(this T element)
        where T : Element => element.Style("flex_col");

    public static T FlexRow<T>(this T element)
        where T : Element => element.Style("flex_row");

    public static T Flex1<T>(this T element)
        where T : Element => element.Style("flex_1");

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

    public static T Relative<T>(this T element)
        where T : Element => element.Style("relative");

    public static T Absolute<T>(this T element)
        where T : Element => element.Style("absolute");

    // Length styles. Each keeps a pixel overload and adds a `Length` overload.
    public static T W<T>(this T element, double pixels)
        where T : Element => element.Style("w", pixels);

    public static T W<T>(this T element, Length length)
        where T : Element => element.Style("w", length);

    public static T H<T>(this T element, double pixels)
        where T : Element => element.Style("h", pixels);

    public static T H<T>(this T element, Length length)
        where T : Element => element.Style("h", length);

    public static T Size<T>(this T element, double pixels)
        where T : Element => element.Style("size", pixels);

    public static T Size<T>(this T element, Length length)
        where T : Element => element.Style("size", length);

    public static T MinW<T>(this T element, double pixels)
        where T : Element => element.Style("min_w", pixels);

    public static T MinW<T>(this T element, Length length)
        where T : Element => element.Style("min_w", length);

    public static T MinH<T>(this T element, double pixels)
        where T : Element => element.Style("min_h", pixels);

    public static T MinH<T>(this T element, Length length)
        where T : Element => element.Style("min_h", length);

    public static T MinSize<T>(this T element, double pixels)
        where T : Element => element.Style("min_size", pixels);

    public static T MinSize<T>(this T element, Length length)
        where T : Element => element.Style("min_size", length);

    public static T MaxW<T>(this T element, double pixels)
        where T : Element => element.Style("max_w", pixels);

    public static T MaxW<T>(this T element, Length length)
        where T : Element => element.Style("max_w", length);

    public static T MaxH<T>(this T element, double pixels)
        where T : Element => element.Style("max_h", pixels);

    public static T MaxH<T>(this T element, Length length)
        where T : Element => element.Style("max_h", length);

    public static T MaxSize<T>(this T element, double pixels)
        where T : Element => element.Style("max_size", pixels);

    public static T MaxSize<T>(this T element, Length length)
        where T : Element => element.Style("max_size", length);

    public static T P<T>(this T element, double pixels)
        where T : Element => element.Style("p", pixels);

    public static T P<T>(this T element, Length length)
        where T : Element => element.Style("p", length);

    public static T Px<T>(this T element, double pixels)
        where T : Element => element.Style("px", pixels);

    public static T Px<T>(this T element, Length length)
        where T : Element => element.Style("px", length);

    public static T Py<T>(this T element, double pixels)
        where T : Element => element.Style("py", pixels);

    public static T Py<T>(this T element, Length length)
        where T : Element => element.Style("py", length);

    public static T Pt<T>(this T element, double pixels)
        where T : Element => element.Style("pt", pixels);

    public static T Pt<T>(this T element, Length length)
        where T : Element => element.Style("pt", length);

    public static T Pb<T>(this T element, double pixels)
        where T : Element => element.Style("pb", pixels);

    public static T Pb<T>(this T element, Length length)
        where T : Element => element.Style("pb", length);

    public static T Pl<T>(this T element, double pixels)
        where T : Element => element.Style("pl", pixels);

    public static T Pl<T>(this T element, Length length)
        where T : Element => element.Style("pl", length);

    public static T Pr<T>(this T element, double pixels)
        where T : Element => element.Style("pr", pixels);

    public static T Pr<T>(this T element, Length length)
        where T : Element => element.Style("pr", length);

    public static T M<T>(this T element, double pixels)
        where T : Element => element.Style("m", pixels);

    public static T M<T>(this T element, Length length)
        where T : Element => element.Style("m", length);

    public static T Mx<T>(this T element, double pixels)
        where T : Element => element.Style("mx", pixels);

    public static T Mx<T>(this T element, Length length)
        where T : Element => element.Style("mx", length);

    public static T My<T>(this T element, double pixels)
        where T : Element => element.Style("my", pixels);

    public static T My<T>(this T element, Length length)
        where T : Element => element.Style("my", length);

    public static T Mt<T>(this T element, double pixels)
        where T : Element => element.Style("mt", pixels);

    public static T Mt<T>(this T element, Length length)
        where T : Element => element.Style("mt", length);

    public static T Mb<T>(this T element, double pixels)
        where T : Element => element.Style("mb", pixels);

    public static T Mb<T>(this T element, Length length)
        where T : Element => element.Style("mb", length);

    public static T Ml<T>(this T element, double pixels)
        where T : Element => element.Style("ml", pixels);

    public static T Ml<T>(this T element, Length length)
        where T : Element => element.Style("ml", length);

    public static T Mr<T>(this T element, double pixels)
        where T : Element => element.Style("mr", pixels);

    public static T Mr<T>(this T element, Length length)
        where T : Element => element.Style("mr", length);

    public static T Inset<T>(this T element, double pixels)
        where T : Element => element.Style("inset", pixels);

    public static T Inset<T>(this T element, Length length)
        where T : Element => element.Style("inset", length);

    public static T Top<T>(this T element, double pixels)
        where T : Element => element.Style("top", pixels);

    public static T Top<T>(this T element, Length length)
        where T : Element => element.Style("top", length);

    public static T Bottom<T>(this T element, double pixels)
        where T : Element => element.Style("bottom", pixels);

    public static T Bottom<T>(this T element, Length length)
        where T : Element => element.Style("bottom", length);

    public static T Left<T>(this T element, double pixels)
        where T : Element => element.Style("left", pixels);

    public static T Left<T>(this T element, Length length)
        where T : Element => element.Style("left", length);

    public static T Right<T>(this T element, double pixels)
        where T : Element => element.Style("right", pixels);

    public static T Right<T>(this T element, Length length)
        where T : Element => element.Style("right", length);

    public static T Gap<T>(this T element, double pixels)
        where T : Element => element.Style("gap", pixels);

    public static T Gap<T>(this T element, Length length)
        where T : Element => element.Style("gap", length);

    public static T GapX<T>(this T element, double pixels)
        where T : Element => element.Style("gap_x", pixels);

    public static T GapX<T>(this T element, Length length)
        where T : Element => element.Style("gap_x", length);

    public static T GapY<T>(this T element, double pixels)
        where T : Element => element.Style("gap_y", pixels);

    public static T GapY<T>(this T element, Length length)
        where T : Element => element.Style("gap_y", length);

    public static T Rounded<T>(this T element, double pixels)
        where T : Element => element.Style("rounded", pixels);

    public static T Rounded<T>(this T element, Length length)
        where T : Element => element.Style("rounded", length);

    public static T RoundedT<T>(this T element, double pixels)
        where T : Element => element.Style("rounded_t", pixels);

    public static T RoundedT<T>(this T element, Length length)
        where T : Element => element.Style("rounded_t", length);

    public static T RoundedB<T>(this T element, double pixels)
        where T : Element => element.Style("rounded_b", pixels);

    public static T RoundedB<T>(this T element, Length length)
        where T : Element => element.Style("rounded_b", length);

    public static T RoundedL<T>(this T element, double pixels)
        where T : Element => element.Style("rounded_l", pixels);

    public static T RoundedL<T>(this T element, Length length)
        where T : Element => element.Style("rounded_l", length);

    public static T RoundedR<T>(this T element, double pixels)
        where T : Element => element.Style("rounded_r", pixels);

    public static T RoundedR<T>(this T element, Length length)
        where T : Element => element.Style("rounded_r", length);

    public static T RoundedTl<T>(this T element, double pixels)
        where T : Element => element.Style("rounded_tl", pixels);

    public static T RoundedTl<T>(this T element, Length length)
        where T : Element => element.Style("rounded_tl", length);

    public static T RoundedTr<T>(this T element, double pixels)
        where T : Element => element.Style("rounded_tr", pixels);

    public static T RoundedTr<T>(this T element, Length length)
        where T : Element => element.Style("rounded_tr", length);

    public static T RoundedBl<T>(this T element, double pixels)
        where T : Element => element.Style("rounded_bl", pixels);

    public static T RoundedBl<T>(this T element, Length length)
        where T : Element => element.Style("rounded_bl", length);

    public static T RoundedBr<T>(this T element, double pixels)
        where T : Element => element.Style("rounded_br", pixels);

    public static T RoundedBr<T>(this T element, Length length)
        where T : Element => element.Style("rounded_br", length);

    public static T Border<T>(this T element, double pixels)
        where T : Element => element.Style("border", pixels);

    public static T Border<T>(this T element, Length length)
        where T : Element => element.Style("border", length);

    public static T BorderT<T>(this T element, double pixels)
        where T : Element => element.Style("border_t", pixels);

    public static T BorderT<T>(this T element, Length length)
        where T : Element => element.Style("border_t", length);

    public static T BorderB<T>(this T element, double pixels)
        where T : Element => element.Style("border_b", pixels);

    public static T BorderB<T>(this T element, Length length)
        where T : Element => element.Style("border_b", length);

    public static T BorderL<T>(this T element, double pixels)
        where T : Element => element.Style("border_l", pixels);

    public static T BorderL<T>(this T element, Length length)
        where T : Element => element.Style("border_l", length);

    public static T BorderR<T>(this T element, double pixels)
        where T : Element => element.Style("border_r", pixels);

    public static T BorderR<T>(this T element, Length length)
        where T : Element => element.Style("border_r", length);

    public static T BorderX<T>(this T element, double pixels)
        where T : Element => element.Style("border_x", pixels);

    public static T BorderX<T>(this T element, Length length)
        where T : Element => element.Style("border_x", length);

    public static T BorderY<T>(this T element, double pixels)
        where T : Element => element.Style("border_y", pixels);

    public static T BorderY<T>(this T element, Length length)
        where T : Element => element.Style("border_y", length);

    public static T TextSize<T>(this T element, double pixels)
        where T : Element => element.Style("text_size", pixels);

    public static T TextSize<T>(this T element, Length length)
        where T : Element => element.Style("text_size", length);

    public static T LineHeight<T>(this T element, double multiplier)
        where T : Element => element.Style("line_height", multiplier);

    public static T LineHeight<T>(this T element, Length length)
        where T : Element => element.Style("line_height", length);

    // Number styles.
    public static T Opacity<T>(this T element, double value)
        where T : Element => element.Style("opacity", value);

    public static T FlexGrow<T>(this T element, double value)
        where T : Element => element.Style("flex_grow", value);

    public static T FlexShrink<T>(this T element, double value)
        where T : Element => element.Style("flex_shrink", value);

    public static T FlexBasis<T>(this T element, double pixels)
        where T : Element => element.Style("flex_basis", pixels);

    public static T FlexBasis<T>(this T element, Length length)
        where T : Element => element.Style("flex_basis", length);

    public static T AspectRatio<T>(this T element, double ratio)
        where T : Element => element.Style("aspect_ratio", ratio);

    public static T FontWeight<T>(this T element, double value)
        where T : Element => element.Style("font_weight", value);

    // Grid placement and definition.
    public static T ColStart<T>(this T element, int start)
        where T : Element => element.Style("col_start", start);

    public static T ColEnd<T>(this T element, int end)
        where T : Element => element.Style("col_end", end);

    public static T ColSpan<T>(this T element, int span)
        where T : Element => element.Style("col_span", span);

    public static T RowStart<T>(this T element, int start)
        where T : Element => element.Style("row_start", start);

    public static T RowEnd<T>(this T element, int end)
        where T : Element => element.Style("row_end", end);

    public static T RowSpan<T>(this T element, int span)
        where T : Element => element.Style("row_span", span);

    public static T GridCols<T>(this T element, int cols)
        where T : Element => element.Style("grid_cols", cols);

    public static T GridColsMinContent<T>(this T element, int cols)
        where T : Element => element.Style("grid_cols_min_content", cols);

    public static T GridColsMaxContent<T>(this T element, int cols)
        where T : Element => element.Style("grid_cols_max_content", cols);

    public static T GridRows<T>(this T element, int rows)
        where T : Element => element.Style("grid_rows", rows);

    public static T GridRowsMinContent<T>(this T element, int rows)
        where T : Element => element.Style("grid_rows_min_content", rows);

    public static T GridRowsMaxContent<T>(this T element, int rows)
        where T : Element => element.Style("grid_rows_max_content", rows);

    // Text detail.
    public static T LineClamp<T>(this T element, int lines)
        where T : Element => element.Style("line_clamp", lines);

    public static T TextAlign<T>(this T element, TextAlignKind align)
        where T : Element =>
        element.StyleString("text_align", align switch
        {
            TextAlignKind.Center => "center",
            TextAlignKind.Right => "right",
            _ => "left",
        });

    public static T TextOverflow<T>(this T element, string ellipsis)
        where T : Element => element.StyleString("text_overflow", ellipsis);

    public static T TextDecorationColor<T>(this T element, string color)
        where T : Element => element.StyleColor("text_decoration_color", color);

    public static T ScrollbarWidth<T>(this T element, double pixels)
        where T : Element => element.Style("scrollbar_width", pixels);

    public static T ScrollbarWidth<T>(this T element, Length length)
        where T : Element => element.Style("scrollbar_width", length);

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
