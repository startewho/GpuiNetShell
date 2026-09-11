//! Built-in component materializers and the shared style surface they compose.
//!
//! Styling is not enumerated per property. A node carries style *method calls*
//! (a GPUI method name plus an argument), which [`crate::style`] folds into a
//! [`StyleRefinement`] using the same reflection table `gpui-shell` uses. A
//! component's own behavior (identity, state, activation) stays typed.

pub mod button;
pub mod div;
pub mod text;

use gpui::prelude::*;
use gpui::{StyleRefinement, Styled};

use crate::snapshot::{Node, Op};
use crate::style::{apply_nullary_name, apply_param, StyleArg};

/// Folds `node`'s style operations into one `StyleRefinement`, in order.
pub fn build_refinement(node: &Node) -> StyleRefinement {
    let mut refinement = StyleRefinement::default();
    for op in &node.ops {
        match op {
            Op::StyleNullary(name) => {
                if let Some(next) = apply_nullary_name(name, refinement.clone()) {
                    refinement = next;
                }
            }
            Op::StyleLength(name, value) | Op::StyleNumber(name, value) => {
                let arg = StyleArg::Number(*value);
                if let Ok(next) = apply_param(name, &arg, refinement.clone()) {
                    refinement = next;
                }
            }
            Op::StyleColor(name, value) | Op::StyleString(name, value) => {
                let arg = StyleArg::String(value.clone());
                if let Ok(next) = apply_param(name, &arg, refinement.clone()) {
                    refinement = next;
                }
            }
            _ => {}
        }
    }
    refinement
}

/// Applies the node's style operations to any `Styled` element.
pub fn apply_style<E: Styled>(mut element: E, node: &Node) -> E {
    let refinement = build_refinement(node);
    element.style().refine(&refinement);
    element
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::COMPONENT_DIV;

    fn node(ops: Vec<Op>) -> Node {
        Node {
            component: COMPONENT_DIV,
            data: String::new(),
            ops,
            children: Vec::new(),
        }
    }

    #[test]
    fn a_nullary_style_name_is_folded_through_reflection() {
        let refinement = build_refinement(&node(vec![Op::StyleNullary("items_center".into())]));
        assert_eq!(refinement.align_items, Some(gpui::AlignItems::Center));
    }

    #[test]
    fn a_length_style_is_folded_as_pixels() {
        let refinement = build_refinement(&node(vec![Op::StyleLength("p".into(), 16.0)]));
        assert_eq!(refinement.padding.top, Some(gpui::px(16.0).into()));
    }

    #[test]
    fn a_color_style_is_folded_from_hex() {
        let refinement =
            build_refinement(&node(vec![Op::StyleColor("bg".into(), "#ff0000".into())]));
        let expected: gpui::Fill = gpui::Hsla::from(gpui::rgba(0xff0000ff)).into();
        assert_eq!(refinement.background, Some(expected));
    }

    #[test]
    fn unknown_style_names_are_inert() {
        let refinement = build_refinement(&node(vec![
            Op::StyleNullary("not_a_style".into()),
            Op::StyleLength("also_not".into(), 4.0),
        ]));
        assert_eq!(refinement, StyleRefinement::default());
    }
}
