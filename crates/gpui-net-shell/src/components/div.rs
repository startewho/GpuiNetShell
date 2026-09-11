//! `Div` materialization: a plain container that carries the shared style
//! surface and accepts children.

use gpui::prelude::*;
use gpui::{div, AnyElement, IntoElement};

use crate::components::apply_style;
use crate::registry::MaterializeContext;

pub fn materialize(ctx: &mut MaterializeContext<'_>) -> Result<AnyElement, String> {
    let element = apply_style(div(), ctx.node);
    Ok(element
        .children(std::mem::take(&mut ctx.children))
        .into_any_element())
}

#[cfg(test)]
mod tests {
    use crate::components::build_refinement;
    use crate::schema::COMPONENT_DIV;
    use crate::snapshot::{Node, Op};

    #[test]
    fn containers_fold_layout_styles_into_the_refinement() {
        let node = Node {
            component: COMPONENT_DIV,
            data: String::new(),
            ops: vec![
                Op::StyleNullary("flex_col".into()),
                Op::StyleLength("p".into(), 24.0),
            ],
            children: Vec::new(),
        };
        let refinement = build_refinement(&node);
        assert_eq!(refinement.flex_direction, Some(gpui::FlexDirection::Column));
        assert_eq!(refinement.padding.top, Some(gpui::px(24.0).into()));
    }
}
