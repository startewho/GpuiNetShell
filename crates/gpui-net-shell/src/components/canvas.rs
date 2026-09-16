//! `Canvas`: a self-painted surface with optional per-frame prepaint and
//! clickable/hoverable regions.
//!
//! A `Canvas` does not lay out its children. Each child is a paint primitive
//! (see [`crate::components::paint`]) or a `HitRegion`; the canvas resolves
//! their coordinates against its own bounds, replays the paint primitives in
//! the paint phase (first declared paints underneath), and routes mouse events
//! on the regions back to managed callbacks.

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    point, size, AnyElement, App, AvailableSpace, Bounds, DispatchPhase, Element, ElementId,
    GlobalElementId, Hitbox, HitboxBehavior, HitboxId, InspectorElementId, IntoElement, LayoutId,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Refineable as _, ScrollDelta,
    ScrollWheelEvent, SharedString, Style, StyleRefinement, Window,
};

use crate::components::paint::{HitRegionSpec, PaintCommand};
use crate::context::HostContext;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    ElementCallback, MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::take_typed;

/// Child component names a `Canvas` accepts.
const CANVAS_CHILDREN: [&str; 7] = [
    "PaintRect",
    "PaintLine",
    "PaintPath",
    "PaintGradient",
    "PaintShadow",
    "PaintImage",
    "HitRegion",
];

#[derive(Clone)]
struct CanvasPayload {
    id: String,
}

#[derive(Clone)]
enum CanvasOp {
    Clip,
    Prepaint(ComponentArgument),
    Measure(ComponentArgument),
}

/// Per-frame state for click synthesis.
#[derive(Default)]
struct ClickState {
    down: Option<HitboxId>,
}

/// Regions currently hovered, keyed by region id (hitbox ids change per frame).
#[derive(Default)]
struct HoverState {
    hovered: HashSet<SharedString>,
}

struct RegionHit {
    hitbox: Hitbox,
    spec: HitRegionSpec,
    region_id: SharedString,
}

pub(crate) struct CanvasPrepaint {
    commands: Rc<Vec<PaintCommand>>,
    hits: Vec<RegionHit>,
}

pub(crate) struct CanvasElement {
    /// Precomputed keyed-state ids, so paint does not format them per frame.
    click_key: ElementId,
    hover_key: ElementId,
    commands: Rc<Vec<PaintCommand>>,
    regions: Rc<Vec<HitRegionSpec>>,
    prepaint_callback: Option<ElementCallback>,
    measure_callback: Option<ElementCallback>,
    clip: bool,
    style: StyleRefinement,
    host: HostContext,
}

impl IntoElement for CanvasElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for CanvasElement {
    type RequestLayoutState = Style;
    type PrepaintState = CanvasPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        let layout_id = if let Some(measure) = self.measure_callback.clone() {
            window.request_measured_layout(style.clone(), move |_known, available, _window, _cx| {
                canvas_measure(&measure, available)
            })
        } else {
            window.request_layout(style.clone(), [], cx)
        };
        (layout_id, style)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Style,
        window: &mut Window,
        cx: &mut App,
    ) -> CanvasPrepaint {
        let mut commands = self.commands.clone();
        let mut regions = self.regions.clone();
        if let Some(callback) = &self.prepaint_callback {
            let arguments = [
                f32::from(bounds.origin.x).to_string(),
                f32::from(bounds.origin.y).to_string(),
                f32::from(bounds.size.width).to_string(),
                f32::from(bounds.size.height).to_string(),
            ];
            match callback.build(&arguments, window, cx) {
                Ok(mut element) => {
                    if let Some(canvas) = element.downcast_mut::<CanvasElement>() {
                        // Only a prepaint callback merges commands, so the
                        // common case keeps the shared `Rc` and copies nothing.
                        let mut merged: Vec<PaintCommand> = self.commands.iter().cloned().collect();
                        merged.extend(canvas.commands.iter().cloned());
                        commands = Rc::new(merged);
                        let mut merged_regions: Vec<HitRegionSpec> =
                            self.regions.iter().cloned().collect();
                        merged_regions.extend(canvas.regions.iter().cloned());
                        regions = Rc::new(merged_regions);
                    } else {
                        eprintln!(
                            "gpui-net-shell: canvas prepaint callback must return a Canvas of \
                             paint commands"
                        );
                    }
                }
                Err(error) => {
                    eprintln!("gpui-net-shell: canvas prepaint callback failed: {error}");
                }
            }
        }
        let mut hits = Vec::with_capacity(regions.len());
        for region in regions.iter() {
            let rect = region_bounds(region, bounds);
            let behavior = if region.block_mouse {
                HitboxBehavior::BlockMouse
            } else if region.block_scroll {
                HitboxBehavior::BlockMouseExceptScroll
            } else {
                HitboxBehavior::Normal
            };
            let hitbox = window.insert_hitbox(rect, behavior);
            hits.push(RegionHit {
                hitbox,
                spec: region.clone(),
                region_id: SharedString::from(region.id.clone()),
            });
        }
        CanvasPrepaint { commands, hits }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        style: &mut Style,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let clip = self.clip;
        let commands = &prepaint.commands;
        style.paint(bounds, window, cx, |window, cx| {
            let draw = |window: &mut Window, cx: &mut App| {
                for command in commands.iter() {
                    command.paint(bounds, window, cx);
                }
            };
            if clip {
                window.paint_layer(bounds, |window| draw(window, cx));
            } else {
                draw(window, cx);
            }
        });

        let host = self.host.clone();
        let click_key = self.click_key.clone();
        let hover_key = self.hover_key.clone();
        let click_state =
            window.use_keyed_state(click_key, cx, |_window, _cx| ClickState::default());
        let hover_state =
            window.use_keyed_state(hover_key, cx, |_window, _cx| HoverState::default());

        for hit in &prepaint.hits {
            let region_id = hit.region_id.clone();

            if let Some(cursor) = hit.spec.cursor {
                if hit.hitbox.id.is_hovered(window) {
                    window.set_cursor_style(cursor, &hit.hitbox);
                }
            }

            // Press + arm click.
            let down_hitbox = hit.hitbox.clone();
            let press = hit.spec.press;
            let region = region_id.clone();
            let host_down = host.clone();
            let state = click_state.clone();
            window.on_mouse_event(move |_event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !down_hitbox.id.is_hovered(window) {
                    return;
                }
                state.update(cx, |state, _| state.down = Some(down_hitbox.id));
                if let Some(token) = press {
                    invoke(&host_down, token, region.as_ref());
                }
            });

            // Release + click.
            let up_hitbox = hit.hitbox.clone();
            let release = hit.spec.release;
            let click_token = hit.spec.click;
            let region = region_id.clone();
            let host_up = host.clone();
            let state = click_state.clone();
            window.on_mouse_event(move |_event: &MouseUpEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || state.read(cx).down != Some(up_hitbox.id) {
                    return;
                }
                state.update(cx, |state, _| state.down = None);
                if !up_hitbox.id.is_hovered(window) {
                    return;
                }
                if let Some(token) = release {
                    invoke(&host_up, token, region.as_ref());
                }
                if let Some(token) = click_token {
                    if let Some(click) = host_up.callbacks.click {
                        // SAFETY: the managed callback copies anything it keeps.
                        unsafe {
                            let _ = click(host_up.session_id, token);
                        }
                    }
                }
                (host_up.invalidate)(cx);
            });

            // Hover enter/exit + move.
            if hit.spec.hover_enter.is_some()
                || hit.spec.hover_exit.is_some()
                || hit.spec.on_move.is_some()
            {
                let move_hitbox = hit.hitbox.clone();
                let region = region_id.clone();
                let enter = hit.spec.hover_enter;
                let exit = hit.spec.hover_exit;
                let move_token = hit.spec.on_move;
                let host_move = host.clone();
                let state = hover_state.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    let hovered = move_hitbox.id.is_hovered(window);
                    let was = state.read(cx).hovered.contains(&region);
                    if hovered && !was {
                        let id = region.clone();
                        state.update(cx, |state, _| {
                            state.hovered.insert(id);
                        });
                        if let Some(token) = enter {
                            invoke(&host_move, token, region.as_ref());
                        }
                    } else if !hovered && was {
                        let id = region.clone();
                        state.update(cx, |state, _| {
                            state.hovered.remove(&id);
                        });
                        if let Some(token) = exit {
                            invoke(&host_move, token, region.as_ref());
                        }
                    }
                    if hovered {
                        if let Some(token) = move_token {
                            let x = f32::from(event.position.x);
                            let y = f32::from(event.position.y);
                            invoke(&host_move, token, &format!("{}\t{x}\t{y}", region.as_ref()));
                        }
                    }
                });
            }

            // Scroll.
            if let Some(scroll_token) = hit.spec.scroll {
                let scroll_hitbox = hit.hitbox.clone();
                let region = region_id.clone();
                let host_scroll = host.clone();
                window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, _cx| {
                    if phase != DispatchPhase::Bubble || !scroll_hitbox.id.is_hovered(window) {
                        return;
                    }
                    let (dx, dy) = match event.delta {
                        ScrollDelta::Pixels(point) => (f32::from(point.x), f32::from(point.y)),
                        ScrollDelta::Lines(point) => (point.x, point.y),
                    };
                    invoke(
                        &host_scroll,
                        scroll_token,
                        &format!("{}\t{dx}\t{dy}", region.as_ref()),
                    );
                });
            }
        }
    }
}

fn region_bounds(region: &HitRegionSpec, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    let x = region.x.resolve(bounds.origin.x, bounds.size.width);
    let y = region.y.resolve(bounds.origin.y, bounds.size.height);
    Bounds::new(
        point(x, y),
        size(
            region.w.extent(bounds.size.width),
            region.h.extent(bounds.size.height),
        ),
    )
}

/// Asks the managed measure callback for a size, given the available space.
/// The callback returns a `Text` node whose data is `"width\theight"`.
fn canvas_measure(
    callback: &ElementCallback,
    available: gpui::Size<AvailableSpace>,
) -> gpui::Size<Pixels> {
    let definite = |space: AvailableSpace| match space {
        AvailableSpace::Definite(pixels) => f32::from(pixels),
        _ => 0.0,
    };
    let arguments = [
        definite(available.width).to_string(),
        definite(available.height).to_string(),
    ];
    callback
        .decode(&arguments)
        .ok()
        .and_then(|snapshot| {
            let node = snapshot.nodes.get(snapshot.root as usize)?;
            parse_measure(&node.data)
        })
        .unwrap_or_else(|| size(gpui::px(0.0), gpui::px(0.0)))
}

fn parse_measure(data: &str) -> Option<gpui::Size<Pixels>> {
    let mut parts = data.split('\t');
    let width = parts.next()?.trim().parse::<f32>().ok()?;
    let height = parts.next()?.trim().parse::<f32>().ok()?;
    Some(size(gpui::px(width), gpui::px(height)))
}

/// Invokes a managed `invoke`-style callback with a string payload.
fn invoke(host: &HostContext, token: u64, payload: &str) {
    if let Some(callback) = host.callbacks.invoke {
        // SAFETY: the managed callback copies anything it keeps; `payload` is
        // live for the call.
        unsafe {
            let _ = callback(
                host.session_id,
                token,
                crate::schema::CALLBACK_VALUE_STRING,
                0.0,
                payload.as_ptr(),
                payload.len() as u32,
            );
        }
    }
}

struct CanvasMaterializer;

impl ComponentMaterializer for CanvasMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<CanvasPayload>()
            .ok_or_else(|| "Canvas received an incompatible payload".to_string())?
            .id
            .clone();
        let mut clip = false;
        let mut prepaint_callback = None;
        let mut measure_callback = None;
        for method in request.methods() {
            match method.payload().downcast_ref::<CanvasOp>() {
                Some(CanvasOp::Clip) => clip = true,
                Some(CanvasOp::Prepaint(argument)) => {
                    prepaint_callback = Some(request.resolve_element_callback(argument)?);
                }
                Some(CanvasOp::Measure(argument)) => {
                    measure_callback = Some(request.resolve_element_callback(argument)?);
                }
                None => {}
            }
        }

        let mut commands = Vec::new();
        let mut regions = Vec::new();
        for (name, mut element) in request.take_children_named() {
            match name {
                "HitRegion" => regions.push(take_typed::<HitRegionSpec>(&mut element, name)?),
                other if CANVAS_CHILDREN.contains(&other) => {
                    commands.push(take_typed::<PaintCommand>(&mut element, other)?);
                }
                other => {
                    return Err(format!(
                        "Canvas accepts only paint primitives or HitRegion; received {other}"
                    ))
                }
            }
        }

        let host = request.host().clone();
        let style = request.take_style();
        let click_key = ElementId::Name(SharedString::from(format!("canvas-click:{id}")));
        let hover_key = ElementId::Name(SharedString::from(format!("canvas-hover:{id}")));
        Ok(CanvasElement {
            click_key,
            hover_key,
            commands: Rc::new(commands),
            regions: Rc::new(regions),
            prepaint_callback,
            measure_callback,
            clip,
            style,
            host,
        }
        .into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Canvas", Arc::new(CanvasMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Canvas",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(CanvasPayload { id: id.clone() }))
                        }
                        _ => Err("Canvas expects a non-empty id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new("clip", vec![], |_| {
                        Ok(ComponentPayload::new(CanvasOp::Clip))
                    })
                    .with_documentation("Clips painting to the canvas bounds."),
                    MethodDescriptor::new(
                        "prepaint",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [argument @ ComponentArgument::Callback(_)] => {
                                Ok(ComponentPayload::new(CanvasOp::Prepaint(argument.clone())))
                            }
                            _ => Err("Canvas.prepaint(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation(
                        "Runs each prepaint with the canvas bounds and returns paint commands \
                         and hit regions.",
                    ),
                    MethodDescriptor::new(
                        "measure",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [argument @ ComponentArgument::Callback(_)] => {
                                Ok(ComponentPayload::new(CanvasOp::Measure(argument.clone())))
                            }
                            _ => Err("Canvas.measure(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation(
                        "Measures the canvas at layout time; the callback returns a `Text` node \
                         whose data is `width\\theight`.",
                    ),
                ])
                .with_documentation(
                    "A self-painted surface. Its children are paint primitives and hit regions, \
                     painted in declaration order against the canvas bounds.",
                ),
        )
        .expect("the built-in Canvas descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_canvas_registers() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        assert!(frozen
            .descriptors()
            .any(|descriptor| descriptor.name() == "Canvas"));
    }
}
