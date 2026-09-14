//! The styling engine behind a style call.
//!
//! The managed host sends a *style opcode* and, for a parametric style, one
//! argument; this module answers one question for any code: which GPUI style
//! method is it, and how is it applied to a [`StyleRefinement`]?
//!
//! The vocabulary is **closed and declared here**, not reflected out of GPUI.
//! A previous revision enabled `gpui-base/inspector` and resolved method names
//! against a proc-macro-generated reflection table; that table forced every
//! `Styled` method to be retained and boxed every refinement through
//! `Box<dyn Any>` on each call. The two lists below are the whole surface the
//! managed `StyleExtensions` can emit, so each opcode is one direct call.
//!
//! * [`NULLARY`] lists the `fn(self) -> Self` methods (`flex_col`,
//!   `items_center`, `size_full`, …). The index is the opcode and the entry is
//!   the direct call.
//! * [`PARAM`] lists the methods that take one argument (`p`, `gap`, `bg`, …).
//!   The index is the opcode; [`apply_param`] binds each name to its call by
//!   hand because the argument type differs per method.
//!
//! The order of both lists is the wire vocabulary, mirrored in
//! `src/GpuiNetShell/Interop/StyleOps.cs`. `the_managed_vocabulary_matches`
//! keeps the two in step.

use gpui::{
    px, relative, rems, rgba, AbsoluteLength, DefiniteLength, FontWeight, Hsla, Length,
    StyleRefinement, Styled, TextAlign, TextOverflow,
};
use gpui_base::StyledExt as _;

/// One argument to a style or component method that takes one.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleArg {
    Number(f32),
    String(String),
    /// A closed-set literal for a component method (`size`, `scroll_axis`, …).
    Enum(String),
    /// A callback token passed as a component method argument (`on_change`).
    Callback(u64),
    /// A child node passed as an element argument, materialized lazily.
    Element(u32),
}

impl StyleArg {
    pub(crate) fn as_f32(&self) -> Result<f32, String> {
        match self {
            StyleArg::Number(value) => Ok(*value),
            StyleArg::String(_)
            | StyleArg::Enum(_)
            | StyleArg::Callback(_)
            | StyleArg::Element(_) => Err("expected a number, got a string".into()),
        }
    }

    pub(crate) fn as_str(&self) -> Result<&str, String> {
        match self {
            StyleArg::String(value) | StyleArg::Enum(value) => Ok(value),
            StyleArg::Number(_) | StyleArg::Callback(_) | StyleArg::Element(_) => {
                Err("expected a string, got a number".into())
            }
        }
    }

    /// Whether the argument reads as present/on, matching shell truthiness.
    pub(crate) fn is_truthy(&self) -> bool {
        match self {
            StyleArg::Number(value) => *value != 0.0 && !value.is_nan(),
            StyleArg::String(value) | StyleArg::Enum(value) => !value.is_empty(),
            StyleArg::Callback(_) | StyleArg::Element(_) => true,
        }
    }

    fn as_pixels(&self) -> Result<gpui::Pixels, String> {
        Ok(px(self.as_f32()?))
    }

    /// A `#rgb`, `#rrggbb`, or `#rrggbbaa` color value, or a named color.
    fn as_color(&self) -> Result<Hsla, String> {
        let text = self.as_str()?;
        if let Some(hex) = text.strip_prefix('#') {
            return parse_hex(hex).ok_or_else(|| {
                format!(
                    "`{text}` is not a valid color literal (expected #rgb, #rrggbb or #rrggbbaa)"
                )
            });
        }
        // Named colors (e.g. `red`, `orange-500`, `blue-600`) come from the
        // component library's palette, so a style can use them too.
        gpui_component::try_parse_color(text).map_err(|error| format!("`{text}`: {error}"))
    }
}

type NullaryFn = fn(StyleRefinement) -> StyleRefinement;

/// Declares the closed style vocabulary. Both arrays are indexed by opcode, and
/// the order is the wire contract mirrored in `StyleOps.cs`.
///
/// Reads the lists from the doc comment below rather than reflection: the
/// surface is small, explicit, and each entry is a direct method call.
macro_rules! style_vocabulary {
    (
        nullary { $($nullary:ident),* $(,)? }
        param { $($param:ident),* $(,)? }
    ) => {
        /// No-argument style methods, index == opcode.
        const NULLARY: &[(&str, NullaryFn)] =
            &[ $( (stringify!($nullary), |style| style.$nullary()) ),* ];

        /// Parametric style method names, index == opcode.
        const PARAM: &[&str] = &[ $( stringify!($param) ),* ];
    };
}

style_vocabulary! {
    // Every no-argument method the managed `StyleExtensions` emits. Order is
    // the opcode; append, never reorder.
    nullary {
        flex,
        flex_col,
        flex_row,
        flex_1,
        w_full,
        h_full,
        size_full,
        items_center,
        items_start,
        items_end,
        justify_center,
        justify_between,
        justify_end,
        font_semibold,
        font_medium,
        font_bold,
        relative,
        absolute,
    }
    // Every parametric method the managed surface can emit; `font_family` is
    // carried for completeness even though no builder exposes it yet.
    param {
        w,
        h,
        size,
        min_w,
        min_h,
        min_size,
        max_w,
        max_h,
        max_size,
        p,
        px,
        py,
        pt,
        pb,
        pl,
        pr,
        m,
        mx,
        my,
        mt,
        mb,
        ml,
        mr,
        inset,
        top,
        bottom,
        left,
        right,
        gap,
        gap_x,
        gap_y,
        flex_grow,
        flex_shrink,
        flex_basis,
        bg,
        text_color,
        text_bg,
        text_size,
        font_family,
        font_weight,
        line_height,
        opacity,
        border,
        border_t,
        border_b,
        border_l,
        border_r,
        border_x,
        border_y,
        border_color,
        rounded,
        rounded_t,
        rounded_b,
        rounded_l,
        rounded_r,
        rounded_tl,
        rounded_tr,
        rounded_bl,
        rounded_br,
        aspect_ratio,
        col_start,
        col_end,
        col_span,
        row_start,
        row_end,
        row_span,
        grid_cols,
        grid_cols_min_content,
        grid_cols_max_content,
        grid_rows,
        grid_rows_min_content,
        grid_rows_max_content,
        line_clamp,
        text_align,
        text_overflow,
        text_decoration_color,
        scrollbar_width,
    }
}

/// The no-argument method names, in opcode order.
#[cfg(test)]
pub fn nullary_names() -> Vec<&'static str> {
    NULLARY.iter().map(|(name, _)| *name).collect()
}

/// The parametric method names, in opcode order.
#[cfg(test)]
pub fn param_names() -> Vec<&'static str> {
    PARAM.to_vec()
}

/// The opcode of a no-argument style method, if `name` is one.
///
/// Used by tests and kept as the readable half of the vocabulary; the wire
/// itself carries the opcode.
#[allow(dead_code)]
pub fn nullary_index(name: &str) -> Option<u16> {
    NULLARY
        .iter()
        .position(|(candidate, _)| *candidate == name)
        .map(|index| index as u16)
}

/// The opcode of a parametric style method, if `name` is one.
#[allow(dead_code)]
pub fn param_index(name: &str) -> Option<u16> {
    PARAM
        .iter()
        .position(|candidate| *candidate == name)
        .map(|index| index as u16)
}

/// The name of a parametric style opcode, for diagnostics.
pub fn param_name(code: u16) -> Option<&'static str> {
    PARAM.get(code as usize).copied()
}

/// Applies a no-argument style opcode by direct call. An out-of-range opcode is
/// inert.
pub fn apply_nullary(code: u16, refinement: StyleRefinement) -> StyleRefinement {
    match NULLARY.get(code as usize) {
        Some((_, apply)) => apply(refinement),
        None => refinement,
    }
}

/// Applies a style method that takes one argument.
pub fn apply_param(
    code: u16,
    arg: &StyleArg,
    refinement: StyleRefinement,
) -> Result<StyleRefinement, String> {
    let name = param_name(code).ok_or_else(|| format!("unknown style opcode {code}"))?;

    macro_rules! length {
        () => {
            length(arg, name)?
        };
    }
    macro_rules! definite {
        () => {
            definite_length(arg, name)?
        };
    }
    macro_rules! absolute {
        () => {
            absolute_length(arg, name)?
        };
    }
    macro_rules! color {
        () => {
            arg.as_color()?
        };
    }
    macro_rules! number {
        () => {
            arg.as_f32()?
        };
    }

    Ok(match name {
        "w" => refinement.w(length!()),
        "h" => refinement.h(length!()),
        "size" => refinement.size(length!()),
        "min_w" => refinement.min_w(length!()),
        "min_h" => refinement.min_h(length!()),
        "min_size" => refinement.min_size(length!()),
        "max_w" => refinement.max_w(length!()),
        "max_h" => refinement.max_h(length!()),
        "max_size" => refinement.max_size(length!()),

        "p" => refinement.p(definite!()),
        "px" => refinement.px(definite!()),
        "py" => refinement.py(definite!()),
        "pt" => refinement.pt(definite!()),
        "pb" => refinement.pb(definite!()),
        "pl" => refinement.pl(definite!()),
        "pr" => refinement.pr(definite!()),

        "m" => refinement.m(length!()),
        "mx" => refinement.mx(length!()),
        "my" => refinement.my(length!()),
        "mt" => refinement.mt(length!()),
        "mb" => refinement.mb(length!()),
        "ml" => refinement.ml(length!()),
        "mr" => refinement.mr(length!()),

        "inset" => refinement.inset(length!()),
        "top" => refinement.top(length!()),
        "bottom" => refinement.bottom(length!()),
        "left" => refinement.left(length!()),
        "right" => refinement.right(length!()),

        "gap" => refinement.gap(definite!()),
        "gap_x" => refinement.gap_x(definite!()),
        "gap_y" => refinement.gap_y(definite!()),
        "flex_grow" => refinement.flex_grow(number!()),
        "flex_shrink" => refinement.flex_shrink(number!()),
        "flex_basis" => refinement.flex_basis(length!()),

        "bg" => refinement.bg(color!()),
        "text_color" => refinement.text_color(color!()),
        "text_bg" => refinement.text_bg(color!()),
        "text_size" => refinement.text_size(absolute!()),
        "font_family" => refinement.font_family(arg.as_str()?.to_owned()),
        "font_weight" => refinement.font_weight(font_weight(number!(), name)?),
        "line_height" => refinement.line_height(line_height(arg, name)?),
        "opacity" => refinement.opacity(number!()),

        "border" => refinement.border(absolute!()),
        "border_t" => refinement.border_t(absolute!()),
        "border_b" => refinement.border_b(absolute!()),
        "border_l" => refinement.border_l(absolute!()),
        "border_r" => refinement.border_r(absolute!()),
        "border_x" => refinement.border_x(absolute!()),
        "border_y" => refinement.border_y(absolute!()),
        "border_color" => refinement.border_color(color!()),

        "rounded" => refinement.rounded(absolute!()),
        "rounded_t" => refinement.rounded_t(absolute!()),
        "rounded_b" => refinement.rounded_b(absolute!()),
        "rounded_l" => refinement.rounded_l(absolute!()),
        "rounded_r" => refinement.rounded_r(absolute!()),
        "rounded_tl" => refinement.rounded_tl(absolute!()),
        "rounded_tr" => refinement.rounded_tr(absolute!()),
        "rounded_bl" => refinement.rounded_bl(absolute!()),
        "rounded_br" => refinement.rounded_br(absolute!()),

        "aspect_ratio" => refinement.aspect_ratio(number!()),
        "col_start" => refinement.col_start(to_i16(arg.as_f32()?, name)?),
        "col_end" => refinement.col_end(to_i16(arg.as_f32()?, name)?),
        "col_span" => refinement.col_span(to_u16(arg.as_f32()?, name)?),
        "row_start" => refinement.row_start(to_i16(arg.as_f32()?, name)?),
        "row_end" => refinement.row_end(to_i16(arg.as_f32()?, name)?),
        "row_span" => refinement.row_span(to_u16(arg.as_f32()?, name)?),
        "grid_cols" => refinement.grid_cols(to_u16(arg.as_f32()?, name)?),
        "grid_cols_min_content" => refinement.grid_cols_min_content(to_u16(arg.as_f32()?, name)?),
        "grid_cols_max_content" => refinement.grid_cols_max_content(to_u16(arg.as_f32()?, name)?),
        "grid_rows" => refinement.grid_rows(to_u16(arg.as_f32()?, name)?),
        "grid_rows_min_content" => refinement.grid_rows_min_content(to_u16(arg.as_f32()?, name)?),
        "grid_rows_max_content" => refinement.grid_rows_max_content(to_u16(arg.as_f32()?, name)?),
        "line_clamp" => refinement.line_clamp(to_usize(arg.as_f32()?, name)?),
        "scrollbar_width" => refinement.scrollbar_width(absolute!()),
        "text_align" => refinement.text_align(match arg.as_str()? {
            "left" => TextAlign::Left,
            "center" => TextAlign::Center,
            "right" => TextAlign::Right,
            other => return Err(format!("unsupported `text_align` value `{other}`")),
        }),
        "text_overflow" => {
            refinement.text_overflow(TextOverflow::Truncate(arg.as_str()?.to_owned().into()))
        }
        "text_decoration_color" => refinement.text_decoration_color(color!()),

        other => return Err(format!("unknown style method `{other}`")),
    })
}

/// Narrows a style number to an exact `u16`, rejecting fractions and overflow.
fn to_u16(value: f32, name: &str) -> Result<u16, String> {
    if value.is_finite() && value >= 0.0 && value <= u16::MAX as f32 && value.fract() == 0.0 {
        Ok(value as u16)
    } else {
        Err(format!(
            "`{name}` expects an integer from 0 to {}",
            u16::MAX
        ))
    }
}

/// Narrows a style number to an exact `i16`, rejecting fractions and overflow.
fn to_i16(value: f32, name: &str) -> Result<i16, String> {
    if value.is_finite()
        && value >= i16::MIN as f32
        && value <= i16::MAX as f32
        && value.fract() == 0.0
    {
        Ok(value as i16)
    } else {
        Err(format!(
            "`{name}` expects an integer from {} to {}",
            i16::MIN,
            i16::MAX
        ))
    }
}

/// Narrows a style number to an exact `usize`, rejecting fractions and overflow.
fn to_usize(value: f32, name: &str) -> Result<usize, String> {
    if value.is_finite() && value >= 0.0 && value <= usize::MAX as f32 && value.fract() == 0.0 {
        Ok(value as usize)
    } else {
        Err(format!("`{name}` expects a non-negative integer"))
    }
}

/// A length as written, before narrowing to what a method accepts.
enum LengthLiteral {
    Absolute(AbsoluteLength),
    Fraction(f32),
    Auto,
}

fn parse_length(value: &StyleArg, method: &str) -> Result<LengthLiteral, String> {
    if let StyleArg::String(text) = value {
        let text = text.trim();
        if text == "auto" {
            return Ok(LengthLiteral::Auto);
        }
        if let Some(number) = text.strip_suffix('%') {
            return parse_number(number, text, method)
                .map(|value| LengthLiteral::Fraction(value / 100.));
        }
        if let Some(number) = text.strip_suffix("rem") {
            return parse_number(number, text, method)
                .map(|value| LengthLiteral::Absolute(rems(value).into()));
        }
        if let Some(number) = text.strip_suffix("px") {
            return parse_number(number, text, method)
                .map(|value| LengthLiteral::Absolute(px(value).into()));
        }
        return Err(format!(
            "`{method}` expects a length: a number of pixels, or a string like \
             \"50%\", \"12px\", \"1rem\" or \"auto\"; got \"{text}\""
        ));
    }

    Ok(LengthLiteral::Absolute(value.as_pixels()?.into()))
}

fn parse_number(number: &str, text: &str, method: &str) -> Result<f32, String> {
    number
        .trim()
        .parse::<f32>()
        .map_err(|_| format!("`{method}` could not read a number in the length \"{text}\""))
}

fn length(value: &StyleArg, method: &str) -> Result<Length, String> {
    Ok(match parse_length(value, method)? {
        LengthLiteral::Absolute(absolute) => Length::Definite(absolute.into()),
        LengthLiteral::Fraction(fraction) => Length::Definite(relative(fraction)),
        LengthLiteral::Auto => Length::Auto,
    })
}

/// A bare number is a multiplier; anything else follows the length grammar.
fn line_height(value: &StyleArg, method: &str) -> Result<DefiniteLength, String> {
    match value {
        StyleArg::Number(multiplier) => Ok(relative(*multiplier)),
        other => definite_length(other, method),
    }
}

fn font_weight(value: f32, method: &str) -> Result<FontWeight, String> {
    if value.is_finite() && (100. ..=900.).contains(&value) {
        Ok(FontWeight(value))
    } else {
        Err(format!(
            "`{method}` expects a finite number between 100 and 900; got {value}"
        ))
    }
}

fn definite_length(value: &StyleArg, method: &str) -> Result<DefiniteLength, String> {
    match parse_length(value, method)? {
        LengthLiteral::Absolute(absolute) => Ok(absolute.into()),
        LengthLiteral::Fraction(fraction) => Ok(relative(fraction)),
        LengthLiteral::Auto => Err(format!(
            "`{method}` cannot be \"auto\"; it expects a definite length such as 12 or \"50%\""
        )),
    }
}

fn absolute_length(value: &StyleArg, method: &str) -> Result<AbsoluteLength, String> {
    match parse_length(value, method)? {
        LengthLiteral::Absolute(absolute) => Ok(absolute),
        LengthLiteral::Fraction(_) | LengthLiteral::Auto => Err(format!(
            "`{method}` expects an absolute length such as 8 or \"0.5rem\"; \
             percentages and \"auto\" are not allowed here"
        )),
    }
}

fn parse_hex(hex: &str) -> Option<Hsla> {
    let expand = |c: char| {
        let d = c.to_digit(16)?;
        Some(d * 17)
    };

    let rgba_value = match hex.len() {
        3 => {
            let mut chars = hex.chars();
            let r = expand(chars.next()?)?;
            let g = expand(chars.next()?)?;
            let b = expand(chars.next()?)?;
            (r << 24) | (g << 16) | (b << 8) | 0xff
        }
        6 => (u32::from_str_radix(hex, 16).ok()? << 8) | 0xff,
        8 => u32::from_str_radix(hex, 16).ok()?,
        _ => return None,
    };

    Some(rgba(rgba_value).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Fill, Hsla};

    /// The exact vocabulary, pinned so an accidental reorder is a reviewable
    /// diff rather than a silent wire break. The managed mirror is
    /// `src/GpuiNetShell/Interop/StyleOps.cs`.
    #[test]
    fn the_vocabulary_is_closed_and_pinned() {
        assert_eq!(
            nullary_names(),
            vec![
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
            ]
        );
        assert_eq!(param_names().len(), 77);
    }

    /// The managed `StyleOps` arrays must list the same names in the same
    /// order, or a style would silently map to the wrong opcode.
    #[test]
    fn the_managed_vocabulary_matches() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src/GpuiNetShell/Interop/StyleOps.cs"
        );
        let source = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("read managed StyleOps from {path}: {error}"));
        assert_eq!(
            array_literals(&source, "Nullary"),
            nullary_names(),
            "Nullary in StyleOps.cs drifted from style.rs"
        );
        assert_eq!(
            array_literals(&source, "Param"),
            param_names(),
            "Param in StyleOps.cs drifted from style.rs"
        );
    }

    /// Reads the quoted entries of the managed `static readonly string[] Name = [ … ];`.
    fn array_literals(source: &str, name: &str) -> Vec<String> {
        let declaration = format!("string[] {name}");
        let anchor = source
            .find(&declaration)
            .unwrap_or_else(|| panic!("StyleOps.cs has no `{declaration}`"))
            + declaration.len();
        let start = source[anchor..]
            .find('[')
            .unwrap_or_else(|| panic!("StyleOps.cs `{name}` has no array literal"))
            + anchor
            + 1;
        let end = source[start..]
            .find(']')
            .unwrap_or_else(|| panic!("StyleOps.cs `{name}` array is unterminated"))
            + start;
        let body = &source[start..end];
        body.split(',')
            .map(|entry| entry.trim().trim_matches('"'))
            .filter(|entry| !entry.is_empty())
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn a_nullary_opcode_applies_its_method_directly() {
        let code = nullary_index("items_center").expect("items_center is in the vocabulary");
        let styled = apply_nullary(code, StyleRefinement::default());
        assert_eq!(styled.align_items, Some(gpui::AlignItems::Center));
    }

    #[test]
    fn layout_nullary_names_are_in_the_vocabulary() {
        for name in ["flex_row", "flex_col", "w_full", "h_full", "size_full"] {
            assert!(nullary_index(name).is_some(), "`{name}` is missing");
        }
    }

    #[test]
    fn an_out_of_range_nullary_opcode_is_inert() {
        let styled = apply_nullary(u16::MAX, StyleRefinement::default());
        assert_eq!(styled, StyleRefinement::default());
    }

    #[test]
    fn grid_text_and_scroll_parametric_styles_apply() {
        use StyleArg::{Number, String};
        let default = StyleRefinement::default;
        for name in [
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
            "scrollbar_width",
        ] {
            let code = param_index(name).unwrap_or_else(|| panic!("`{name}` is missing"));
            assert!(
                apply_param(code, &Number(2.0), default()).is_ok(),
                "`{name}` did not apply"
            );
        }
        assert!(apply_param(param_index("text_align").unwrap(), &String("center".into()), default()).is_ok());
        assert!(apply_param(param_index("text_overflow").unwrap(), &String("…".into()), default()).is_ok());
        assert!(apply_param(
            param_index("text_decoration_color").unwrap(),
            &String("#ff0000".into()),
            default()
        )
        .is_ok());
        assert!(apply_param(param_index("text_align").unwrap(), &String("middle".into()), default()).is_err());
        assert!(apply_param(param_index("col_span").unwrap(), &Number(1.5), default()).is_err());
    }

    #[test]
    fn a_bare_number_is_pixels_and_a_percent_string_is_relative() {
        let padded = apply_param(
            param_index("p").unwrap(),
            &StyleArg::Number(12.),
            StyleRefinement::default(),
        )
        .unwrap();
        assert_eq!(padded.padding.top, Some(px(12.).into()));

        let wide = apply_param(
            param_index("w").unwrap(),
            &StyleArg::String("50%".into()),
            StyleRefinement::default(),
        )
        .unwrap();
        assert_eq!(wide.size.width, Some(Length::Definite(relative(0.5))));
    }

    #[test]
    fn bg_sets_a_background_from_a_hex_literal() {
        let styled = apply_param(
            param_index("bg").unwrap(),
            &StyleArg::String("#ff0000".into()),
            StyleRefinement::default(),
        )
        .unwrap();

        let expected: Fill = Hsla::from(gpui::rgba(0xff0000ff)).into();
        assert_eq!(styled.background, Some(expected));
    }

    #[test]
    fn font_weight_rejects_out_of_range_values() {
        assert!(apply_param(
            param_index("font_weight").unwrap(),
            &StyleArg::Number(99.),
            StyleRefinement::default()
        )
        .is_err());
    }

    #[test]
    fn an_unknown_parametric_opcode_is_an_error() {
        assert!(apply_param(
            u16::MAX,
            &StyleArg::Number(1.),
            StyleRefinement::default()
        )
        .is_err());
    }

    #[test]
    fn the_two_vocabularies_are_disjoint() {
        for name in nullary_names() {
            assert!(
                param_index(name).is_none(),
                "`{name}` is both nullary and parametric"
            );
        }
    }
}
