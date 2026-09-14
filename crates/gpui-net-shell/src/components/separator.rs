//! `Separator`, ported from `component-shell`'s `separator.rs`.
//!
//! A horizontal or vertical, solid or dashed line. The orientation/dash variant
//! is chosen by constructor name; `label` and `color` refine it.

use std::sync::Arc;

use gpui::{
    canvas, fill, point, px, size, AnyElement, App, Axis, Bounds, Hsla, IntoElement, Pixels,
    Refineable as _, Styled, Window,
};
use gpui_component::{separator::Separator, try_parse_color, ActiveTheme as _};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone, Copy)]
enum SeparatorPayload {
    Horizontal,
    Vertical,
    HorizontalDashed,
    VerticalDashed,
}

#[derive(Clone)]
enum SeparatorOp {
    Label(String),
    Color(Hsla),
    Dashed,
}

impl SeparatorPayload {
    fn into_component(self) -> Separator {
        match self {
            Self::Horizontal => Separator::horizontal(),
            Self::Vertical => Separator::vertical(),
            Self::HorizontalDashed => Separator::horizontal_dashed(),
            Self::VerticalDashed => Separator::vertical_dashed(),
        }
    }
}

struct SeparatorMaterializer;

impl SeparatorMaterializer {
    fn component<'a>(
        payload: &ComponentPayload,
        operations: impl IntoIterator<Item = &'a SeparatorOp>,
    ) -> Result<Separator, String> {
        let payload = payload
            .downcast_ref::<SeparatorPayload>()
            .ok_or_else(|| "Separator received an incompatible payload".to_string())?;
        Ok(operations.into_iter().fold(
            payload.to_owned().into_component(),
            |component, operation| match operation {
                SeparatorOp::Label(label) => component.label(label.clone()),
                SeparatorOp::Color(color) => component.color(*color),
                SeparatorOp::Dashed => component.dashed(),
            },
        ))
    }
}

impl ComponentMaterializer for SeparatorMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<SeparatorPayload>()
            .ok_or_else(|| "Separator received an incompatible payload".to_string())?
            .to_owned();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<SeparatorOp>().cloned())
            .collect::<Vec<_>>();
        let dashed = matches!(
            payload,
            SeparatorPayload::HorizontalDashed | SeparatorPayload::VerticalDashed
        );
        let labelled = operations
            .iter()
            .any(|operation| matches!(operation, SeparatorOp::Label(_)));
        let axis = match payload {
            SeparatorPayload::Horizontal | SeparatorPayload::HorizontalDashed => Axis::Horizontal,
            SeparatorPayload::Vertical | SeparatorPayload::VerticalDashed => Axis::Vertical,
        };

        // GPUI rasterizes a dashed path into a retained path cache (~30 MB the
        // first time any dashed separator is painted). Draw dashes as quads
        // instead, which is free. Labelled dashed separators keep the component
        // so the label still overlays the line.
        if dashed && !labelled {
            let color = operations
                .iter()
                .rev()
                .find_map(|operation| match operation {
                    SeparatorOp::Color(color) => Some(*color),
                    _ => None,
                })
                .unwrap_or_else(|| request.with_window_app(|_, cx| cx.theme().border));
            let mut element = dashed_line(axis, color);
            element.style().refine(&request.take_style());
            return Ok(element.into_any_element());
        }

        let mut element = Self::component(request.payload(), operations.iter())?;
        element.style().refine(&request.take_style());
        Ok(element.into_any_element())
    }
}

/// A dashed line drawn from quads: 4px dashes with 2px gaps, 1px thick.
fn dashed_line(axis: Axis, color: Hsla) -> impl Styled + IntoElement {
    let paint = move |bounds: Bounds<Pixels>, _: (), window: &mut Window, _: &mut App| {
        let dash = px(4.0);
        let gap = px(2.0);
        let thickness = px(1.0);
        let step = dash + gap;
        match axis {
            Axis::Horizontal => {
                let y = bounds.origin.y;
                let end = bounds.origin.x + bounds.size.width;
                let mut x = bounds.origin.x;
                while x < end {
                    let length = dash.min(end - x);
                    window.paint_quad(fill(
                        Bounds::new(point(x, y), size(length, thickness)),
                        color,
                    ));
                    x += step;
                }
            }
            Axis::Vertical => {
                let x = bounds.origin.x;
                let end = bounds.origin.y + bounds.size.height;
                let mut y = bounds.origin.y;
                while y < end {
                    let length = dash.min(end - y);
                    window.paint_quad(fill(
                        Bounds::new(point(x, y), size(thickness, length)),
                        color,
                    ));
                    y += step;
                }
            }
        }
    };
    let element = canvas(move |_, _, _| {}, paint);
    match axis {
        Axis::Horizontal => element.w_full().h(px(1.0)),
        Axis::Vertical => element.w(px(1.0)).h_full(),
    }
}

fn constructor(export: &'static str, payload: SeparatorPayload) -> ConstructorDescriptor {
    ConstructorDescriptor::new(export, Vec::new(), move |_| {
        Ok(ComponentPayload::new(payload))
    })
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Separator", Arc::new(SeparatorMaterializer))
                .with_constructors(vec![
                    constructor("Separator", SeparatorPayload::Horizontal),
                    constructor("VerticalSeparator", SeparatorPayload::Vertical),
                    constructor("DashedSeparator", SeparatorPayload::HorizontalDashed),
                    constructor("VerticalDashedSeparator", SeparatorPayload::VerticalDashed),
                ])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "label",
                        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(label)] => {
                                Ok(ComponentPayload::new(SeparatorOp::Label(label.clone())))
                            }
                            _ => Err("Separator.label(label) expects a string".into()),
                        },
                    )
                    .with_documentation("Displays text centered over the separator line."),
                    MethodDescriptor::new(
                        "color",
                        vec![ArgumentDescriptor::new("color", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(color)] => try_parse_color(color)
                                .map(|color| ComponentPayload::new(SeparatorOp::Color(color)))
                                .map_err(|error| format!("invalid Separator color: {error}")),
                            _ => Err("Separator.color(color) expects a color string".into()),
                        },
                    )
                    .with_documentation("Sets the separator line color."),
                    MethodDescriptor::new("dashed", Vec::new(), |_| {
                        Ok(ComponentPayload::new(SeparatorOp::Dashed))
                    })
                    .with_documentation("Uses a dashed separator line."),
                ])
                .with_documentation("A horizontal or vertical, solid or dashed separator."),
        )
        .expect("the built-in Separator descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separator_variants_materialize_real_component_elements() {
        for payload in [
            SeparatorPayload::Horizontal,
            SeparatorPayload::Vertical,
            SeparatorPayload::HorizontalDashed,
            SeparatorPayload::VerticalDashed,
        ] {
            drop(
                SeparatorMaterializer::component(
                    &ComponentPayload::new(payload),
                    std::iter::empty(),
                )
                .unwrap()
                .into_any_element(),
            );
        }
    }

    #[test]
    fn separator_rejects_an_incompatible_payload() {
        let error =
            SeparatorMaterializer::component(&ComponentPayload::new(()), std::iter::empty())
                .err()
                .unwrap();
        assert_eq!(error, "Separator received an incompatible payload");
    }
}
