//! `DataTable`, redesigned so the managed host keeps the row objects and the
//! native table asks for one row index at a time.
//!
//! Column headers come from `columns(keys)`; the row count from the constructor.
//! `render_cell` is a managed element callback receiving `[row_index, column]`,
//! so managed code indexes its own list instead of building a native snapshot.
//! `ContextMenuItem`/`ContextMenuSeparator` typed children become the row
//! right-click menu, and their callbacks receive the row index.

use std::sync::Arc;

use gpui::{
    div, px, AnyElement, App, IntoElement, ParentElement as _, Refineable as _, SharedString,
    Styled as _,
};
use gpui_component::menu::PopupMenu;
use gpui_component::table::{Column, DataTable, TableDelegate, TableState};

use super::context_menu::Entry;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    ElementCallback, MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::take_typed;

#[derive(Clone)]
struct DataTablePayload {
    id: String,
    row_count: usize,
}

#[derive(Clone)]
enum DataTableOp {
    Columns(Vec<String>),
    Stripe(bool),
    Bordered(bool),
    RenderCell(ComponentArgument),
}

#[derive(Clone)]
struct Delegate {
    columns: Vec<Column>,
    row_count: usize,
    render_cell: Option<ElementCallback>,
    row_menu: Vec<Entry>,
}

fn columns_from(keys: &[String]) -> Vec<Column> {
    keys.iter()
        .map(|key| Column::new(key.clone(), key.clone()))
        .collect()
}

impl TableDelegate for Delegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.row_count
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(callback) = &self.render_cell else {
            return div().into_any_element();
        };
        let column = self
            .columns
            .get(col_ix)
            .map(|column| column.key.to_string())
            .unwrap_or_default();
        callback
            .build(&[row_ix.to_string(), column], window, cx)
            .unwrap_or_else(|error| {
                div()
                    .child(format!("Failed to render DataTable cell: {error}"))
                    .into_any_element()
            })
    }

    fn context_menu(
        &mut self,
        row_ix: usize,
        mut menu: PopupMenu,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<TableState<Self>>,
    ) -> PopupMenu {
        for entry in self.row_menu.clone() {
            menu = menu.item(entry.into_menu_item(Some(row_ix)));
        }
        menu
    }

    fn cell_text(&self, _row_ix: usize, _col_ix: usize, _: &App) -> String {
        String::new()
    }
}

struct DataTableMaterializer;

impl ComponentMaterializer for DataTableMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<DataTablePayload>()
            .ok_or_else(|| "DataTable received an incompatible payload".to_string())?
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<DataTableOp>().cloned())
            .collect::<Vec<_>>();
        let keys = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                DataTableOp::Columns(keys) => Some(keys.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let columns = columns_from(&keys);
        let render_cell = operations
            .iter()
            .find_map(|op| match op {
                DataTableOp::RenderCell(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_element_callback(&argument))
            .transpose()?;
        let mut row_menu = Vec::new();
        for (name, mut element) in request.take_children_named() {
            match name {
                "ContextMenuItem" | "ContextMenuSeparator" => {
                    row_menu.push(take_typed::<Entry>(&mut element, name)?);
                }
                other => {
                    return Err(format!(
                        "DataTable accepts only ContextMenuItem or ContextMenuSeparator \
                         children; received {other}"
                    ))
                }
            }
        }
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-table:{}", payload.id)));
        let entity = request.use_keyed_state(key, {
            let delegate = Delegate {
                columns: columns.clone(),
                row_count: payload.row_count,
                render_cell: render_cell.clone(),
                row_menu: row_menu.clone(),
            };
            move |window, cx| TableState::new(delegate, window, cx)
        });
        request.with_window_app(|_, app| {
            entity.update(app, |state, cx| {
                let delegate = state.delegate_mut();
                delegate.columns = columns;
                delegate.row_count = payload.row_count;
                delegate.render_cell = render_cell;
                delegate.row_menu = row_menu;
                // The table caches column/row measurement in its state; without
                // a refresh it keeps rendering only the header of the delegate
                // it was created with.
                state.refresh(cx);
            });
        });

        let mut table = DataTable::new(&entity);
        for operation in operations {
            table = match operation {
                DataTableOp::Stripe(value) => table.stripe(value),
                DataTableOp::Bordered(value) => table.bordered(value),
                DataTableOp::Columns(_) | DataTableOp::RenderCell(_) => table,
            };
        }
        let mut host = div().w_full().min_h(px(160.0)).child(table);
        host.style().refine(&style);
        Ok(host.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("DataTable", Arc::new(DataTableMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "DataTable",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("row_count", ArgumentSchema::Number),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(row_count)]
                            if !id.trim().is_empty() =>
                        {
                            let row_count = row_count.parse::<usize>().map_err(|_| {
                                "DataTable row_count must be a non-negative integer".to_string()
                            })?;
                            Ok(ComponentPayload::new(DataTablePayload {
                                id: id.clone(),
                                row_count,
                            }))
                        }
                        _ => {
                            Err("DataTable expects a non-empty id and a row_count".into())
                        }
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "columns",
                        vec![ArgumentDescriptor::new("keys", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(keys)] => {
                                let keys: Vec<String> = keys
                                    .split('\n')
                                    .filter(|key| !key.trim().is_empty())
                                    .map(str::to_owned)
                                    .collect();
                                Ok(ComponentPayload::new(DataTableOp::Columns(keys)))
                            }
                            _ => Err("DataTable.columns(keys) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the column headers from a newline-separated list."),
                    MethodDescriptor::new(
                        "stripe",
                        vec![ArgumentDescriptor::new("stripe", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(DataTableOp::Stripe(*value)))
                            }
                            _ => Err("DataTable.stripe(stripe) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Alternates row backgrounds."),
                    MethodDescriptor::new(
                        "bordered",
                        vec![ArgumentDescriptor::new("bordered", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(DataTableOp::Bordered(*value)))
                            }
                            _ => Err("DataTable.bordered(bordered) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Draws cell borders."),
                    MethodDescriptor::new(
                        "render_cell",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                DataTableOp::RenderCell(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("DataTable.render_cell(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation(
                        "Renders one cell with managed code, receiving `[row_index, column]`.",
                    ),
                ])
                .with_documentation(
                    "Retained native DataTable that asks managed code for one row index at a time. \
                     ContextMenuItem/ContextMenuSeparator children become the row right-click menu.",
                ),
        )
        .expect("the built-in DataTable descriptor is valid");
}
