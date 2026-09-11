//! The styling engine behind a style call, ported from `gpui-shell`.
//!
//! The managed host sends a *method name* and an argument; this module answers
//! one question for any name: is it a style method, and how is it applied to a
//! [`StyleRefinement`]?
//!
//! There are two halves, for different reasons:
//!
//! * **No-argument methods** are reflected out of GPUI
//!   (`gpui_base::styled_ext_reflection_methods` and
//!   `gpui::styled_reflection::methods`), so `flex_col`, `items_center`,
//!   `size_full`, `rounded_md`, `text_sm`, and hundreds more are available with
//!   no maintenance and without a fixed enum of style operations.
//! * **Methods that take arguments** cannot be reflected and are bound by hand
//!   in [`apply_param`].
//!
//! A style name that is neither reflected nor bound is a no-op here; the
//! managed surface only emits names it knows, and a typo is visible at the call
//! site rather than silently accepted.

use std::collections::HashMap;
use std::sync::OnceLock;

use gpui::inspector_reflection::FunctionReflection;
use gpui::{
    px, relative, rems, rgba, AbsoluteLength, DefiniteLength, FontWeight, Hsla, Length,
    StyleRefinement, Styled,
};
use gpui_base::StyledExt as _;

/// One argument to a style method that takes one.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleArg {
    Number(f32),
    String(String),
}

impl StyleArg {
    fn as_f32(&self) -> Result<f32, String> {
        match self {
            StyleArg::Number(value) => Ok(*value),
            StyleArg::String(_) => Err("expected a number, got a string".into()),
        }
    }

    fn as_str(&self) -> Result<&str, String> {
        match self {
            StyleArg::String(value) => Ok(value),
            StyleArg::Number(_) => Err("expected a string, got a number".into()),
        }
    }

    fn as_pixels(&self) -> Result<gpui::Pixels, String> {
        Ok(px(self.as_f32()?))
    }

    /// A `#rgb`, `#rrggbb`, or `#rrggbbaa` color value.
    fn as_color(&self) -> Result<Hsla, String> {
        let text = self.as_str()?;
        let Some(hex) = text.strip_prefix('#') else {
            return Err(format!(
                "`{text}` is not a color literal (expected #rgb, #rrggbb or #rrggbbaa)"
            ));
        };
        parse_hex(hex).ok_or_else(|| {
            format!("`{text}` is not a valid color literal (expected #rgb, #rrggbb or #rrggbbaa)")
        })
    }
}

/// Style methods that take one argument, bound by hand.
const PARAM_STYLES: &[&str] = &[
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
];

type NullaryFn = fn(StyleRefinement) -> StyleRefinement;

/// No-argument style methods reflection does not reach. `gpui-base` generates
/// its font-weight helpers with a macro, which the reflection pass skips.
const EXTRA_NULLARY: &[(&str, NullaryFn)] = &[
    ("font_thin", |style| style.font_thin()),
    ("font_extralight", |style| style.font_extralight()),
    ("font_light", |style| style.font_light()),
    ("font_normal", |style| style.font_normal()),
    ("font_medium", |style| style.font_medium()),
    ("font_semibold", |style| style.font_semibold()),
    ("font_bold", |style| style.font_bold()),
    ("font_extrabold", |style| style.font_extrabold()),
    ("font_black", |style| style.font_black()),
];

struct StyleTable {
    nullary: Vec<FunctionReflection<StyleRefinement>>,
    by_name: HashMap<&'static str, u16>,
}

fn table() -> &'static StyleTable {
    static TABLE: OnceLock<StyleTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        let nullary: Vec<_> = [
            gpui_base::styled_ext_reflection_methods::<StyleRefinement>(),
            gpui::styled_reflection::methods::<StyleRefinement>(),
        ]
        .into_iter()
        .flatten()
        .collect();

        let mut by_name = HashMap::with_capacity(nullary.len() + EXTRA_NULLARY.len());
        for (index, method) in nullary.iter().enumerate() {
            by_name.entry(method.name).or_insert(index as u16);
        }
        for (offset, (name, _)) in EXTRA_NULLARY.iter().enumerate() {
            by_name
                .entry(*name)
                .or_insert((nullary.len() + offset) as u16);
        }

        StyleTable { nullary, by_name }
    })
}

/// The reflected no-argument style names, for diagnostics and tests.
#[cfg(test)]
pub fn nullary_count() -> usize {
    table().nullary.len() + EXTRA_NULLARY.len()
}

/// Index of a no-argument style method, if `name` is one.
pub fn nullary_index(name: &str) -> Option<u16> {
    table().by_name.get(name).copied()
}

/// Whether `name` is a style method that takes one argument.
pub fn param_style_name(name: &str) -> Option<&'static str> {
    PARAM_STYLES
        .iter()
        .copied()
        .find(|candidate| *candidate == name)
}

/// Applies a no-argument style method by index. An out-of-range index is inert.
pub fn apply_nullary(index: u16, refinement: StyleRefinement) -> StyleRefinement {
    let table = table();
    if let Some(method) = table.nullary.get(index as usize) {
        return method.invoke(refinement);
    }
    match EXTRA_NULLARY.get(index as usize - table.nullary.len()) {
        Some((_, apply)) => apply(refinement),
        None => refinement,
    }
}

/// Applies a no-argument style method by name, or returns `None` when unknown.
pub fn apply_nullary_name(name: &str, refinement: StyleRefinement) -> Option<StyleRefinement> {
    let index = nullary_index(name)?;
    Some(apply_nullary(index, refinement))
}

/// Applies a style method that takes one argument.
pub fn apply_param(
    name: &str,
    arg: &StyleArg,
    refinement: StyleRefinement,
) -> Result<StyleRefinement, String> {
    // A name that is neither reflected nor hand-bound is an error, not a silent
    // no-op; the `PARAM_STYLES` table is what makes that check total.
    if param_style_name(name).is_none() {
        return Err(format!("unknown style method `{name}`"));
    }

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

        other => return Err(format!("unknown style method `{other}`")),
    })
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

    #[test]
    fn the_reflection_table_is_populated() {
        // Guards the `inspector` feature: without it this table is empty and
        // every no-argument style silently stops working.
        assert!(
            nullary_count() > 100,
            "expected hundreds of reflected style methods, got {}",
            nullary_count()
        );
    }

    #[test]
    fn a_nullary_name_is_applied_through_reflection() {
        let styled = apply_nullary_name("items_center", StyleRefinement::default())
            .expect("items_center is reflected");
        assert_eq!(styled.align_items, Some(gpui::AlignItems::Center));
    }

    #[test]
    fn an_unknown_nullary_name_is_none() {
        assert!(apply_nullary_name("not_a_style_at_all", StyleRefinement::default()).is_none());
    }

    #[test]
    fn a_bare_number_is_pixels_and_a_percent_string_is_relative() {
        let padded = apply_param("p", &StyleArg::Number(12.), StyleRefinement::default()).unwrap();
        assert_eq!(padded.padding.top, Some(px(12.).into()));

        let wide = apply_param(
            "w",
            &StyleArg::String("50%".into()),
            StyleRefinement::default(),
        )
        .unwrap();
        assert_eq!(wide.size.width, Some(Length::Definite(relative(0.5))));
    }

    #[test]
    fn bg_sets_a_background_from_a_hex_literal() {
        let styled = apply_param(
            "bg",
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
            "font_weight",
            &StyleArg::Number(99.),
            StyleRefinement::default()
        )
        .is_err());
    }

    #[test]
    fn an_unknown_parametric_name_is_an_error() {
        assert!(apply_param(
            "not_a_style",
            &StyleArg::Number(1.),
            StyleRefinement::default()
        )
        .is_err());
    }

    #[test]
    fn param_styles_are_disjoint_from_reflection() {
        for name in PARAM_STYLES {
            assert!(
                nullary_index(name).is_none(),
                "`{name}` is both reflected and hand-bound"
            );
        }
    }
}
