//! `Canvas`: a self-painted surface with optional per-frame prepaint and
//! clickable regions.
//!
//! A `Canvas` does not lay out its children. Each child is a paint primitive
//! (see [`crate::components::paint`]) or a `HitRegion`; the canvas resolves
//! their coordinates against its own bounds, replays the paint primitives in
//! the paint phase (first declared paints underneath), and routes clicks on the
//! regions back to managed callbacks.

use std::sync::Arc;

use gpui::{
    point, size, AnyElement, App, Bounds, DispatchPhase, Element, ElementId, GlobalElementId,
    Hitbox, HitboxBehavior, HitboxId, InspectorElementId, IntoElement, LayoutId, MouseDownEvent,
    MouseUpEvent, Pixels, Refineable as _, SharedString, Style, StyleRefinement, Window,
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
const CANVAS_CHILDREN: [&str; 6] = [
    "PaintRect",
    "PaintLine",
    "PaintPath",
    "PaintGradient",
    "PaintShadow",
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
}

/// Per-frame state for click synthesis.
#[derive(Default)]
struct ClickState {
    down: Option<HitboxId>,
}

struct RegionHit {
    hitbox: Hitbox,
    click: Option<u64>,
}

pub(crate) struct CanvasPrepaint {
    commands: Vec<PaintCommand>,
    hits: Vec<RegionHit>,
}
pub(crate) struct CanvasElement {
    id: SharedString,
    commands: Vec<PaintCommand>,
    regions: Vec<HitRegionSpec>,
    prepaint_callback: Option<ElementCallback>,
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
        let layout_id = window.request_layout(style.clone(), [], cx);
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
                        commands.extend(canvas.commands.iter().cloned());
                        regions.extend(canvas.regions.iter().cloned());
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
        for region in regions {
            let rect = region_bounds(&region, bounds);
            let behavior = if region.block_mouse {
                HitboxBehavior::BlockMouse
            } else {
                HitboxBehavior::Normal
            };
            let hitbox = window.insert_hitbox(rect, behavior);
            hits.push(RegionHit {
                hitbox,
                click: region.click,
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
        style.paint(bounds, window, cx, |window, _cx| {
            let draw = |window: &mut Window| {
                for command in commands {
                    command.paint(bounds, window);
                }
            };
            if clip {
                window.paint_layer(bounds, draw);
            } else {
                draw(window);
            }
        });

        let host = self.host.clone();
        let key = ElementId::Name(SharedString::from(format!("canvas-click:{}", self.id)));
        let click_state = window.use_keyed_state(key, cx, |_window, _cx| ClickState::default());
        for hit in &prepaint.hits {
            let down_hitbox = hit.hitbox.clone();
            let state = click_state.clone();
            window.on_mouse_event(move |_event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && down_hitbox.id.is_hovered(window) {
                    state.update(cx, |state, _| state.down = Some(down_hitbox.id));
                }
            });

            let up_hitbox = hit.hitbox.clone();
            let token = hit.click;
            let state = click_state.clone();
            let host = host.clone();
            window.on_mouse_event(move |_event: &MouseUpEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || state.read(cx).down != Some(up_hitbox.id) {
                    return;
                }
                state.update(cx, |state, _| state.down = None);
                if !up_hitbox.id.is_hovered(window) {
                    return;
                }
                if let Some(token) = token {
                    if let Some(click) = host.callbacks.click {
                        // SAFETY: the managed callback copies anything it keeps.
                        unsafe {
                            let _ = click(host.session_id, token);
                        }
                    }
                }
                (host.invalidate)(cx);
            });
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
        for method in request.methods() {
            match method.payload().downcast_ref::<CanvasOp>() {
                Some(CanvasOp::Clip) => clip = true,
                Some(CanvasOp::Prepaint(argument)) => {
                    prepaint_callback = Some(request.resolve_element_callback(argument)?);
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
        Ok(CanvasElement {
            id: SharedString::from(id),
            commands,
            regions,
            prepaint_callback,
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
