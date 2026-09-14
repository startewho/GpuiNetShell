//! Paint primitives for [`crate::components::canvas`].
//!
//! A `Canvas` consumes its children as paint commands rather than laying them
//! out. Each primitive parses its constructor data (wire length strings) and
//! methods into a [`PaintCommand`], carried to the canvas through the typed
//! child seam; the canvas resolves coordinates against its bounds and paints.
//!
//! Lengths follow the shell convention: `12px`/`4` is absolute, `50%` is a
//! fraction of the corresponding canvas dimension. Paths use a compact DSL in
//! the node data (`M/L/Q/C/A/Z`, with a trailing `f` or `%` for fractions).

use std::sync::Arc;

use gpui::{
    linear_color_stop, linear_gradient, point, px, size, transparent_black, AnyElement, App,
    Background, BorderStyle, Bounds, BoxShadow, Corners, CursorStyle, Edges, Hsla,
    IntoElement as _, PathBuilder, Pixels, Point, SharedString, TransformationMatrix, Window,
};
use gpui_component::try_parse_color;

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

/// A resolved-or-deferred length. `Px` is logical pixels; `Frac` is a fraction
/// of the corresponding canvas dimension.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Coord {
    Px(f32),
    Frac(f32),
}

impl Coord {
    pub(crate) fn parse(raw: &str) -> Result<Self, String> {
        let raw = raw.trim();
        if let Some(percent) = raw.strip_suffix('%') {
            return percent
                .parse::<f32>()
                .map(|value| Self::Frac(value / 100.0))
                .map_err(|_| format!("invalid paint length `{raw}`"));
        }
        if let Some(pixels) = raw.strip_suffix("px") {
            return pixels
                .parse::<f32>()
                .map(Self::Px)
                .map_err(|_| format!("invalid paint length `{raw}`"));
        }
        raw.parse::<f32>()
            .map(Self::Px)
            .map_err(|_| format!("invalid paint length `{raw}`"))
    }

    pub(crate) fn resolve(self, origin: Pixels, extent: Pixels) -> Pixels {
        origin
            + match self {
                Self::Px(value) => px(value),
                Self::Frac(fraction) => extent * fraction,
            }
    }

    pub(crate) fn extent(self, extent: Pixels) -> Pixels {
        match self {
            Self::Px(value) => px(value),
            Self::Frac(fraction) => extent * fraction,
        }
    }
}

/// One drawable operation, resolved against the canvas bounds when painted.
#[derive(Clone, Debug)]
pub(crate) enum PaintCommand {
    Rect {
        x: Coord,
        y: Coord,
        w: Coord,
        h: Coord,
        background: Option<Background>,
        border_width: f32,
        border_color: Option<Hsla>,
        radius: f32,
    },
    Line {
        x1: Coord,
        y1: Coord,
        x2: Coord,
        y2: Coord,
        width: f32,
        color: Hsla,
        dash: Option<(f32, f32)>,
    },
    Path {
        dsl: String,
        fill: Option<Hsla>,
        stroke: Option<f32>,
        color: Option<Hsla>,
        dash: Option<(f32, f32)>,
    },
    Gradient {
        x: Coord,
        y: Coord,
        w: Coord,
        h: Coord,
        angle: f32,
        from: Hsla,
        to: Hsla,
        radius: f32,
    },
    Shadow {
        x: Coord,
        y: Coord,
        w: Coord,
        h: Coord,
        offset_x: f32,
        offset_y: f32,
        blur: f32,
        color: Hsla,
        inset: bool,
        radius: f32,
    },
    Image {
        x: Coord,
        y: Coord,
        w: Coord,
        h: Coord,
        source: String,
        tint: Hsla,
    },
}

impl PaintCommand {
    /// Paints this command into `window`, resolving lengths against `bounds`.
    pub(crate) fn paint(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let (origin, extent) = (bounds.origin, bounds.size);
        match self {
            Self::Rect {
                x,
                y,
                w,
                h,
                background,
                border_width,
                border_color,
                radius,
            } => {
                let rect = Bounds::new(
                    point(
                        x.resolve(origin.x, extent.width),
                        y.resolve(origin.y, extent.height),
                    ),
                    size(w.extent(extent.width), h.extent(extent.height)),
                );
                window.paint_quad(gpui::quad(
                    rect,
                    Corners::all(px(*radius)),
                    background.unwrap_or_else(|| transparent_black().into()),
                    Edges::all(px(*border_width)),
                    border_color.unwrap_or_else(transparent_black),
                    BorderStyle::default(),
                ));
            }
            Self::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                color,
                dash,
            } => {
                let from = point(
                    x1.resolve(origin.x, extent.width),
                    y1.resolve(origin.y, extent.height),
                );
                let to = point(
                    x2.resolve(origin.x, extent.width),
                    y2.resolve(origin.y, extent.height),
                );
                paint_stroke(
                    &[Seg::Move(from), Seg::Line(to)],
                    *width,
                    *color,
                    *dash,
                    window,
                );
            }
            Self::Path {
                dsl,
                fill,
                stroke,
                color,
                dash,
            } => {
                let Ok(segments) = parse_path(dsl, bounds) else {
                    return;
                };
                if let Some(color) = fill {
                    paint_fill(&segments, *color, window);
                }
                if let Some(width) = stroke {
                    paint_stroke(
                        &segments,
                        *width,
                        color.unwrap_or_else(transparent_black),
                        *dash,
                        window,
                    );
                }
            }
            Self::Gradient {
                x,
                y,
                w,
                h,
                angle,
                from,
                to,
                radius,
            } => {
                let rect = Bounds::new(
                    point(
                        x.resolve(origin.x, extent.width),
                        y.resolve(origin.y, extent.height),
                    ),
                    size(w.extent(extent.width), h.extent(extent.height)),
                );
                window.paint_quad(gpui::quad(
                    rect,
                    Corners::all(px(*radius)),
                    linear_gradient(
                        *angle,
                        linear_color_stop(*from, 0.0),
                        linear_color_stop(*to, 1.0),
                    ),
                    Edges::all(px(0.0)),
                    transparent_black(),
                    BorderStyle::default(),
                ));
            }
            Self::Shadow {
                x,
                y,
                w,
                h,
                offset_x,
                offset_y,
                blur,
                color,
                inset,
                radius,
            } => {
                let rect = Bounds::new(
                    point(
                        x.resolve(origin.x, extent.width),
                        y.resolve(origin.y, extent.height),
                    ),
                    size(w.extent(extent.width), h.extent(extent.height)),
                );
                let shadow = BoxShadow {
                    color: *color,
                    offset: point(px(*offset_x), px(*offset_y)),
                    blur_radius: px(*blur),
                    spread_radius: px(0.0),
                    inset: *inset,
                };
                let radii = Corners::all(px(*radius));
                if *inset {
                    window.paint_inset_shadows(rect, radii, &[shadow]);
                } else {
                    window.paint_drop_shadows(rect, radii, &[shadow]);
                }
            }
            Self::Image {
                x,
                y,
                w,
                h,
                source,
                tint,
            } => {
                let rect = Bounds::new(
                    point(
                        x.resolve(origin.x, extent.width),
                        y.resolve(origin.y, extent.height),
                    ),
                    size(w.extent(extent.width), h.extent(extent.height)),
                );
                let _ = window.paint_svg(
                    rect,
                    SharedString::from(source.clone()),
                    None,
                    TransformationMatrix::unit(),
                    *tint,
                    cx,
                );
            }
        }
    }
}

/// One parsed path segment, in absolute canvas coordinates.
#[derive(Clone, Copy)]
enum Seg {
    Move(Point<Pixels>),
    Line(Point<Pixels>),
    Quad(Point<Pixels>, Point<Pixels>),
    Cubic(Point<Pixels>, Point<Pixels>, Point<Pixels>),
    Arc {
        radii: Point<Pixels>,
        rotation: f32,
        large: bool,
        sweep: bool,
        to: Point<Pixels>,
    },
    Close,
}

fn feed(builder: &mut PathBuilder, segments: &[Seg]) {
    for segment in segments {
        match segment {
            Seg::Move(to) => builder.move_to(*to),
            Seg::Line(to) => builder.line_to(*to),
            Seg::Quad(control, to) => builder.curve_to(*to, *control),
            Seg::Cubic(control_a, control_b, to) => {
                builder.cubic_bezier_to(*to, *control_a, *control_b)
            }
            Seg::Arc {
                radii,
                rotation,
                large,
                sweep,
                to,
            } => builder.arc_to(*radii, px(*rotation), *large, *sweep, *to),
            Seg::Close => builder.close(),
        }
    }
}

fn paint_stroke(
    segments: &[Seg],
    width: f32,
    color: Hsla,
    dash: Option<(f32, f32)>,
    window: &mut Window,
) {
    let mut builder = PathBuilder::stroke(px(width));
    if let Some((on, off)) = dash {
        builder = builder.dash_array(&[px(on), px(off)]);
    }
    feed(&mut builder, segments);
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn paint_fill(segments: &[Seg], color: Hsla, window: &mut Window) {
    let mut builder = PathBuilder::fill();
    feed(&mut builder, segments);
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

/// Parses the path DSL into absolute coordinates using `bounds`.
fn parse_path(dsl: &str, bounds: Bounds<Pixels>) -> Result<Vec<Seg>, String> {
    let width = bounds.size.width;
    let height = bounds.size.height;
    let ox = bounds.origin.x;
    let oy = bounds.origin.y;
    let tokens = dsl.split_whitespace().collect::<Vec<_>>();
    let mut index = 0;
    let mut segments = Vec::new();
    while index < tokens.len() {
        let command = take(&tokens, &mut index)?;
        match command {
            "M" => segments.push(Seg::Move(point(
                path_coord(take(&tokens, &mut index)?, ox, width)?,
                path_coord(take(&tokens, &mut index)?, oy, height)?,
            ))),
            "L" => segments.push(Seg::Line(point(
                path_coord(take(&tokens, &mut index)?, ox, width)?,
                path_coord(take(&tokens, &mut index)?, oy, height)?,
            ))),
            "Q" => {
                let control = point(
                    path_coord(take(&tokens, &mut index)?, ox, width)?,
                    path_coord(take(&tokens, &mut index)?, oy, height)?,
                );
                let to = point(
                    path_coord(take(&tokens, &mut index)?, ox, width)?,
                    path_coord(take(&tokens, &mut index)?, oy, height)?,
                );
                segments.push(Seg::Quad(control, to));
            }
            "C" => {
                let a = point(
                    path_coord(take(&tokens, &mut index)?, ox, width)?,
                    path_coord(take(&tokens, &mut index)?, oy, height)?,
                );
                let b = point(
                    path_coord(take(&tokens, &mut index)?, ox, width)?,
                    path_coord(take(&tokens, &mut index)?, oy, height)?,
                );
                let to = point(
                    path_coord(take(&tokens, &mut index)?, ox, width)?,
                    path_coord(take(&tokens, &mut index)?, oy, height)?,
                );
                segments.push(Seg::Cubic(a, b, to));
            }
            "A" => {
                let radii = point(
                    path_coord(take(&tokens, &mut index)?, px(0.0), width)?,
                    path_coord(take(&tokens, &mut index)?, px(0.0), height)?,
                );
                let rotation = take(&tokens, &mut index)?
                    .parse::<f32>()
                    .map_err(|_| "invalid path number".to_string())?;
                let large = take(&tokens, &mut index)?
                    .parse::<f32>()
                    .map_err(|_| "invalid path number".to_string())?
                    != 0.0;
                let sweep = take(&tokens, &mut index)?
                    .parse::<f32>()
                    .map_err(|_| "invalid path number".to_string())?
                    != 0.0;
                let to = point(
                    path_coord(take(&tokens, &mut index)?, ox, width)?,
                    path_coord(take(&tokens, &mut index)?, oy, height)?,
                );
                segments.push(Seg::Arc {
                    radii,
                    rotation,
                    large,
                    sweep,
                    to,
                });
            }
            "Z" | "z" => segments.push(Seg::Close),
            other => return Err(format!("unknown path command `{other}`")),
        }
    }
    Ok(segments)
}

fn take<'a>(tokens: &[&'a str], index: &mut usize) -> Result<&'a str, String> {
    let value = tokens
        .get(*index)
        .copied()
        .ok_or_else(|| "path ended early".to_string())?;
    *index += 1;
    Ok(value)
}

fn path_coord(raw: &str, origin: Pixels, extent: Pixels) -> Result<Pixels, String> {
    let coord = if let Some(fraction) = raw.strip_suffix(['f', 'F']) {
        Coord::Frac(
            fraction
                .parse::<f32>()
                .map_err(|_| format!("invalid path length `{raw}`"))?,
        )
    } else if let Some(percent) = raw.strip_suffix('%') {
        Coord::Frac(
            percent
                .parse::<f32>()
                .map_err(|_| format!("invalid path length `{raw}`"))?
                / 100.0,
        )
    } else {
        Coord::Px(
            raw.parse::<f32>()
                .map_err(|_| format!("invalid path length `{raw}`"))?,
        )
    };
    Ok(coord.resolve(origin, extent))
}

/// The decoded method operations a paint primitive can carry.
#[derive(Clone)]
enum PaintOp {
    Fill(Hsla),
    Stroke(f32),
    Color(Hsla),
    Radius(f32),
    Dash(f32, f32),
    Inset(bool),
}

/// The raw constructor data of a paint primitive (wire length strings).
#[derive(Clone)]
struct PaintArgs(Vec<String>);

fn args(request: &MaterializeRequest<'_>, arity: usize) -> Result<Vec<String>, String> {
    let args = request
        .payload()
        .downcast_ref::<PaintArgs>()
        .ok_or_else(|| "paint primitive received an incompatible payload".to_string())?;
    if args.0.len() != arity {
        return Err(format!(
            "paint primitive expects {arity} arguments, got {}",
            args.0.len()
        ));
    }
    Ok(args.0.clone())
}

fn ops(request: &MaterializeRequest<'_>) -> Vec<PaintOp> {
    request
        .methods()
        .filter_map(|method| method.payload().downcast_ref::<PaintOp>().cloned())
        .collect()
}

fn coord_arg(raw: &str) -> Result<Coord, String> {
    Coord::parse(raw)
}

fn color_argument(value: &ComponentArgument) -> Result<Hsla, String> {
    value
        .as_str()
        .and_then(|text| try_parse_color(text).ok())
        .ok_or_else(|| "expected a color".to_string())
}

// -- Method descriptors -----------------------------------------------------

fn color_method(
    name: &'static str,
    make: fn(Hsla) -> PaintOp,
    docs: &'static str,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new("color", ArgumentSchema::String)],
        move |arguments| match arguments {
            [value] => color_argument(value)
                .map(|color| ComponentPayload::new(make(color)))
                .map_err(|error| format!("{name}: {error}")),
            _ => Err(format!("{name}(color) expects one color")),
        },
    )
    .with_documentation(docs)
}

fn number_method(
    name: &'static str,
    make: fn(f32) -> PaintOp,
    docs: &'static str,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new("value", ArgumentSchema::Number)],
        move |arguments| match arguments {
            [ComponentArgument::Number(value)] if value.is_finite() => {
                Ok(ComponentPayload::new(make(*value as f32)))
            }
            _ => Err(format!("{name}(value) expects a number")),
        },
    )
    .with_documentation(docs)
}

fn dash_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "dash",
        vec![ArgumentDescriptor::new("pattern", ArgumentSchema::String)],
        |arguments| match arguments {
            [value] => {
                let text = value
                    .as_str()
                    .ok_or_else(|| "dash: expected a pattern".to_string())?;
                let mut parts = text.split_whitespace();
                let on = parts
                    .next()
                    .and_then(|part| part.parse::<f32>().ok())
                    .ok_or_else(|| "dash: expected `on off`".to_string())?;
                let off = parts
                    .next()
                    .and_then(|part| part.parse::<f32>().ok())
                    .ok_or_else(|| "dash: expected `on off`".to_string())?;
                Ok(ComponentPayload::new(PaintOp::Dash(on, off)))
            }
            _ => Err("dash(pattern) expects a string".into()),
        },
    )
    .with_documentation("Sets a dash pattern, in pixels.")
}

fn inset_method() -> MethodDescriptor {
    MethodDescriptor::new(
        "inset",
        vec![ArgumentDescriptor::new("inset", ArgumentSchema::Boolean)],
        |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => {
                Ok(ComponentPayload::new(PaintOp::Inset(*value)))
            }
            [ComponentArgument::Number(value)] => {
                Ok(ComponentPayload::new(PaintOp::Inset(*value != 0.0)))
            }
            _ => Err("inset(flag) expects a boolean".into()),
        },
    )
    .with_documentation("Draws the shadow inside the bounds.")
}

fn constructor(export: &'static str, arguments: Vec<ArgumentDescriptor>) -> ConstructorDescriptor {
    ConstructorDescriptor::new(export, arguments, |values| {
        Ok(ComponentPayload::new(PaintArgs(
            values
                .iter()
                .map(|value| value.as_str().unwrap_or_default().to_string())
                .collect(),
        )))
    })
}

fn length_arg(name: &'static str) -> ArgumentDescriptor {
    ArgumentDescriptor::new(name, ArgumentSchema::String)
}

// -- Materializers ----------------------------------------------------------

struct PaintRectMaterializer;

impl ComponentMaterializer for PaintRectMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let raw = args(&request, 4)?;
        let (mut background, mut border_width, mut border_color, mut radius) =
            (None, 0.0, None, 0.0);
        for op in ops(&request) {
            match op {
                PaintOp::Fill(color) => background = Some(color.into()),
                PaintOp::Stroke(width) => border_width = width,
                PaintOp::Color(color) => border_color = Some(color),
                PaintOp::Radius(value) => radius = value,
                _ => {}
            }
        }
        let command = PaintCommand::Rect {
            x: coord_arg(&raw[0])?,
            y: coord_arg(&raw[1])?,
            w: coord_arg(&raw[2])?,
            h: coord_arg(&raw[3])?,
            background,
            border_width,
            border_color,
            radius,
        };
        Ok(Carrier::new(command).into_any_element())
    }
}

struct PaintLineMaterializer;

impl ComponentMaterializer for PaintLineMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let raw = args(&request, 4)?;
        let (mut width, mut color, mut dash) = (1.0, None, None);
        for op in ops(&request) {
            match op {
                PaintOp::Stroke(value) => width = value,
                PaintOp::Color(value) => color = Some(value),
                PaintOp::Dash(on, off) => dash = Some((on, off)),
                _ => {}
            }
        }
        let command = PaintCommand::Line {
            x1: coord_arg(&raw[0])?,
            y1: coord_arg(&raw[1])?,
            x2: coord_arg(&raw[2])?,
            y2: coord_arg(&raw[3])?,
            width,
            color: color.unwrap_or_else(transparent_black),
            dash,
        };
        Ok(Carrier::new(command).into_any_element())
    }
}

struct PaintPathMaterializer;

impl ComponentMaterializer for PaintPathMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let raw = args(&request, 1)?;
        let (mut fill, mut stroke, mut color, mut dash) = (None, None, None, None);
        for op in ops(&request) {
            match op {
                PaintOp::Fill(value) => fill = Some(value),
                PaintOp::Stroke(value) => stroke = Some(value),
                PaintOp::Color(value) => color = Some(value),
                PaintOp::Dash(on, off) => dash = Some((on, off)),
                _ => {}
            }
        }
        let command = PaintCommand::Path {
            dsl: raw[0].clone(),
            fill,
            stroke,
            color,
            dash,
        };
        Ok(Carrier::new(command).into_any_element())
    }
}

struct PaintGradientMaterializer;

impl ComponentMaterializer for PaintGradientMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let raw = args(&request, 7)?;
        let mut radius = 0.0;
        for op in ops(&request) {
            if let PaintOp::Radius(value) = op {
                radius = value;
            }
        }
        let angle = raw[4]
            .parse::<f32>()
            .map_err(|_| "gradient angle must be a number".to_string())?;
        let command = PaintCommand::Gradient {
            x: coord_arg(&raw[0])?,
            y: coord_arg(&raw[1])?,
            w: coord_arg(&raw[2])?,
            h: coord_arg(&raw[3])?,
            angle,
            from: try_parse_color(&raw[5]).map_err(|error| error.to_string())?,
            to: try_parse_color(&raw[6]).map_err(|error| error.to_string())?,
            radius,
        };
        Ok(Carrier::new(command).into_any_element())
    }
}

struct PaintShadowMaterializer;

impl ComponentMaterializer for PaintShadowMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let raw = args(&request, 8)?;
        let (mut inset, mut radius) = (false, 0.0);
        for op in ops(&request) {
            match op {
                PaintOp::Inset(value) => inset = value,
                PaintOp::Radius(value) => radius = value,
                _ => {}
            }
        }
        let command = PaintCommand::Shadow {
            x: coord_arg(&raw[0])?,
            y: coord_arg(&raw[1])?,
            w: coord_arg(&raw[2])?,
            h: coord_arg(&raw[3])?,
            offset_x: raw[4]
                .parse::<f32>()
                .map_err(|_| "shadow offset must be a number".to_string())?,
            offset_y: raw[5]
                .parse::<f32>()
                .map_err(|_| "shadow offset must be a number".to_string())?,
            blur: raw[6]
                .parse::<f32>()
                .map_err(|_| "shadow blur must be a number".to_string())?,
            color: try_parse_color(&raw[7]).map_err(|error| error.to_string())?,
            inset,
            radius,
        };
        Ok(Carrier::new(command).into_any_element())
    }
}

struct PaintImageMaterializer;

impl ComponentMaterializer for PaintImageMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let raw = args(&request, 5)?;
        let mut tint = None;
        for op in ops(&request) {
            if let PaintOp::Color(color) = op {
                tint = Some(color);
            }
        }
        let command = PaintCommand::Image {
            x: coord_arg(&raw[0])?,
            y: coord_arg(&raw[1])?,
            w: coord_arg(&raw[2])?,
            h: coord_arg(&raw[3])?,
            source: raw[4].clone(),
            tint: tint.unwrap_or_else(gpui::white),
        };
        Ok(Carrier::new(command).into_any_element())
    }
}

/// A clickable region declared on a [`crate::components::canvas::CanvasElement`].
///
/// The canvas inserts a hitbox for it during prepaint; the region itself paints
/// nothing. Each event has its own managed callback token.
#[derive(Clone, Debug)]
pub(crate) struct HitRegionSpec {
    pub id: String,
    pub x: Coord,
    pub y: Coord,
    pub w: Coord,
    pub h: Coord,
    pub block_mouse: bool,
    pub block_scroll: bool,
    pub cursor: Option<CursorStyle>,
    pub click: Option<u64>,
    pub hover_enter: Option<u64>,
    pub hover_exit: Option<u64>,
    pub press: Option<u64>,
    pub release: Option<u64>,
    pub on_move: Option<u64>,
    pub scroll: Option<u64>,
}

#[derive(Clone)]
enum HitRegionOp {
    BlockMouse(bool),
    BlockScroll(bool),
    Cursor(CursorStyle),
    HoverEnter(u64),
    HoverExit(u64),
    Press(u64),
    Release(u64),
    Move(u64),
    Scroll(u64),
}

fn parse_cursor(name: &str) -> Option<CursorStyle> {
    Some(match name {
        "arrow" | "default" => CursorStyle::Arrow,
        "pointer" => CursorStyle::PointingHand,
        "text" => CursorStyle::IBeam,
        "crosshair" => CursorStyle::Crosshair,
        "grab" => CursorStyle::OpenHand,
        "grabbing" => CursorStyle::ClosedHand,
        "ew-resize" => CursorStyle::ResizeLeftRight,
        "ns-resize" => CursorStyle::ResizeUpDown,
        "n-resize" => CursorStyle::ResizeUp,
        "s-resize" => CursorStyle::ResizeDown,
        "w-resize" => CursorStyle::ResizeLeft,
        "e-resize" => CursorStyle::ResizeRight,
        _ => return None,
    })
}

struct HitRegionMaterializer;

impl ComponentMaterializer for HitRegionMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let raw = args(&request, 5)?;
        let mut spec = HitRegionSpec {
            id: raw[0].clone(),
            x: coord_arg(&raw[1])?,
            y: coord_arg(&raw[2])?,
            w: coord_arg(&raw[3])?,
            h: coord_arg(&raw[4])?,
            block_mouse: false,
            block_scroll: false,
            cursor: None,
            click: request.on_click(),
            hover_enter: None,
            hover_exit: None,
            press: None,
            release: None,
            on_move: None,
            scroll: None,
        };
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<HitRegionOp>())
        {
            match op {
                HitRegionOp::BlockMouse(value) => spec.block_mouse = *value,
                HitRegionOp::BlockScroll(value) => spec.block_scroll = *value,
                HitRegionOp::Cursor(value) => spec.cursor = Some(*value),
                HitRegionOp::HoverEnter(token) => spec.hover_enter = Some(*token),
                HitRegionOp::HoverExit(token) => spec.hover_exit = Some(*token),
                HitRegionOp::Press(token) => spec.press = Some(*token),
                HitRegionOp::Release(token) => spec.release = Some(*token),
                HitRegionOp::Move(token) => spec.on_move = Some(*token),
                HitRegionOp::Scroll(token) => spec.scroll = Some(*token),
            }
        }
        Ok(Carrier::new(spec).into_any_element())
    }
}

fn region_callback_method(
    name: &'static str,
    make: fn(u64) -> HitRegionOp,
    docs: &'static str,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(
            "callback",
            ArgumentSchema::Callback,
        )],
        move |arguments| match arguments {
            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(make(*token))),
            _ => Err(format!("{name}(callback) expects a callback")),
        },
    )
    .with_documentation(docs)
}

fn register_rect(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("PaintRect", Arc::new(PaintRectMaterializer))
                .with_constructors(vec![constructor(
                    "PaintRect",
                    vec![
                        length_arg("x"),
                        length_arg("y"),
                        length_arg("w"),
                        length_arg("h"),
                    ],
                )])
                .with_methods(vec![
                    color_method("fill", PaintOp::Fill, "Fills the rectangle."),
                    number_method(
                        "stroke",
                        PaintOp::Stroke,
                        "Sets the border width, in pixels.",
                    ),
                    color_method("color", PaintOp::Color, "Sets the border color."),
                    number_method(
                        "radius",
                        PaintOp::Radius,
                        "Sets the corner radius, in pixels.",
                    ),
                ])
                .with_documentation("A filled/outlined rectangle painted on a Canvas."),
        )
        .expect("the PaintRect descriptor is valid");
}

fn register_line(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("PaintLine", Arc::new(PaintLineMaterializer))
                .with_constructors(vec![constructor(
                    "PaintLine",
                    vec![
                        length_arg("x1"),
                        length_arg("y1"),
                        length_arg("x2"),
                        length_arg("y2"),
                    ],
                )])
                .with_methods(vec![
                    number_method("stroke", PaintOp::Stroke, "Sets the line width, in pixels."),
                    color_method("color", PaintOp::Color, "Sets the line color."),
                    dash_method(),
                ])
                .with_documentation("A straight line painted on a Canvas."),
        )
        .expect("the PaintLine descriptor is valid");
}

fn register_path(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("PaintPath", Arc::new(PaintPathMaterializer))
                .with_constructors(vec![constructor("PaintPath", vec![length_arg("dsl")])])
                .with_methods(vec![
                    color_method("fill", PaintOp::Fill, "Fills the path."),
                    number_method("stroke", PaintOp::Stroke, "Strokes the path, in pixels."),
                    color_method("color", PaintOp::Color, "Sets the stroke color."),
                    dash_method(),
                ])
                .with_documentation("A path painted on a Canvas, from a compact DSL."),
        )
        .expect("the PaintPath descriptor is valid");
}

fn register_gradient(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("PaintGradient", Arc::new(PaintGradientMaterializer))
                .with_constructors(vec![constructor(
                    "PaintGradient",
                    vec![
                        length_arg("x"),
                        length_arg("y"),
                        length_arg("w"),
                        length_arg("h"),
                        length_arg("angle"),
                        length_arg("from"),
                        length_arg("to"),
                    ],
                )])
                .with_methods(vec![number_method(
                    "radius",
                    PaintOp::Radius,
                    "Sets the corner radius, in pixels.",
                )])
                .with_documentation("A two-stop linear gradient rectangle painted on a Canvas."),
        )
        .expect("the PaintGradient descriptor is valid");
}

fn register_shadow(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("PaintShadow", Arc::new(PaintShadowMaterializer))
                .with_constructors(vec![constructor(
                    "PaintShadow",
                    vec![
                        length_arg("x"),
                        length_arg("y"),
                        length_arg("w"),
                        length_arg("h"),
                        length_arg("offset_x"),
                        length_arg("offset_y"),
                        length_arg("blur"),
                        length_arg("color"),
                    ],
                )])
                .with_methods(vec![
                    inset_method(),
                    number_method(
                        "radius",
                        PaintOp::Radius,
                        "Sets the corner radius, in pixels.",
                    ),
                ])
                .with_documentation("A drop/inset shadow painted on a Canvas."),
        )
        .expect("the PaintShadow descriptor is valid");
}

fn register_hit_region(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("HitRegion", Arc::new(HitRegionMaterializer))
                .with_constructors(vec![constructor(
                    "HitRegion",
                    vec![
                        length_arg("id"),
                        length_arg("x"),
                        length_arg("y"),
                        length_arg("w"),
                        length_arg("h"),
                    ],
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "block_mouse",
                        vec![ArgumentDescriptor::new("block", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(HitRegionOp::BlockMouse(*value)))
                            }
                            [ComponentArgument::Number(value)] => Ok(ComponentPayload::new(
                                HitRegionOp::BlockMouse(*value != 0.0),
                            )),
                            _ => Err("block_mouse(flag) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Blocks mouse events from reaching regions beneath."),
                    MethodDescriptor::new(
                        "block_mouse_except_scroll",
                        vec![ArgumentDescriptor::new("block", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(HitRegionOp::BlockScroll(*value)))
                            }
                            [ComponentArgument::Number(value)] => Ok(ComponentPayload::new(
                                HitRegionOp::BlockScroll(*value != 0.0),
                            )),
                            _ => Err("block_mouse_except_scroll(flag) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Blocks mouse events but allows scroll to pass through."),
                    MethodDescriptor::new(
                        "cursor",
                        vec![ArgumentDescriptor::new(
                            "cursor",
                            ArgumentSchema::Enum(&[
                                "default",
                                "pointer",
                                "text",
                                "crosshair",
                                "grab",
                                "grabbing",
                                "ew-resize",
                                "ns-resize",
                            ]),
                        )],
                        |arguments| match arguments {
                            [value] => value
                                .as_str()
                                .and_then(parse_cursor)
                                .map(|cursor| ComponentPayload::new(HitRegionOp::Cursor(cursor)))
                                .ok_or_else(|| "cursor: unknown cursor name".to_string()),
                            _ => Err("cursor(name) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the cursor while hovering the region."),
                    region_callback_method(
                        "on_hover_enter",
                        HitRegionOp::HoverEnter,
                        "Runs when the pointer enters the region.",
                    ),
                    region_callback_method(
                        "on_hover_exit",
                        HitRegionOp::HoverExit,
                        "Runs when the pointer leaves the region.",
                    ),
                    region_callback_method(
                        "on_press",
                        HitRegionOp::Press,
                        "Runs when the region is pressed.",
                    ),
                    region_callback_method(
                        "on_release",
                        HitRegionOp::Release,
                        "Runs when the region is released.",
                    ),
                    region_callback_method(
                        "on_move",
                        HitRegionOp::Move,
                        "Runs as the pointer moves over the region, with local coordinates.",
                    ),
                    region_callback_method(
                        "on_scroll",
                        HitRegionOp::Scroll,
                        "Runs when the region is scrolled, with the scroll delta.",
                    ),
                ])
                .with_documentation("A clickable region on a Canvas."),
        )
        .expect("the HitRegion descriptor is valid");
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    register_rect(registry);
    register_line(registry);
    register_path(registry);
    register_gradient(registry);
    register_shadow(registry);
    register_hit_region(registry);
}

/// Registers `PaintImage`. Kept separate so it takes the last component id
/// without shifting the earlier paint primitives.
pub(super) fn register_image(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("PaintImage", Arc::new(PaintImageMaterializer))
                .with_constructors(vec![constructor(
                    "PaintImage",
                    vec![
                        length_arg("x"),
                        length_arg("y"),
                        length_arg("w"),
                        length_arg("h"),
                        length_arg("source"),
                    ],
                )])
                .with_methods(vec![color_method("tint", PaintOp::Color, "Tints the SVG.")])
                .with_documentation("An SVG painted on a Canvas."),
        )
        .expect("the PaintImage descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Bounds;

    fn bounds() -> Bounds<Pixels> {
        Bounds::new(point(px(0.0), px(0.0)), size(px(100.0), px(50.0)))
    }

    #[test]
    fn lengths_resolve_pixels_and_fractions() {
        assert_eq!(
            coord_arg("12px").unwrap().resolve(px(5.0), px(100.0)),
            px(17.0)
        );
        assert_eq!(
            coord_arg("50%").unwrap().resolve(px(5.0), px(100.0)),
            px(55.0)
        );
        assert!(coord_arg("nonsense").is_err());
    }

    #[test]
    fn paths_parse_absolute_and_fractional_coordinates() {
        let segments = parse_path("M 0 0 L 50% 1f Q 10 10 20 0 Z", bounds()).unwrap();
        assert_eq!(segments.len(), 4);
    }

    #[test]
    fn an_unknown_path_command_is_an_error() {
        assert!(parse_path("M 0 0 X 1 1", bounds()).is_err());
    }
}
