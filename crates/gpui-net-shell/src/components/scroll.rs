//! `Scroll` and `Scrollbar`.
//!
//! A `Scroll` is a scrollable area that keeps its position in a `ScrollHandle`
//! under its own name. A `Scrollbar` pairs with a scroll area **by name** and
//! drives that shared handle, exactly as the shell's `Scrollbar` does.

use std::sync::Arc;

use gpui::{
    div, px, AnyElement, ElementId, InteractiveElement as _, IntoElement as _, ParentElement as _,
    Refineable as _, ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _,
};
use gpui_base::{Scrollbar, ScrollbarAxis};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct IdPayload(String);

#[derive(Clone)]
struct AxisOp(bool);

fn id_payload(arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
    match arguments {
        [ComponentArgument::String(id)] => Ok(ComponentPayload::new(IdPayload(id.clone()))),
        _ => Err("Scroll(id) expects one string".into()),
    }
}

fn axis_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "axis",
        vec![ArgumentDescriptor::new("axis", ArgumentSchema::Number)],
        |args| {
            let horizontal = args
                .first()
                .and_then(ComponentArgument::as_f64)
                .unwrap_or(0.0)
                >= 0.5;
            Ok(ComponentPayload::new(AxisOp(horizontal)))
        },
    )
    .with_documentation("Sets the scroll axis: 0 is vertical, 1 is horizontal.")
}

fn scroll_axis(request: &MaterializeRequest<'_>) -> bool {
    let mut horizontal = false;
    for method in request.methods() {
        if method.name() == "axis" {
            if let Some(op) = method.payload().downcast_ref::<AxisOp>() {
                horizontal = op.0;
            }
        }
    }
    horizontal
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
        let horizontal = scroll_axis(&request);
        let style = request.take_style();
        let children = request.take_children();

        let mut element = request.with_window_app(|window, cx| {
            let key = ElementId::Name(SharedString::from(id));
            let handle = window
                .use_keyed_state(key.clone(), cx, |_, _| ScrollHandle::default())
                .read(cx)
                .clone();
            let element = div().id(key).size_full().track_scroll(&handle);
            if horizontal {
                element.overflow_x_scroll()
            } else {
                element.overflow_y_scroll()
            }
            .children(children)
        });
        element.style().refine(&style);
        Ok(element.into_any_element())
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
        let horizontal = scroll_axis(&request);
        let style = request.take_style();

        Ok(request.with_window_app(|window, cx| {
            let key = ElementId::Name(SharedString::from(target));
            let handle = window
                .use_keyed_state(key, cx, |_, _| ScrollHandle::default())
                .read(cx)
                .clone();
            let axis = if horizontal {
                ScrollbarAxis::Horizontal
            } else {
                ScrollbarAxis::Vertical
            };
            let bar = Scrollbar::new(&handle).axis(axis);

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
                .with_documentation("A scrollable area; a Scrollbar of the same id drives it."),
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
                .with_methods(vec![axis_method()])
                .with_documentation("A bar that drives the Scroll area sharing its id."),
        )
        .expect("the built-in Scrollbar descriptor is valid");
}
