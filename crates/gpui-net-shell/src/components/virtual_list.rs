//! `VirtualList`: a virtualized list over managed data, vertical or horizontal.
//!
//! The managed host owns the items and supplies a count plus an
//! `render_item(index)` element callback; the native list renders only the
//! visible range. It also supports per-item sizes, selection, a row right-click
//! menu, and imperative scroll commands (top / bottom / index), matching the
//! index-callback shape of `DataTable`, `Tree`, `List`, and `Select`.
//!
//! Built on `gpui-base`'s two-axis virtual list, with a retained scroll handle
//! so scroll position survives re-renders.

use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, px, size, AnyElement, App, Axis, Context, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Refineable as _, Render, ScrollStrategy, SharedString, Size,
    StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_base::{h_virtual_list, v_virtual_list, VirtualListScrollHandle};
use gpui_component::menu::ContextMenuExt as _;
use gpui_component::scroll::ScrollableElement as _;

use super::context_menu::Entry;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, ElementCallback, MaterializeRequest,
    MethodDescriptor,
};
use crate::typed_child::take_typed;

#[derive(Clone)]
struct VirtualListPayload {
    id: String,
    item_count: usize,
}

#[derive(Clone)]
enum VirtualListOp {
    ItemSize(f32),
    ItemSizes(ComponentArgument),
    Axis(Axis),
    RenderItem(ComponentArgument),
    OnSelect(ComponentArgument),
    ScrollTo(usize),
    ScrollToken(u64),
}

/// A cached uniform size vector plus the inputs it was built from.
type UniformSizes = (usize, f32, Rc<Vec<Size<Pixels>>>);

struct VirtualListView {
    id: ElementId,
    axis: Axis,
    item_count: usize,
    item_size: f32,
    /// Per-item sizes when the host supplies them; otherwise a uniform size.
    item_sizes: Option<Rc<Vec<Size<Pixels>>>>,
    /// The uniform size vector, rebuilt only when the count or size changes.
    uniform_sizes: Option<UniformSizes>,
    renderer: Option<ElementCallback>,
    on_select: Option<ComponentCallback>,
    row_menu: Rc<Vec<Entry>>,
    /// A one-shot scroll request, applied when `scroll_token` changes.
    scroll_target: Option<usize>,
    scroll_token: u64,
    scroll_handle: VirtualListScrollHandle,
}

impl VirtualListView {
    fn new(id: ElementId, axis: Axis, item_count: usize, item_size: f32) -> Self {
        Self {
            id,
            axis,
            item_count,
            item_size,
            item_sizes: None,
            uniform_sizes: None,
            renderer: None,
            on_select: None,
            row_menu: Rc::new(Vec::new()),
            scroll_target: None,
            scroll_token: 0,
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }

    /// The size vector for the list, building the uniform one only when the
    /// item count or size changed since the last build.
    fn sizes(&mut self) -> Rc<Vec<Size<Pixels>>> {
        if let Some(sizes) = &self.item_sizes {
            return sizes.clone();
        }
        let stale = match &self.uniform_sizes {
            Some((count, size, _)) => *count != self.item_count || *size != self.item_size,
            None => true,
        };
        if stale {
            let built = Rc::new(vec![
                size(px(self.item_size), px(self.item_size));
                self.item_count
            ]);
            self.uniform_sizes = Some((self.item_count, self.item_size, built));
        }
        self.uniform_sizes
            .as_ref()
            .expect("uniform sizes were just built")
            .2
            .clone()
    }
}

/// Renders one visible index into an element, or a placeholder on error.
fn render_index(
    renderer: &Option<ElementCallback>,
    index: usize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    match renderer {
        Some(callback) => callback
            .build(&[index.to_string()], window, cx)
            .unwrap_or_else(|error| {
                div()
                    .child(format!("Failed to render virtual item: {error}"))
                    .into_any_element()
            }),
        None => div().child(format!("Item {index}")).into_any_element(),
    }
}

/// Wraps one item with its click handler and row context menu.
fn item_element(
    renderer: &Option<ElementCallback>,
    on_select: &Option<ComponentCallback>,
    row_menu: &Rc<Vec<Entry>>,
    horizontal: bool,
    index: usize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let content = render_index(renderer, index, window, cx);
    let mut item = div().id(("virtual-item", index)).flex_shrink_0();
    if !horizontal {
        item = item.w_full();
    }
    item = item.child(content);

    if let Some(callback) = on_select {
        let callback = callback.clone();
        item = item.on_click(move |_, window, cx| {
            callback.invoke_with(
                "VirtualList.on_select callback failed",
                &[ComponentCallbackArgument::Number(index as f64)],
                window,
                cx,
            );
        });
    }
    if !row_menu.is_empty() {
        let entries = row_menu.clone();
        return item
            .context_menu(move |mut menu, _window, _cx| {
                for entry in entries.iter() {
                    menu = menu.item(entry.clone().into_menu_item(Some(index)));
                }
                menu
            })
            .into_any_element();
    }
    item.into_any_element()
}

impl Render for VirtualListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(target) = self.scroll_target.take() {
            self.scroll_handle
                .scroll_to_item(target, ScrollStrategy::Top);
        }

        let sizes = self.sizes();
        let entity = cx.entity();
        let handle = self.scroll_handle.clone();
        let renderer = self.renderer.clone();
        let on_select = self.on_select.clone();
        let row_menu = self.row_menu.clone();

        let list = if matches!(self.axis, Axis::Horizontal) {
            let (renderer, on_select, row_menu) =
                (renderer.clone(), on_select.clone(), row_menu.clone());
            h_virtual_list(
                entity,
                self.id.clone(),
                sizes,
                move |_view: &mut VirtualListView, range: Range<usize>, window, cx| {
                    range
                        .map(|index| {
                            item_element(&renderer, &on_select, &row_menu, true, index, window, cx)
                        })
                        .collect::<Vec<_>>()
                },
            )
        } else {
            let (renderer, on_select, row_menu) =
                (renderer.clone(), on_select.clone(), row_menu.clone());
            v_virtual_list(
                entity,
                self.id.clone(),
                sizes,
                move |_view: &mut VirtualListView, range: Range<usize>, window, cx| {
                    range
                        .map(|index| {
                            item_element(&renderer, &on_select, &row_menu, false, index, window, cx)
                        })
                        .collect::<Vec<_>>()
                },
            )
        };
        let list = list.track_scroll(&handle);

        // The scrollbar tracks the same retained handle as the virtual list.
        if matches!(self.axis, Axis::Horizontal) {
            div().size_full().child(list).horizontal_scrollbar(&handle)
        } else {
            div().size_full().child(list).vertical_scrollbar(&handle)
        }
    }
}

struct VirtualListMaterializer;

impl ComponentMaterializer for VirtualListMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<VirtualListPayload>()
            .ok_or_else(|| "VirtualList received an incompatible payload".to_string())?
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<VirtualListOp>().cloned())
            .collect::<Vec<_>>();
        let item_size = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                VirtualListOp::ItemSize(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(32.0);
        let axis = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                VirtualListOp::Axis(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(Axis::Vertical);
        let renderer = operations
            .iter()
            .find_map(|op| match op {
                VirtualListOp::RenderItem(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_element_callback(&argument))
            .transpose()?;
        let on_select = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                VirtualListOp::OnSelect(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_callback(&argument))
            .transpose()?;
        let scroll = operations.iter().rev().find_map(|op| match op {
            VirtualListOp::ScrollTo(index) => Some(*index),
            _ => None,
        });
        let scroll_token = operations.iter().rev().find_map(|op| match op {
            VirtualListOp::ScrollToken(token) => Some(*token),
            _ => None,
        });
        let sizes_argument = operations.iter().find_map(|op| match op {
            VirtualListOp::ItemSizes(argument) => Some(argument.clone()),
            _ => None,
        });

        let mut request = request;
        let mut row_menu = Vec::new();
        for (name, mut element) in request.take_children_named() {
            match name {
                "ContextMenuItem" | "ContextMenuSeparator" => {
                    row_menu.push(take_typed::<Entry>(&mut element, name)?);
                }
                other => {
                    return Err(format!(
                        "VirtualList accepts only ContextMenuItem or ContextMenuSeparator \
                         children; received {other}"
                    ))
                }
            }
        }
        let item_sizes = match &sizes_argument {
            Some(argument) => Some(Rc::new(
                request
                    .resolve_rows(argument)?
                    .iter()
                    .filter_map(|row| row.first().and_then(|value| value.parse::<f32>().ok()))
                    .map(|value| size(px(value), px(value)))
                    .collect::<Vec<_>>(),
            )),
            None => None,
        };
        let style = request.take_style();
        let key = SharedString::from(format!("shell-virtual-list:{}", payload.id));
        let id = ElementId::Name(SharedString::from(format!("virtual-list:{}", payload.id)));
        let entity = request.use_keyed_state(key, move |_window, _cx| {
            VirtualListView::new(id, axis, payload.item_count, item_size)
        });
        let item_count = item_sizes
            .as_ref()
            .map_or(payload.item_count, |sizes| sizes.len());
        request.update_entity(&entity, |view, _| {
            view.item_count = item_count;
            view.item_size = item_size;
            view.item_sizes = item_sizes;
            view.axis = axis;
            view.renderer = renderer;
            view.on_select = on_select;
            view.row_menu = Rc::new(row_menu);
            if let Some(token) = scroll_token {
                if token != view.scroll_token {
                    view.scroll_token = token;
                    view.scroll_target = scroll;
                }
            }
        });

        let mut wrapper = div().size_full().child(entity);
        wrapper.style().refine(&style);
        Ok(wrapper.into_any_element())
    }
}

fn callback_method(
    name: &'static str,
    make: fn(ComponentArgument) -> VirtualListOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(
            "callback",
            ArgumentSchema::Callback,
        )],
        move |arguments| match arguments {
            [argument @ ComponentArgument::Callback(_)] => {
                Ok(ComponentPayload::new(make(argument.clone())))
            }
            _ => Err(format!("VirtualList.{name}(callback) expects a callback")),
        },
    )
    .with_documentation("Runs with the item index.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("VirtualList", Arc::new(VirtualListMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "VirtualList",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("item_count", ArgumentSchema::Number),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(item_count)]
                            if !id.trim().is_empty() =>
                        {
                            let item_count = item_count.parse::<usize>().map_err(|_| {
                                "VirtualList item_count must be a non-negative integer".to_string()
                            })?;
                            Ok(ComponentPayload::new(VirtualListPayload {
                                id: id.clone(),
                                item_count,
                            }))
                        }
                        _ => Err("VirtualList expects a non-empty id and an item_count".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "item_size",
                        vec![ArgumentDescriptor::new("pixels", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite() && *value > 0.0 =>
                            {
                                Ok(ComponentPayload::new(VirtualListOp::ItemSize(
                                    *value as f32,
                                )))
                            }
                            _ => Err("VirtualList.item_size expects positive pixels".into()),
                        },
                    )
                    .with_documentation("Sets the fixed item size along the scroll axis."),
                    MethodDescriptor::new(
                        "item_sizes",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                VirtualListOp::ItemSizes(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("VirtualList.item_sizes(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation(
                        "Supplies per-item sizes (newline-separated) for varying item sizes.",
                    ),
                    MethodDescriptor::new(
                        "axis",
                        vec![ArgumentDescriptor::new(
                            "axis",
                            ArgumentSchema::Enum(&["vertical", "horizontal"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "vertical" => {
                                    Ok(ComponentPayload::new(VirtualListOp::Axis(Axis::Vertical)))
                                }
                                "horizontal" => {
                                    Ok(ComponentPayload::new(VirtualListOp::Axis(Axis::Horizontal)))
                                }
                                _ => Err(format!("unsupported VirtualList axis `{value}`")),
                            },
                            _ => Err("VirtualList.axis expects vertical or horizontal".into()),
                        },
                    )
                    .with_documentation("Sets the scroll axis."),
                    MethodDescriptor::new(
                        "render_item",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                VirtualListOp::RenderItem(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("VirtualList.render_item(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Renders one item with managed code, receiving its index."),
                    callback_method("on_select", VirtualListOp::OnSelect),
                    MethodDescriptor::new(
                        "scroll_to",
                        vec![ArgumentDescriptor::new("index", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(index)]
                                if index.is_finite() && *index >= 0.0 && index.fract() == 0.0 =>
                            {
                                Ok(ComponentPayload::new(VirtualListOp::ScrollTo(
                                    *index as usize,
                                )))
                            }
                            _ => Err("VirtualList.scroll_to expects an index".into()),
                        },
                    )
                    .with_documentation("Sets the item index to scroll to when the token changes."),
                    MethodDescriptor::new(
                        "scroll_token",
                        vec![ArgumentDescriptor::new("token", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(token)]
                                if token.is_finite() && *token >= 0.0 =>
                            {
                                Ok(ComponentPayload::new(VirtualListOp::ScrollToken(
                                    *token as u64,
                                )))
                            }
                            _ => {
                                Err("VirtualList.scroll_token expects a non-negative token".into())
                            }
                        },
                    )
                    .with_documentation(
                        "Bumps the scroll token; a change applies the pending `scroll_to`.",
                    ),
                ])
                .with_documentation(
                    "A virtualized managed list over `item_count` items, vertical or horizontal, \
                     with selection, a row context menu, and scroll commands.",
                ),
        )
        .expect("the built-in VirtualList descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_virtual_list_registers() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        assert!(frozen
            .descriptors()
            .any(|descriptor| descriptor.name() == "VirtualList"));
    }
}
