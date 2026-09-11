//! `Div` materialization: a plain container that carries the shared style
//! surface and accepts children.

use gpui::prelude::*;
use gpui::{div, AnyElement, IntoElement};

use crate::components::{apply_style, style_ops};
use crate::registry::MaterializeContext;

pub fn materialize(ctx: &mut MaterializeContext<'_>) -> Result<AnyElement, String> {
    let element = apply_style(div(), &style_ops(ctx.node));
    Ok(element
        .children(std::mem::take(&mut ctx.children))
        .into_any_element())
}

#[cfg(test)]
mod tests {
    use crate::components::{style_ops, StyleOp};
    use crate::schema::*;
    use crate::snapshot::{Node, Op};

    #[test]
    fn containers_report_their_layout_operations() {
        let node = Node {
            component: COMPONENT_DIV,
            data: String::new(),
            ops: vec![
                Op {
                    code: OP_FLEX_COL,
                    a: 0,
                    b: 0,
                    data: None,
                },
                Op {
                    code: OP_PADDING,
                    a: 24.0f32.to_bits() as u64,
                    b: 0,
                    data: None,
                },
            ],
            children: Vec::new(),
        };
        assert_eq!(
            style_ops(&node),
            vec![StyleOp::FlexCol, StyleOp::Padding(24.0)]
        );
    }
}
