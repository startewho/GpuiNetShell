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
    MaterializeRequest, Row,
};

#[derive(Clone)]
struct ListPayload {
    id: String,
    rows: ComponentArgument,
}

#[derive(Clone)]
struct ListRow {
    id: SharedString,
    label: SharedString,
    disabled: bool,
}

struct ListDelegateImpl {
    rows: Vec<ListRow>,
    selected: Option<IndexPath>,
}

fn parse_list_rows(rows: Vec<Row>) -> Vec<ListRow> {
    rows.into_iter()
        .enumerate()
        .map(|(index, fields)| {
            let mut fields = fields.into_iter();
            let id = fields
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| format!("row-{index}"));
            let label = fields.next().unwrap_or_else(|| id.clone());
            let disabled = matches!(fields.next().as_deref(), Some("true" | "1"));
            ListRow {
                id: id.into(),
                label: label.into(),
                disabled,
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
        _: &mut gpui::Window,
        _: &mut gpui::Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let row = self.rows.get(path.row)?;
        Some(
            ListItem::new(row.id.clone())
                .selected(self.selected == Some(path))
                .disabled(row.disabled)
                .child(row.label.clone()),
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
        let rows = parse_list_rows(request.resolve_rows(&payload.rows)?);
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-list:{}", payload.id)));
        let entity = request.use_keyed_state(key, {
            let rows = rows.clone();
            move |window, cx| {
                ListState::new(
                    ListDelegateImpl {
                        rows,
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
                )])
                .with_methods(Vec::new())
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
        let rows = parse_list_rows(vec![
            vec!["a".into(), "Alpha".into()],
            vec!["b".into(), "Beta".into(), "true".into()],
        ]);
        assert_eq!(rows[0].id, "a");
        assert_eq!(rows[0].label, "Alpha");
        assert!(!rows[0].disabled);
        assert!(rows[1].disabled);
    }
}
