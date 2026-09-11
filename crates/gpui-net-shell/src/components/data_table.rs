//! `DataTable`, adapted from `component-shell`'s `data_table/mod.rs`.
//!
//! A retained native table backed by an immutable row snapshot (P4). Column
//! headers are declared with `columns(keys)` as a newline-separated list; each
//! row is a tab-separated list of cell strings. The custom `render_cell`
//! element callback is not exposed; cells render their text.

use std::sync::Arc;

use gpui::{
    div, AnyElement, App, IntoElement, ParentElement as _, Refineable as _, SharedString,
    Styled as _,
};
use gpui_component::table::{Column, DataTable, TableDelegate, TableState};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
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
}

#[derive(Clone)]
struct Delegate {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
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
        _: &mut gpui::Window,
        _: &mut gpui::Context<TableState<Self>>,
    ) -> impl IntoElement {
        let text = self
            .rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .unwrap_or_default();
        div().child(text)
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
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-table:{}", payload.id)));
        let entity = request.use_keyed_state(key, {
            let delegate = Delegate {
                columns: columns.clone(),
                rows: rows.clone(),
            };
            move |window, cx| TableState::new(delegate, window, cx)
        });
        request.with_window_app(|_, app| {
            entity.update(app, |state, _| {
                let delegate = state.delegate_mut();
                delegate.columns = columns;
                delegate.rows = rows;
            });
        });

        let mut table = DataTable::new(&entity);
        for operation in operations {
            table = match operation {
                DataTableOp::Stripe(value) => table.stripe(value),
                DataTableOp::Bordered(value) => table.bordered(value),
                DataTableOp::Columns(_) => table,
            };
        }
        let mut host = div().size_full().child(table);
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
                            let token = rows.parse::<u64>().map_err(|_| {
                                "DataTable rows token must be a number".to_string()
                            })?;
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
                ])
                .with_documentation(
                    "Retained native DataTable over `id\\t...` rows with declared columns.",
                ),
        )
        .expect("the built-in DataTable descriptor is valid");
}
