//! `List`, adapted from `component-shell`'s `delegate_collections/list.rs`.
//!
//! A retained native list backed by an immutable row snapshot (P4). Rows are
//! `id`, `label`, and an optional `disabled` flag, delivered as tab-separated
//! fields. The custom `render_row` element callback is not exposed: this
//! runtime renders each row from its `label`.

use std::sync::Arc;

use gpui::{
    AnyElement, App, IntoElement as _, ParentElement as _, Refineable as _, SharedString,
    Styled as _,
};
use gpui_component::{
    list::{List, ListDelegate, ListItem, ListState},
    IndexPath,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    ElementCallback, MaterializeRequest, MethodDescriptor, Row,
};

#[derive(Clone)]
struct ListPayload {
    id: String,
    rows: ComponentArgument,
}

#[derive(Clone)]
enum ListOp {
    RenderRow(ComponentArgument),
}

#[derive(Clone)]
struct ListRow {
    id: SharedString,
    label: SharedString,
    disabled: bool,
    fields: Vec<String>,
}

struct ListDelegateImpl {
    rows: Vec<ListRow>,
    render_row: Option<ElementCallback>,
    selected: Option<IndexPath>,
}

fn parse_list_rows(rows: &[Row]) -> Vec<ListRow> {
    rows.iter()
        .enumerate()
        .map(|(index, fields)| {
            let mut parts = fields.iter().cloned();
            let id = parts
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| format!("row-{index}"));
            let label = parts.next().unwrap_or_else(|| id.clone());
            let disabled = matches!(parts.next().as_deref(), Some("true" | "1"));
            ListRow {
                id: id.into(),
                label: label.into(),
                disabled,
                fields: fields.clone(),
            }
        })
        .collect()
}

impl ListDelegate for ListDelegateImpl {
    type Item = ListItem;

    fn items_count(&self, section: usize, _: &App) -> usize {
        usize::from(section == 0) * self.rows.len()
    }

    fn render_item(
        &mut self,
        path: IndexPath,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let row = self.rows.get(path.row)?;
        let content = match &self.render_row {
            Some(callback) => callback
                .build(&row.fields, window, cx)
                .unwrap_or_else(|error| {
                    gpui::div()
                        .child(format!("Failed to render List row: {error}"))
                        .into_any_element()
                }),
            None => gpui::div().child(row.label.clone()).into_any_element(),
        };
        Some(
            ListItem::new(row.id.clone())
                .selected(self.selected == Some(path))
                .disabled(row.disabled)
                .child(content),
        )
    }

    fn set_selected_index(
        &mut self,
        path: Option<IndexPath>,
        _: &mut gpui::Window,
        _: &mut gpui::Context<ListState<Self>>,
    ) {
        self.selected = path;
    }
}

struct ListMaterializer;

impl ComponentMaterializer for ListMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<ListPayload>()
            .ok_or_else(|| "List received an incompatible payload".to_string())?
            .clone();
        if request.children_len() != 0 {
            return Err("List does not accept children".to_string());
        }
        let rows = parse_list_rows(&request.resolve_rows(&payload.rows)?);
        let render_row = request.methods().find_map(|method| {
            method
                .payload()
                .downcast_ref::<ListOp>()
                .map(|ListOp::RenderRow(argument)| argument.clone())
        });
        let render_row = match render_row {
            Some(argument) => Some(request.resolve_element_callback(&argument)?),
            None => None,
        };
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-list:{}", payload.id)));
        let entity = request.use_keyed_state(key, {
            let rows = rows.clone();
            let render_row = render_row.clone();
            move |window, cx| {
                ListState::new(
                    ListDelegateImpl {
                        rows,
                        render_row,
                        selected: None,
                    },
                    window,
                    cx,
                )
            }
        });
        request.with_window_app(|_, app| {
            entity.update(app, |state, _| {
                state.delegate_mut().rows = rows;
                state.delegate_mut().render_row = render_row;
            });
        });

        let mut list = List::new(&entity);
        list.style().refine(&style);
        Ok(list.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("List", Arc::new(ListMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "List",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("rows", ArgumentSchema::Callback),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(rows)]
                            if !id.trim().is_empty() =>
                        {
                            let token = rows.parse::<u64>().map_err(|_| {
                                "List rows token must be a number".to_string()
                            })?;
                            Ok(ComponentPayload::new(ListPayload {
                                id: id.clone(),
                                rows: ComponentArgument::Callback(token),
                            }))
                        }
                        _ => Err("List expects a non-empty id and a rows callback".into()),
                    },
                )]                )
                .with_methods(vec![MethodDescriptor::new(
                    "render_row",
                    vec![ArgumentDescriptor::new("callback", ArgumentSchema::Callback)],
                    |arguments| match arguments {
                        [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                            ListOp::RenderRow(ComponentArgument::Callback(*token)),
                        )),
                        _ => Err("List.render_row(callback) expects a callback".into()),
                    },
                )
                .with_documentation(
                    "Renders each row with managed code, receiving the row's fields.",
                )])
                .with_documentation(
                    "Native retained List backed by an immutable `id\\tlabel[\\tdisabled]` row snapshot.",
                ),
        )
        .expect("the built-in List descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_parse_id_label_and_disabled() {
        let rows = parse_list_rows(&[
            vec!["a".into(), "Alpha".into()],
            vec!["b".into(), "Beta".into(), "true".into()],
        ]);
        assert_eq!(rows[0].id, "a");
        assert_eq!(rows[0].label, "Alpha");
        assert!(!rows[0].disabled);
        assert!(rows[1].disabled);
    }
}
