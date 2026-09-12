//! `DataTable`, adapted from `component-shell`'s `data_table/mod.rs`.
//!
//! A retained native table backed by an immutable row snapshot (P4). Column
//! headers are declared with `columns(keys)` as a newline-separated list; each
//! row is a tab-separated list of cell strings. The custom `render_cell`
//! element callback is not exposed; cells render their text.

use std::sync::Arc;

use gpui::{
    div, px, AnyElement, App, IntoElement, ParentElement as _, Refineable as _, SharedString,
    Styled as _,
};
use gpui_component::table::{Column, DataTable, TableDelegate, TableState};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    ElementCallback, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct DataTablePayload {
    id: String,
    rows: ComponentArgument,
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
    rows: Vec<Vec<String>>,
    render_cell: Option<ElementCallback>,
}

fn columns_from(keys: &[String], rows: &[Vec<String>]) -> Vec<Column> {
    let keys = if keys.is_empty() {
        let width = rows.iter().map(Vec::len).max().unwrap_or(0);
        (0..width).map(|index| format!("column-{index}")).collect()
    } else {
        keys.to_vec()
    };
    keys.iter()
        .map(|key| Column::new(key.clone(), key.clone()))
        .collect()
}

impl TableDelegate for Delegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
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
        match &self.render_cell {
            Some(callback) => {
                let row = self
                    .rows
                    .get(row_ix)
                    .map(|row| row.join("\t"))
                    .unwrap_or_default();
                let column = self
                    .columns
                    .get(col_ix)
                    .map(|column| column.key.to_string())
                    .unwrap_or_default();
                callback
                    .build(&[row, column], window, cx)
                    .unwrap_or_else(|error| {
                        div()
                            .child(format!("Failed to render DataTable cell: {error}"))
                            .into_any_element()
                    })
            }
            None => {
                let text = self
                    .rows
                    .get(row_ix)
                    .and_then(|row| row.get(col_ix))
                    .cloned()
                    .unwrap_or_default();
                div().child(text).into_any_element()
            }
        }
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        self.rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .unwrap_or_default()
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
        if request.children_len() != 0 {
            return Err("DataTable does not accept children".to_string());
        }
        let rows: Vec<Vec<String>> = request.resolve_rows(&payload.rows)?;
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
        let columns = columns_from(&keys, &rows);
        let render_cell = operations
            .iter()
            .find_map(|op| match op {
                DataTableOp::RenderCell(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_element_callback(&argument))
            .transpose()?;
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-table:{}", payload.id)));
        let entity = request.use_keyed_state(key, {
            let delegate = Delegate {
                columns: columns.clone(),
                rows: rows.clone(),
                render_cell: render_cell.clone(),
            };
            move |window, cx| TableState::new(delegate, window, cx)
        });
        request.with_window_app(|_, app| {
            entity.update(app, |state, cx| {
                let delegate = state.delegate_mut();
                delegate.columns = columns;
                delegate.rows = rows;
                delegate.render_cell = render_cell;
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
                        ArgumentDescriptor::new("rows", ArgumentSchema::Callback),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(rows)]
                            if !id.trim().is_empty() =>
                        {
                            let token = rows
                                .parse::<u64>()
                                .map_err(|_| "DataTable rows token must be a number".to_string())?;
                            Ok(ComponentPayload::new(DataTablePayload {
                                id: id.clone(),
                                rows: ComponentArgument::Callback(token),
                            }))
                        }
                        _ => Err("DataTable expects a non-empty id and a rows callback".into()),
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
                        "Renders each cell with managed code, receiving `[row, column]`.",
                    ),
                ])
                .with_documentation(
                    "Retained native DataTable over `id\\t...` rows with declared columns.",
                ),
        )
        .expect("the built-in DataTable descriptor is valid");
}
