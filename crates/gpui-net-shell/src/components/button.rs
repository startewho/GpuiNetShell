//! `Button` materialization, ported from `component-shell`'s control binding.
//!
//! The op set is closed: identity is the node's data string, label/tooltip are
//! string operations, variant/size/loading/compact are scalar operations, and
//! activation is a click callback token. Unknown ops are ignored here and
//! rejected during decode.

use gpui::prelude::*;
use gpui::{AnyElement, IntoElement};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::{Disableable as _, Selectable as _, Size, Sizable as _};

use crate::components::apply_style;
use crate::registry::MaterializeContext;
use crate::schema::*;
use crate::snapshot::{Node, Op};

/// The pure, window-free interpretation of a Button node.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ButtonPlan {
    pub id: String,
    pub label: Option<String>,
    pub tooltip: Option<String>,
    pub variant: u64,
    pub size: Option<u64>,
    pub loading: bool,
    pub compact: bool,
    pub disabled: bool,
    pub selected: bool,
    pub on_click: Option<u64>,
}

/// Interprets `node` into a [`ButtonPlan`]. Later declarations win, matching
/// the declarative snapshot rule.
pub fn plan(node: &Node) -> ButtonPlan {
    let mut plan = ButtonPlan {
        id: node.data.clone(),
        ..ButtonPlan::default()
    };
    for op in &node.ops {
        match op {
            Op::Label(value) => plan.label = Some(value.clone()),
            Op::Tooltip(value) => plan.tooltip = Some(value.clone()),
            Op::ButtonVariant(value) => plan.variant = *value,
            Op::ButtonSize(value) => plan.size = Some(*value),
            Op::Loading(value) => plan.loading = *value,
            Op::Compact => plan.compact = true,
            Op::Disabled(value) => plan.disabled = *value,
            Op::Selected(value) => plan.selected = *value,
            Op::OnClick(token) => plan.on_click = Some(*token),
            _ => {}
        }
    }
    plan
}

fn resolved_size(value: u64) -> Size {
    match value {
        BUTTON_SIZE_XSMALL => Size::XSmall,
        BUTTON_SIZE_SMALL => Size::Small,
        BUTTON_SIZE_LARGE => Size::Large,
        BUTTON_SIZE_MEDIUM => Size::Medium,
        _ => Size::Medium,
    }
}

fn with_variant(button: Button, variant: u64) -> Button {
    match variant {
        BUTTON_VARIANT_PRIMARY => button.primary(),
        BUTTON_VARIANT_SECONDARY => button.secondary(),
        BUTTON_VARIANT_DANGER => button.danger(),
        BUTTON_VARIANT_SUCCESS => button.success(),
        BUTTON_VARIANT_WARNING => button.warning(),
        BUTTON_VARIANT_GHOST => button.ghost(),
        BUTTON_VARIANT_LINK => button.link(),
        BUTTON_VARIANT_DEFAULT => button,
        _ => button,
    }
}

pub fn materialize(ctx: &mut MaterializeContext<'_>) -> Result<AnyElement, String> {
    let plan = plan(ctx.node);
    let mut button = Button::new(plan.id.clone())
        .loading(plan.loading)
        .disabled(plan.disabled)
        .selected(plan.selected);
    if plan.compact {
        button = button.compact();
    }
    if let Some(size) = plan.size {
        button = button.with_size(resolved_size(size));
    }
    if let Some(label) = plan.label.clone() {
        button = button.label(label);
    }
    if let Some(tooltip) = plan.tooltip.clone() {
        button = button.tooltip(tooltip);
    }
    button = with_variant(button, plan.variant);
    button = apply_style(button, ctx.node);
    button = button.children(std::mem::take(&mut ctx.children));

    if let Some(token) = plan.on_click {
        let callbacks = *ctx.callbacks;
        let session_id = ctx.session_id;
        let invalidate = ctx.invalidate.clone();
        button = button.on_click(move |_event, _window, cx| {
            if let Some(click) = callbacks.click {
                // SAFETY: managed callback; it copies anything it keeps.
                unsafe {
                    let _ = click(session_id, token);
                }
            }
            invalidate(cx);
        });
    }

    Ok(button.into_any_element())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button_node(ops: Vec<Op>) -> Node {
        Node {
            component: COMPONENT_BUTTON,
            data: "save".into(),
            ops,
            children: Vec::new(),
        }
    }

    #[test]
    fn identity_comes_from_node_data() {
        let plan = plan(&button_node(Vec::new()));
        assert_eq!(plan.id, "save");
        assert_eq!(plan.variant, BUTTON_VARIANT_DEFAULT);
        assert_eq!(plan.on_click, None);
    }

    #[test]
    fn string_and_scalar_operations_land_in_the_plan() {
        let plan = plan(&button_node(vec![
            Op::Label("First".into()),
            Op::Label("Last".into()),
            Op::ButtonVariant(BUTTON_VARIANT_DANGER),
            Op::ButtonSize(BUTTON_SIZE_LARGE),
            Op::Loading(true),
            Op::Disabled(true),
            Op::OnClick(42),
        ]));
        assert_eq!(plan.label.as_deref(), Some("Last"));
        assert_eq!(plan.variant, BUTTON_VARIANT_DANGER);
        assert_eq!(plan.size, Some(BUTTON_SIZE_LARGE));
        assert!(plan.loading);
        assert!(plan.disabled);
        assert_eq!(plan.on_click, Some(42));
    }

    #[test]
    fn every_variant_resolves_to_a_real_button_without_a_window() {
        for variant in [
            BUTTON_VARIANT_DEFAULT,
            BUTTON_VARIANT_PRIMARY,
            BUTTON_VARIANT_SECONDARY,
            BUTTON_VARIANT_DANGER,
            BUTTON_VARIANT_SUCCESS,
            BUTTON_VARIANT_WARNING,
            BUTTON_VARIANT_GHOST,
            BUTTON_VARIANT_LINK,
        ] {
            let component = with_variant(Button::new("variant"), variant);
            drop(component.into_any_element());
        }
    }
}
