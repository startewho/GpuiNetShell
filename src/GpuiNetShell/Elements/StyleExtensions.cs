namespace GpuiNetShell.Elements;

/// <summary>
/// The shared style surface. Every method records a GPUI style method name and
/// its argument; the native host maps the name to a direct GPUI style call
/// through the closed opcode vocabulary in <c>StyleOps</c>. A new style is one
/// entry there and one native arm, with no change to the C ABI.
/// </summary>
/// <remarks>
/// Names are the GPUI/Rust spelling (`items_center`, `size_full`, `p`, `gap`),
/// matching `gpui-shell`.
///
/// Lengths are ergonomic by default: an <see cref="int"/> is logical pixels
/// (<c>.W(200)</c>), a <see cref="double"/> from 0 to 1 is a percentage
/// (<c>.W(0.5)</c> is 50%), and a <see cref="Length"/> names any other unit
/// explicitly (<c>.W(Length.Auto)</c>, <c>.W(Length.Rems(1.5))</c>). A color is
/// a <c>#rgb</c>, <c>#rrggbb</c>, or <c>#rrggbbaa</c> literal.
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

    /// <summary>Records an arbitrary style method taking a numeric argument.</summary>
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

    // Length helpers: int pixels, double fraction (0..1) as a percentage.
    private static T Px<T>(T element, string method, int pixels)
        where T : Element => element.Style(method, (double)pixels);

    private static T Frac<T>(T element, string method, double fraction)
        where T : Element => element.Style(method, Length.Relative(fraction));

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

    /// <summary>
    /// Clips children to this element's bounds. Because the overflow is not
    /// visible, the element's automatic minimum size is zero, so a flexible
    /// child can shrink below its content instead of pushing its parent wider.
    /// </summary>
    public static T OverflowHidden<T>(this T element)
        where T : Element => element.Style("overflow_hidden");

    /// <summary>
    /// Lets flex children wrap onto new lines. With fixed-width cells this lays
    /// out an adaptive grid whose column count follows the available width.
    /// </summary>
    public static T Wrap<T>(this T element)
        where T : Element => element.Style("flex_wrap");

    // Width, height, size and min/max.
    public static T W<T>(this T element, int pixels)
        where T : Element => Px(element, "w", pixels);

    public static T W<T>(this T element, double fraction)
        where T : Element => Frac(element, "w", fraction);

    public static T W<T>(this T element, Length length)
        where T : Element => element.Style("w", length);

    public static T H<T>(this T element, int pixels)
        where T : Element => Px(element, "h", pixels);

    public static T H<T>(this T element, double fraction)
        where T : Element => Frac(element, "h", fraction);

    public static T H<T>(this T element, Length length)
        where T : Element => element.Style("h", length);

    public static T Size<T>(this T element, int pixels)
        where T : Element => Px(element, "size", pixels);

    public static T Size<T>(this T element, double fraction)
        where T : Element => Frac(element, "size", fraction);

    public static T Size<T>(this T element, Length length)
        where T : Element => element.Style("size", length);

    public static T MinW<T>(this T element, int pixels)
        where T : Element => Px(element, "min_w", pixels);

    public static T MinW<T>(this T element, double fraction)
        where T : Element => Frac(element, "min_w", fraction);

    public static T MinW<T>(this T element, Length length)
        where T : Element => element.Style("min_w", length);

    public static T MinH<T>(this T element, int pixels)
        where T : Element => Px(element, "min_h", pixels);

    public static T MinH<T>(this T element, double fraction)
        where T : Element => Frac(element, "min_h", fraction);

    public static T MinH<T>(this T element, Length length)
        where T : Element => element.Style("min_h", length);

    public static T MinSize<T>(this T element, int pixels)
        where T : Element => Px(element, "min_size", pixels);

    public static T MinSize<T>(this T element, double fraction)
        where T : Element => Frac(element, "min_size", fraction);

    public static T MinSize<T>(this T element, Length length)
        where T : Element => element.Style("min_size", length);

    public static T MaxW<T>(this T element, int pixels)
        where T : Element => Px(element, "max_w", pixels);

    public static T MaxW<T>(this T element, double fraction)
        where T : Element => Frac(element, "max_w", fraction);

    public static T MaxW<T>(this T element, Length length)
        where T : Element => element.Style("max_w", length);

    public static T MaxH<T>(this T element, int pixels)
        where T : Element => Px(element, "max_h", pixels);

    public static T MaxH<T>(this T element, double fraction)
        where T : Element => Frac(element, "max_h", fraction);

    public static T MaxH<T>(this T element, Length length)
        where T : Element => element.Style("max_h", length);

    public static T MaxSize<T>(this T element, int pixels)
        where T : Element => Px(element, "max_size", pixels);

    public static T MaxSize<T>(this T element, double fraction)
        where T : Element => Frac(element, "max_size", fraction);

    public static T MaxSize<T>(this T element, Length length)
        where T : Element => element.Style("max_size", length);

    // Padding.
    public static T P<T>(this T element, int pixels)
        where T : Element => Px(element, "p", pixels);

    public static T P<T>(this T element, double fraction)
        where T : Element => Frac(element, "p", fraction);

    public static T P<T>(this T element, Length length)
        where T : Element => element.Style("p", length);

    public static T Px<T>(this T element, int pixels)
        where T : Element => Px(element, "px", pixels);

    public static T Px<T>(this T element, double fraction)
        where T : Element => Frac(element, "px", fraction);

    public static T Px<T>(this T element, Length length)
        where T : Element => element.Style("px", length);

    public static T Py<T>(this T element, int pixels)
        where T : Element => Px(element, "py", pixels);

    public static T Py<T>(this T element, double fraction)
        where T : Element => Frac(element, "py", fraction);

    public static T Py<T>(this T element, Length length)
        where T : Element => element.Style("py", length);

    public static T Pt<T>(this T element, int pixels)
        where T : Element => Px(element, "pt", pixels);

    public static T Pt<T>(this T element, double fraction)
        where T : Element => Frac(element, "pt", fraction);

    public static T Pt<T>(this T element, Length length)
        where T : Element => element.Style("pt", length);

    public static T Pb<T>(this T element, int pixels)
        where T : Element => Px(element, "pb", pixels);

    public static T Pb<T>(this T element, double fraction)
        where T : Element => Frac(element, "pb", fraction);

    public static T Pb<T>(this T element, Length length)
        where T : Element => element.Style("pb", length);

    public static T Pl<T>(this T element, int pixels)
        where T : Element => Px(element, "pl", pixels);

    public static T Pl<T>(this T element, double fraction)
        where T : Element => Frac(element, "pl", fraction);

    public static T Pl<T>(this T element, Length length)
        where T : Element => element.Style("pl", length);

    public static T Pr<T>(this T element, int pixels)
        where T : Element => Px(element, "pr", pixels);

    public static T Pr<T>(this T element, double fraction)
        where T : Element => Frac(element, "pr", fraction);

    public static T Pr<T>(this T element, Length length)
        where T : Element => element.Style("pr", length);

    // Margin.
    public static T M<T>(this T element, int pixels)
        where T : Element => Px(element, "m", pixels);

    public static T M<T>(this T element, double fraction)
        where T : Element => Frac(element, "m", fraction);

    public static T M<T>(this T element, Length length)
        where T : Element => element.Style("m", length);

    public static T Mx<T>(this T element, int pixels)
        where T : Element => Px(element, "mx", pixels);

    public static T Mx<T>(this T element, double fraction)
        where T : Element => Frac(element, "mx", fraction);

    public static T Mx<T>(this T element, Length length)
        where T : Element => element.Style("mx", length);

    public static T My<T>(this T element, int pixels)
        where T : Element => Px(element, "my", pixels);

    public static T My<T>(this T element, double fraction)
        where T : Element => Frac(element, "my", fraction);

    public static T My<T>(this T element, Length length)
        where T : Element => element.Style("my", length);

    public static T Mt<T>(this T element, int pixels)
        where T : Element => Px(element, "mt", pixels);

    public static T Mt<T>(this T element, double fraction)
        where T : Element => Frac(element, "mt", fraction);

    public static T Mt<T>(this T element, Length length)
        where T : Element => element.Style("mt", length);

    public static T Mb<T>(this T element, int pixels)
        where T : Element => Px(element, "mb", pixels);

    public static T Mb<T>(this T element, double fraction)
        where T : Element => Frac(element, "mb", fraction);

    public static T Mb<T>(this T element, Length length)
        where T : Element => element.Style("mb", length);

    public static T Ml<T>(this T element, int pixels)
        where T : Element => Px(element, "ml", pixels);

    public static T Ml<T>(this T element, double fraction)
        where T : Element => Frac(element, "ml", fraction);

    public static T Ml<T>(this T element, Length length)
        where T : Element => element.Style("ml", length);

    public static T Mr<T>(this T element, int pixels)
        where T : Element => Px(element, "mr", pixels);

    public static T Mr<T>(this T element, double fraction)
        where T : Element => Frac(element, "mr", fraction);

    public static T Mr<T>(this T element, Length length)
        where T : Element => element.Style("mr", length);

    // Inset.
    public static T Inset<T>(this T element, int pixels)
        where T : Element => Px(element, "inset", pixels);

    public static T Inset<T>(this T element, double fraction)
        where T : Element => Frac(element, "inset", fraction);

    public static T Inset<T>(this T element, Length length)
        where T : Element => element.Style("inset", length);

    public static T Top<T>(this T element, int pixels)
        where T : Element => Px(element, "top", pixels);

    public static T Top<T>(this T element, double fraction)
        where T : Element => Frac(element, "top", fraction);

    public static T Top<T>(this T element, Length length)
        where T : Element => element.Style("top", length);

    public static T Bottom<T>(this T element, int pixels)
        where T : Element => Px(element, "bottom", pixels);

    public static T Bottom<T>(this T element, double fraction)
        where T : Element => Frac(element, "bottom", fraction);

    public static T Bottom<T>(this T element, Length length)
        where T : Element => element.Style("bottom", length);

    public static T Left<T>(this T element, int pixels)
        where T : Element => Px(element, "left", pixels);

    public static T Left<T>(this T element, double fraction)
        where T : Element => Frac(element, "left", fraction);

    public static T Left<T>(this T element, Length length)
        where T : Element => element.Style("left", length);

    public static T Right<T>(this T element, int pixels)
        where T : Element => Px(element, "right", pixels);

    public static T Right<T>(this T element, double fraction)
        where T : Element => Frac(element, "right", fraction);

    public static T Right<T>(this T element, Length length)
        where T : Element => element.Style("right", length);

    // Gap.
    public static T Gap<T>(this T element, int pixels)
        where T : Element => Px(element, "gap", pixels);

    public static T Gap<T>(this T element, double fraction)
        where T : Element => Frac(element, "gap", fraction);

    public static T Gap<T>(this T element, Length length)
        where T : Element => element.Style("gap", length);

    public static T GapX<T>(this T element, int pixels)
        where T : Element => Px(element, "gap_x", pixels);

    public static T GapX<T>(this T element, double fraction)
        where T : Element => Frac(element, "gap_x", fraction);

    public static T GapX<T>(this T element, Length length)
        where T : Element => element.Style("gap_x", length);

    public static T GapY<T>(this T element, int pixels)
        where T : Element => Px(element, "gap_y", pixels);

    public static T GapY<T>(this T element, double fraction)
        where T : Element => Frac(element, "gap_y", fraction);

    public static T GapY<T>(this T element, Length length)
        where T : Element => element.Style("gap_y", length);

    // Radius.
    public static T Rounded<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded", pixels);

    public static T Rounded<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded", fraction);

    public static T Rounded<T>(this T element, Length length)
        where T : Element => element.Style("rounded", length);

    public static T RoundedT<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded_t", pixels);

    public static T RoundedT<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded_t", fraction);

    public static T RoundedT<T>(this T element, Length length)
        where T : Element => element.Style("rounded_t", length);

    public static T RoundedB<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded_b", pixels);

    public static T RoundedB<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded_b", fraction);

    public static T RoundedB<T>(this T element, Length length)
        where T : Element => element.Style("rounded_b", length);

    public static T RoundedL<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded_l", pixels);

    public static T RoundedL<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded_l", fraction);

    public static T RoundedL<T>(this T element, Length length)
        where T : Element => element.Style("rounded_l", length);

    public static T RoundedR<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded_r", pixels);

    public static T RoundedR<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded_r", fraction);

    public static T RoundedR<T>(this T element, Length length)
        where T : Element => element.Style("rounded_r", length);

    public static T RoundedTl<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded_tl", pixels);

    public static T RoundedTl<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded_tl", fraction);

    public static T RoundedTl<T>(this T element, Length length)
        where T : Element => element.Style("rounded_tl", length);

    public static T RoundedTr<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded_tr", pixels);

    public static T RoundedTr<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded_tr", fraction);

    public static T RoundedTr<T>(this T element, Length length)
        where T : Element => element.Style("rounded_tr", length);

    public static T RoundedBl<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded_bl", pixels);

    public static T RoundedBl<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded_bl", fraction);

    public static T RoundedBl<T>(this T element, Length length)
        where T : Element => element.Style("rounded_bl", length);

    public static T RoundedBr<T>(this T element, int pixels)
        where T : Element => Px(element, "rounded_br", pixels);

    public static T RoundedBr<T>(this T element, double fraction)
        where T : Element => Frac(element, "rounded_br", fraction);

    public static T RoundedBr<T>(this T element, Length length)
        where T : Element => element.Style("rounded_br", length);

    // Border width.
    public static T Border<T>(this T element, int pixels)
        where T : Element => Px(element, "border", pixels);

    public static T Border<T>(this T element, double fraction)
        where T : Element => Frac(element, "border", fraction);

    public static T Border<T>(this T element, Length length)
        where T : Element => element.Style("border", length);

    public static T BorderT<T>(this T element, int pixels)
        where T : Element => Px(element, "border_t", pixels);

    public static T BorderT<T>(this T element, double fraction)
        where T : Element => Frac(element, "border_t", fraction);

    public static T BorderT<T>(this T element, Length length)
        where T : Element => element.Style("border_t", length);

    public static T BorderB<T>(this T element, int pixels)
        where T : Element => Px(element, "border_b", pixels);

    public static T BorderB<T>(this T element, double fraction)
        where T : Element => Frac(element, "border_b", fraction);

    public static T BorderB<T>(this T element, Length length)
        where T : Element => element.Style("border_b", length);

    public static T BorderL<T>(this T element, int pixels)
        where T : Element => Px(element, "border_l", pixels);

    public static T BorderL<T>(this T element, double fraction)
        where T : Element => Frac(element, "border_l", fraction);

    public static T BorderL<T>(this T element, Length length)
        where T : Element => element.Style("border_l", length);

    public static T BorderR<T>(this T element, int pixels)
        where T : Element => Px(element, "border_r", pixels);

    public static T BorderR<T>(this T element, double fraction)
        where T : Element => Frac(element, "border_r", fraction);

    public static T BorderR<T>(this T element, Length length)
        where T : Element => element.Style("border_r", length);

    public static T BorderX<T>(this T element, int pixels)
        where T : Element => Px(element, "border_x", pixels);

    public static T BorderX<T>(this T element, double fraction)
        where T : Element => Frac(element, "border_x", fraction);

    public static T BorderX<T>(this T element, Length length)
        where T : Element => element.Style("border_x", length);

    public static T BorderY<T>(this T element, int pixels)
        where T : Element => Px(element, "border_y", pixels);

    public static T BorderY<T>(this T element, double fraction)
        where T : Element => Frac(element, "border_y", fraction);

    public static T BorderY<T>(this T element, Length length)
        where T : Element => element.Style("border_y", length);

    public static T TextSize<T>(this T element, int pixels)
        where T : Element => Px(element, "text_size", pixels);

    public static T TextSize<T>(this T element, double fraction)
        where T : Element => Frac(element, "text_size", fraction);

    public static T TextSize<T>(this T element, Length length)
        where T : Element => element.Style("text_size", length);

    // Number styles (not lengths).
    public static T Opacity<T>(this T element, double value)
        where T : Element => element.Style("opacity", value);

    public static T FlexGrow<T>(this T element, double value)
        where T : Element => element.Style("flex_grow", value);

    public static T FlexShrink<T>(this T element, double value)
        where T : Element => element.Style("flex_shrink", value);

    public static T FlexBasis<T>(this T element, int pixels)
        where T : Element => Px(element, "flex_basis", pixels);

    public static T FlexBasis<T>(this T element, double fraction)
        where T : Element => Frac(element, "flex_basis", fraction);

    public static T FlexBasis<T>(this T element, Length length)
        where T : Element => element.Style("flex_basis", length);

    public static T AspectRatio<T>(this T element, double ratio)
        where T : Element => element.Style("aspect_ratio", ratio);

    public static T FontWeight<T>(this T element, double value)
        where T : Element => element.Style("font_weight", value);

    /// <summary>Sets the line height as a multiplier of the font size.</summary>
    public static T LineHeight<T>(this T element, double multiplier)
        where T : Element => element.Style("line_height", multiplier);

    public static T LineHeight<T>(this T element, Length length)
        where T : Element => element.Style("line_height", length);

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

    public static T ScrollbarWidth<T>(this T element, int pixels)
        where T : Element => Px(element, "scrollbar_width", pixels);

    public static T ScrollbarWidth<T>(this T element, double fraction)
        where T : Element => Frac(element, "scrollbar_width", fraction);

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
