//! `Scroll` and `Scrollbar`, ported from `component-shell`'s `scroll/scroll.rs`.
//!
//! A `Scroll` is a scrollable area that keeps its position in a `ScrollHandle`
//! under its own name. A `Scrollbar` pairs with a scroll area **by name** and
//! drives that shared handle. The method vocabulary matches `component-shell`
//! (`scroll_axis`, `mode`, `viewport_from_layout`); the handle is retained by
//! keyed element state rather than a registered entity, which this runtime does
//! not yet expose.

use std::sync::Arc;

use gpui::{
    div, px, AnyElement, ElementId, InteractiveElement as _, IntoElement as _, ParentElement as _,
    Refineable as _, ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _,
};
use gpui_base::{InteractiveElementExt as _, Scrollbar, ScrollbarAxis, ScrollbarMode};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct IdPayload(String);

#[derive(Clone, Copy)]
enum AxisOp {
    Axis(ScrollbarAxis),
    Mode(ScrollbarMode),
    ViewportFromLayout(bool),
}

fn resolve_ops<'a>(
    ops: impl Iterator<Item = &'a AxisOp>,
) -> (Option<ScrollbarAxis>, Option<ScrollbarMode>, bool) {
    let mut axis = None;
    let mut mode = None;
    let mut viewport_from_layout = false;
    for op in ops {
        match op {
            AxisOp::Axis(value) => axis = Some(*value),
            AxisOp::Mode(value) => mode = Some(*value),
            AxisOp::ViewportFromLayout(value) => viewport_from_layout = *value,
        }
    }
    (axis, mode, viewport_from_layout)
}

fn id_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(id)] => Ok(ComponentPayload::new(IdPayload(id.clone()))),
        _ => Err("Scroll(id) expects one string".into()),
    }
}

fn axis_method() -> MethodDescriptor {
    MethodDescriptor::new(
        // Not `axis`: `component-shell` uses `scroll_axis` because its element
        // prototype reserves the bare name. Keeping the same name keeps the two
        // surfaces identical.
        "scroll_axis",
        vec![ArgumentDescriptor::new(
            "axis",
            ArgumentSchema::Enum(&["vertical", "horizontal", "both"]),
        )],
        |args| match args {
            [ComponentArgument::Enum(value)] => match value.as_str() {
                "vertical" => Ok(ComponentPayload::new(AxisOp::Axis(ScrollbarAxis::Vertical))),
                "horizontal" => Ok(ComponentPayload::new(AxisOp::Axis(
                    ScrollbarAxis::Horizontal,
                ))),
                "both" => Ok(ComponentPayload::new(AxisOp::Axis(ScrollbarAxis::Both))),
                _ => Err("axis expects vertical, horizontal, or both".into()),
            },
            _ => Err("axis expects one enum string".into()),
        },
    )
    .with_documentation("Selects the native scroll axes.")
}

fn mode_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "mode",
        vec![ArgumentDescriptor::new(
            "mode",
            ArgumentSchema::Enum(&["scrolling", "hover", "always"]),
        )],
        |args| match args {
            [ComponentArgument::Enum(value)] => match value.as_str() {
                "scrolling" => Ok(ComponentPayload::new(AxisOp::Mode(
                    ScrollbarMode::Scrolling,
                ))),
                "hover" => Ok(ComponentPayload::new(AxisOp::Mode(ScrollbarMode::Hover))),
                "always" => Ok(ComponentPayload::new(AxisOp::Mode(ScrollbarMode::Always))),
                _ => Err("Scrollbar.mode expects scrolling, hover, or always".into()),
            },
            _ => Err("Scrollbar.mode expects one enum string".into()),
        },
    )
    .with_documentation("Sets the native scrollbar visibility policy.")
}

fn viewport_from_layout_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "viewport_from_layout",
        vec![ArgumentDescriptor::new("enabled", ArgumentSchema::Boolean)],
        |args| match args {
            [ComponentArgument::Boolean(value)] => {
                Ok(ComponentPayload::new(AxisOp::ViewportFromLayout(*value)))
            }
            _ => Err("Scrollbar.viewport_from_layout expects boolean".into()),
        },
    )
    .with_documentation("Uses this element's layout bounds as the native viewport.")
}

struct ScrollMaterializer;

impl ComponentMaterializer for ScrollMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Scroll received an incompatible payload".to_string())?
            .0
            .clone();
        let (axis, _, _) = resolve_ops(
            request
                .methods()
                .filter_map(|method| method.payload().downcast_ref::<AxisOp>()),
        );
        let axis = axis.unwrap_or(ScrollbarAxis::Vertical);
        let children = request.take_children();
        let style = request.take_style();

        let mut area = request.with_window_app(|window, cx| {
            let key = ElementId::Name(SharedString::from(id));
            let handle = window
                .use_keyed_state(key.clone(), cx, |_, _| ScrollHandle::default())
                .read(cx)
                .clone();
            let mut area = div()
                .id(key)
                .flex()
                .track_scroll(&handle)
                .lock_scroll_axis();
            area = match axis {
                ScrollbarAxis::Vertical => area.flex_col().overflow_y_scroll(),
                ScrollbarAxis::Horizontal => area.flex_row().overflow_x_scroll(),
                ScrollbarAxis::Both => area.overflow_scroll(),
            };
            area.children(children)
        });
        area.style().refine(&style);
        Ok(area.into_any_element())
    }
}

struct ScrollbarMaterializer;

impl ComponentMaterializer for ScrollbarMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let target = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Scrollbar received an incompatible payload".to_string())?
            .0
            .clone();
        let (axis, mode, viewport_from_layout) = resolve_ops(
            request
                .methods()
                .filter_map(|method| method.payload().downcast_ref::<AxisOp>()),
        );
        let style = request.take_style();

        Ok(request.with_window_app(|window, cx| {
            let key = ElementId::Name(SharedString::from(target));
            let handle = window
                .use_keyed_state(key, cx, |_, _| ScrollHandle::default())
                .read(cx)
                .clone();
            let mut bar = Scrollbar::new(&handle);
            if let Some(axis) = axis {
                bar = bar.axis(axis);
            }
            if let Some(mode) = mode {
                bar = bar.mode(mode);
            }
            if viewport_from_layout {
                bar = bar.viewport_from_layout();
            }

            let mut frame = div().absolute().top_0().bottom_0().right_0().w(px(12.0));
            frame.style().refine(&style);
            frame.child(bar).into_any_element()
        }))
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Scroll", Arc::new(ScrollMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Scroll",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    id_payload,
                )])
                .with_methods(vec![axis_method()])
                .with_documentation(
                    "A scrollable area; a Scrollbar of the same id drives it. Defaults to \
                     vertical scrolling and accepts ordinary children and shell style.",
                ),
        )
        .expect("the built-in Scroll descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Scrollbar", Arc::new(ScrollbarMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Scrollbar",
                    vec![ArgumentDescriptor::new("target", ArgumentSchema::String)],
                    id_payload,
                )])
                .with_methods(vec![
                    axis_method(),
                    mode_method(),
                    viewport_from_layout_method(),
                ])
                .with_documentation("A bar that drives the Scroll area sharing its id."),
        )
        .expect("the built-in Scrollbar descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_configuration_is_last_call_wins() {
        let ops = [
            AxisOp::Axis(ScrollbarAxis::Horizontal),
            AxisOp::Mode(ScrollbarMode::Hover),
            AxisOp::ViewportFromLayout(true),
            AxisOp::Axis(ScrollbarAxis::Vertical),
            AxisOp::Mode(ScrollbarMode::Always),
            AxisOp::ViewportFromLayout(false),
        ];
        let (axis, mode, viewport) = resolve_ops(ops.iter());
        assert_eq!(axis, Some(ScrollbarAxis::Vertical));
        assert_eq!(mode, Some(ScrollbarMode::Always));
        assert!(!viewport);
    }
}
