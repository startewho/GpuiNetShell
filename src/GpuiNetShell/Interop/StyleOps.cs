namespace GpuiNetShell.Interop;

/// <summary>
/// The closed style vocabulary, in opcode order. The index of a name is the
/// <c>u16</c> style opcode sent to the native host; the native host maps the
/// same index to a direct GPUI style method call (see
/// <c>crates/gpui-net-shell/src/style.rs</c>).
/// </summary>
/// <remarks>
/// This is a hand-kept mirror of the two lists declared by the
/// <c>style_vocabulary!</c> macro. <c>style.rs</c> has a
/// <c>the_managed_vocabulary_matches</c> test that parses this file and fails
/// if the names or their order drift, so an accidental reorder on either side
/// is caught before it becomes a silently wrong style.
///
/// Append new styles at the end; never reorder or remove, because the index is
/// the wire opcode.
/// </remarks>
internal static class StyleOps
{
    /// <summary>No-argument style methods; index == <see cref="NativeProtocol.OpNullaryStyle"/> opcode.</summary>
    internal static readonly string[] Nullary =
    [
        "flex",
        "flex_col",
        "flex_row",
        "flex_1",
        "w_full",
        "h_full",
        "size_full",
        "items_center",
        "items_start",
        "items_end",
        "justify_center",
        "justify_between",
        "justify_end",
        "font_semibold",
        "font_medium",
        "font_bold",
        "relative",
        "absolute",
        "overflow_hidden",
        "flex_wrap",
    ];

    /// <summary>Parametric style methods; index == <see cref="NativeProtocol.OpParamStyle"/> opcode.</summary>
    internal static readonly string[] Param =
    [
        "w",
        "h",
        "size",
        "min_w",
        "min_h",
        "min_size",
        "max_w",
        "max_h",
        "max_size",
        "p",
        "px",
        "py",
        "pt",
        "pb",
        "pl",
        "pr",
        "m",
        "mx",
        "my",
        "mt",
        "mb",
        "ml",
        "mr",
        "inset",
        "top",
        "bottom",
        "left",
        "right",
        "gap",
        "gap_x",
        "gap_y",
        "flex_grow",
        "flex_shrink",
        "flex_basis",
        "bg",
        "text_color",
        "text_bg",
        "text_size",
        "font_family",
        "font_weight",
        "line_height",
        "opacity",
        "border",
        "border_t",
        "border_b",
        "border_l",
        "border_r",
        "border_x",
        "border_y",
        "border_color",
        "rounded",
        "rounded_t",
        "rounded_b",
        "rounded_l",
        "rounded_r",
        "rounded_tl",
        "rounded_tr",
        "rounded_bl",
        "rounded_br",
        "aspect_ratio",
        "col_start",
        "col_end",
        "col_span",
        "row_start",
        "row_end",
        "row_span",
        "grid_cols",
        "grid_cols_min_content",
        "grid_cols_max_content",
        "grid_rows",
        "grid_rows_min_content",
        "grid_rows_max_content",
        "line_clamp",
        "text_align",
        "text_overflow",
        "text_decoration_color",
        "scrollbar_width",
    ];

    private static readonly Dictionary<string, ushort> NullaryCodes = Build(Nullary);
    private static readonly Dictionary<string, ushort> ParamCodes = Build(Param);

    /// <summary>The opcode of a no-argument style method, if it is known.</summary>
    internal static bool TryNullary(string method, out ushort code) =>
        NullaryCodes.TryGetValue(method, out code);

    /// <summary>The opcode of a parametric style method, if it is known.</summary>
    internal static bool TryParam(string method, out ushort code) =>
        ParamCodes.TryGetValue(method, out code);

    private static Dictionary<string, ushort> Build(string[] names)
    {
        var codes = new Dictionary<string, ushort>(names.Length, StringComparer.Ordinal);
        for (var index = 0; index < names.Length; index++)
        {
            codes[names[index]] = (ushort)index;
        }
        return codes;
    }
}
