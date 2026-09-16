//! `Select`, adapted from `component-shell`'s `delegate_select/mod.rs`.
//!
//! A retained single-value select backed by an immutable row snapshot (P4) of
//! `id`, `label`, and an optional `disabled` flag. The custom `render_row`
//! element callback is not exposed; rows render their `label`. Selection is
//! reported through `on_select` with the row id.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext as _, Entity, IntoElement, ParentElement as _, Refineable as _,
    SharedString, Styled as _, Subscription, Window,
};
use gpui_component::{
    searchable_list::{SearchableListDelegate, SearchableListItem},
    select::{Select, SelectEvent, SelectState},
    IndexPath,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, ElementCallback, MaterializeRequest,
    MethodDescriptor, Row,
};

#[derive(Clone)]
struct SelectPayload {
    id: String,
    rows: ComponentArgument,
    on_select: ComponentArgument,
}

#[derive(Clone)]
enum SelectOp {
    Placeholder(String),
    MenuWidth(f32),
    Disabled(bool),
    RenderRow(ComponentArgument),
}

#[derive(Clone)]
struct Item {
    id: String,
    title: SharedString,
    disabled: bool,
    fields: Vec<String>,
    renderer: Option<ElementCallback>,
}

impl SearchableListItem for Item {
    type Value = String;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn render(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        match &self.renderer {
            Some(callback) => callback
                .build(&self.fields, window, cx)
                .unwrap_or_else(|error| {
                    gpui::div()
                        .child(format!("Failed to render Select row: {error}"))
                        .into_any_element()
                }),
            None => gpui::div().child(self.title.clone()).into_any_element(),
        }
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }

    fn disabled(&self) -> bool {
        self.disabled
    }
}

fn parse_items(rows: &[Row], renderer: Option<ElementCallback>) -> Vec<Item> {
    rows.iter()
        .enumerate()
        .map(|(index, fields)| {
            let mut parts = fields.iter().cloned();
            let id = parts
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| format!("row-{index}"));
            let title = parts.next().unwrap_or_else(|| id.clone());
            let disabled = matches!(parts.next().as_deref(), Some("true" | "1"));
            Item {
                id,
                title: title.into(),
                disabled,
                fields: fields.clone(),
                renderer: renderer.clone(),
            }
        })
        .collect()
}

#[derive(Clone)]
struct Delegate {
    items: Vec<Item>,
}

impl SearchableListDelegate for Delegate {
    type Item = Item;

    fn items_count(&self, section: usize) -> usize {
        usize::from(section == 0) * self.items.len()
    }

    fn item(&self, ix: IndexPath) -> Option<&Self::Item> {
        (ix.section == 0).then(|| self.items.get(ix.row)).flatten()
    }

    fn position<V>(&self, value: &V) -> Option<IndexPath>
    where
        Self::Item: SearchableListItem<Value = V>,
        V: PartialEq,
    {
        self.items
            .iter()
            .position(|item| item.value() == value)
            .map(IndexPath::new)
    }
}

struct Host {
    state: Entity<SelectState<Delegate>>,
    callback: Rc<RefCell<ComponentCallback>>,
    _selection: Subscription,
}

struct SelectMaterializer;

impl ComponentMaterializer for SelectMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<SelectPayload>()
            .ok_or_else(|| "Select received an incompatible payload".to_string())?
            .clone();
        if request.children_len() != 0 {
            return Err("Select does not accept children".to_string());
        }
        let items = parse_items(
            &request.resolve_rows(&payload.rows)?,
            request
                .methods()
                .find_map(|method| match method.payload().downcast_ref::<SelectOp>() {
                    Some(SelectOp::RenderRow(argument)) => Some(argument.clone()),
                    Some(_) | None => None,
                })
                .map(|argument| request.resolve_element_callback(&argument))
                .transpose()?,
        );
        let on_select = request.resolve_callback(&payload.on_select)?;
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<SelectOp>().cloned())
            .collect::<Vec<_>>();
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-select:{}", payload.id)));
        let host = request.use_keyed_state(key, {
            let initial = Delegate {
                items: items.clone(),
            };
            let on_select = on_select.clone();
            move |window, cx| {
                let state = cx.new(|cx| SelectState::new(initial, None, window, cx));
                let callback = Rc::new(RefCell::new(on_select));
                let event_callback = callback.clone();
                let selection = window.subscribe(
                    &state,
                    cx,
                    move |_, event: &SelectEvent<Delegate>, window, cx| {
                        if let SelectEvent::Confirm(Some(value)) = event {
                            let callback = event_callback.borrow().clone();
                            callback.invoke_with(
                                "Select.on_select",
                                &[ComponentCallbackArgument::String(value.clone())],
                                window,
                                cx,
                            );
                        }
                    },
                );
                Host {
                    state,
                    callback,
                    _selection: selection,
                }
            }
        });
        let state = request.with_window_app(|_, cx| host.read(cx).state.clone());
        request.update_entity(&host, |host, _| {
            *host.callback.borrow_mut() = on_select;
        });
        request.with_window_app(|window, app| {
            state.update(app, |state, cx| {
                state.set_items(Delegate { items }, window, cx);
            });
        });

        let mut select = Select::new(&state);
        for operation in operations {
            select = match operation {
                SelectOp::Placeholder(value) => select.placeholder(value),
                SelectOp::MenuWidth(value) => select.menu_width(gpui::px(value)),
                SelectOp::Disabled(value) => select.disabled(value),
                SelectOp::RenderRow(_) => select,
            };
        }
        select.style().refine(&style);
        Ok(select.into_any_element())
    }
}

fn method(
    name: &'static str,
    documentation: &'static str,
    schema: ArgumentSchema,
    make: fn(&ComponentArgument) -> Option<SelectOp>,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, schema)],
        move |args| {
            args.first()
                .and_then(make)
                .map(ComponentPayload::new)
                .ok_or_else(|| format!("Select.{name} received an invalid value"))
        },
    )
    .with_documentation(documentation)
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Select", Arc::new(SelectMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Select",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("rows", ArgumentSchema::Callback),
                        ArgumentDescriptor::new("on_select", ArgumentSchema::Callback),
                    ],
                    |arguments| match arguments {
                        [
                            ComponentArgument::String(id),
                            ComponentArgument::String(rows),
                            ComponentArgument::String(on_select),
                        ] if !id.trim().is_empty() => {
                            let rows = rows
                                .parse::<u64>()
                                .map_err(|_| "Select rows token must be a number".to_string())?;
                            let on_select = on_select.parse::<u64>().map_err(|_| {
                                "Select on_select token must be a number".to_string()
                            })?;
                            Ok(ComponentPayload::new(SelectPayload {
                                id: id.clone(),
                                rows: ComponentArgument::Callback(rows),
                                on_select: ComponentArgument::Callback(on_select),
                            }))
                        }
                        _ => Err(
                            "Select expects id, rows callback, and selection callback".into(),
                        ),
                    },
                )])
                .with_methods(vec![
                    method(
                        "placeholder",
                        "Sets the text shown while nothing is selected.",
                        ArgumentSchema::String,
                        |arg| match arg {
                            ComponentArgument::String(value) => {
                                Some(SelectOp::Placeholder(value.clone()))
                            }
                            _ => None,
                        },
                    ),
                    method(
                        "menu_width",
                        "Sets the popup menu width in pixels.",
                        ArgumentSchema::Number,
                        |arg| match arg {
                            ComponentArgument::Number(value)
                                if value.is_finite() && *value > 0.0 && *value <= f32::MAX as f64 =>
                            {
                                Some(SelectOp::MenuWidth(*value as f32))
                            }
                            _ => None,
                        },
                    ),
                    method(
                        "disabled",
                        "Disables the select.",
                        ArgumentSchema::Boolean,
                        |arg| match arg {
                            ComponentArgument::Boolean(value) => Some(SelectOp::Disabled(*value)),
                            _ => None,
                        },
                    ),
                    MethodDescriptor::new(
                        "render_row",
                        vec![ArgumentDescriptor::new("callback", ArgumentSchema::Callback)],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                SelectOp::RenderRow(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("Select.render_row(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation(
                        "Renders each option with managed code, receiving the row's fields.",
                    ),
                ])
                .with_documentation(
                    "Native retained single-value Select backed by `id\\tlabel[\\tdisabled]` rows.",
                ),
        )
        .expect("the built-in Select descriptor is valid");
}
