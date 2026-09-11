//! `Text` materialization: node data is the content, styled through a shell
//! wrapper div so the shared style surface applies uniformly.

use gpui::prelude::*;
use gpui::{div, AnyElement, IntoElement};
use gpui_component::text::Text;

use crate::components::apply_style;
use crate::registry::MaterializeContext;

pub fn materialize(ctx: &mut MaterializeContext<'_>) -> Result<AnyElement, String> {
    let content = ctx.node.data.clone();
    let element = apply_style(div(), ctx.node);
    Ok(element.child(Text::from(content)).into_any_element())
}

#[cfg(test)]
mod tests {
    use crate::components::build_refinement;
    use crate::schema::COMPONENT_TEXT;
    use crate::snapshot::{Node, Op};

    #[test]
    fn content_is_the_node_data_and_styles_fold_into_the_refinement() {
        let node = Node {
            component: COMPONENT_TEXT,
            data: "hello".into(),
            ops: vec![Op::StyleColor("text_color".into(), "#112233".into())],
            children: Vec::new(),
        };
        assert_eq!(node.data, "hello");
        let expected: gpui::Fill = gpui::Hsla::from(gpui::rgba(0x112233ff)).into();
        assert_eq!(build_refinement(&node).text.color, Some(expected.into()));
    }
}
