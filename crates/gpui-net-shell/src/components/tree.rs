//! `Tree` and `TreeItem`, ported from `component-shell`'s `collections/tree.rs`.
//!
//! `Tree` retains a native `TreeState` keyed by its id; `TreeItem` children carry
//! their native value through [`crate::typed_child::Carrier`]. Label/structure
//! syncs by unique item id so native expansion, selection, focus, and scroll
//! persist across renders.

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    px, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _, Refineable as _,
    SharedString, Styled as _,
};
use gpui_component::{h_flex, list::ListItem, tree::Tree, tree::TreeItem, tree::TreeState};
use gpui_component::{Icon, IconName};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone)]
struct ItemPayload {
    id: String,
    label: String,
}

#[derive(Clone, Copy)]
enum ItemOp {
    Expanded(bool),
    Disabled(bool),
}

#[derive(Clone)]
struct TreePayload(String);

struct RetainedTree {
    native: Entity<TreeState>,
    fingerprint: Vec<ItemFingerprint>,
    roots: Vec<TreeItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ItemFingerprint {
    id: String,
    label: String,
    expanded: bool,
    disabled: bool,
    children: Vec<ItemFingerprint>,
}

struct ItemMaterializer;

impl ComponentMaterializer for ItemMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<ItemPayload>()
            .ok_or_else(|| "TreeItem received an incompatible payload".to_string())?;
        let mut item = TreeItem::new(payload.id.clone(), payload.label.clone());
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ItemOp>())
        {
            item = match op {
                ItemOp::Expanded(value) => item.expanded(*value),
                ItemOp::Disabled(value) => item.disabled(*value),
            };
        }
        let _ = request.take_style();
        let children = request.take_typed_children::<TreeItem>(&["TreeItem"])?;
        for child in children {
            item = item.child(child);
        }
        Ok(Carrier::new(item).into_any_element())
    }
}

struct TreeMaterializer;

impl ComponentMaterializer for TreeMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<TreePayload>()
            .ok_or_else(|| "Tree received an incompatible payload".to_string())?
            .0
            .clone();
        let items = request.take_typed_children::<TreeItem>(&["TreeItem"])?;
        validate_unique_ids(&items)?;
        let fingerprint = fingerprint(&items);
        let style = request.take_style();

        let state = request.with_window_app(|window, cx| {
            let retained = window.use_keyed_state(
                SharedString::from(format!("shell-tree:{id}")),
                cx,
                |_, cx| RetainedTree {
                    native: cx.new(|cx| TreeState::new(cx).items(items.clone())),
                    fingerprint: fingerprint.clone(),
                    roots: items.clone(),
                },
            );
            retained.update(cx, |retained, cx| {
                if retained.fingerprint != fingerprint {
                    let mut next = items.clone();
                    preserve_expansion(&mut next, &retained.roots);
                    let selected_id = retained
                        .native
                        .read(cx)
                        .selected_item()
                        .map(|item| item.id.clone());
                    retained.native.update(cx, |native, cx| {
                        native.set_items(next, cx);
                        let selected_ix = selected_id
                            .as_ref()
                            .and_then(|selected| native.index_of(selected));
                        native.set_selected_index(selected_ix, cx);
                    });
                    retained.fingerprint = fingerprint.clone();
                    retained.roots = items;
                }
            });
            retained.read(cx).native.clone()
        });

        let mut tree = Tree::new(&state, move |_ix, entry, selected, _, _| {
            ListItem::new(entry.item().id.clone())
                .selected(selected)
                .w_full()
                .px_3()
                .pl(px(16.) * entry.depth() + px(12.))
                .child(
                    h_flex()
                        .gap_2()
                        .child(Icon::new(if !entry.is_folder() {
                            IconName::File
                        } else if entry.is_expanded() {
                            IconName::FolderOpen
                        } else {
                            IconName::Folder
                        }))
                        .child(entry.item().label.clone()),
                )
        });
        tree.style().refine(&style);
        Ok(tree.into_any_element())
    }
}

fn validate_unique_ids(items: &[TreeItem]) -> Result<(), String> {
    fn walk<'a>(seen: &mut HashSet<&'a str>, item: &'a TreeItem) -> Result<(), String> {
        if !seen.insert(item.id.as_ref()) {
            return Err(format!(
                "TreeItem id `{}` is duplicated; ids must be unique within a Tree",
                item.id
            ));
        }
        for child in &item.children {
            walk(seen, child)?;
        }
        Ok(())
    }
    let mut seen = HashSet::new();
    for item in items {
        walk(&mut seen, item)?;
    }
    Ok(())
}

fn preserve_expansion(incoming: &mut [TreeItem], previous: &[TreeItem]) {
    fn collect(items: &[TreeItem], states: &mut Vec<(SharedString, bool)>) {
        for item in items {
            states.push((item.id.clone(), item.is_expanded()));
            collect(&item.children, states);
        }
    }
    fn apply(items: &mut [TreeItem], states: &[(SharedString, bool)]) {
        for item in items {
            if let Some((_, expanded)) = states.iter().find(|(id, _)| id == &item.id) {
                *item = item.clone().expanded(*expanded);
            }
            apply(&mut item.children, states);
        }
    }
    let mut states = Vec::new();
    collect(previous, &mut states);
    apply(incoming, &states);
}

fn fingerprint(items: &[TreeItem]) -> Vec<ItemFingerprint> {
    items
        .iter()
        .map(|item| ItemFingerprint {
            id: item.id.to_string(),
            label: item.label.to_string(),
            expanded: item.is_expanded(),
            disabled: item.is_disabled(),
            children: fingerprint(&item.children),
        })
        .collect()
}

fn bool_method(name: &'static str, make: fn(bool) -> ItemOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(make(*value))),
            _ => Err(format!("TreeItem.{name}({name}) expects a boolean")),
        },
    )
    .with_documentation("Sets native tree item state.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("TreeItem", Arc::new(ItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "TreeItem",
                    vec![
                        ArgumentDescriptor::new("id", ArgumentSchema::String),
                        ArgumentDescriptor::new("label", ArgumentSchema::String),
                    ],
                    |arguments| match arguments {
                        [ComponentArgument::String(id), ComponentArgument::String(label)]
                            if !id.trim().is_empty() && !label.trim().is_empty() =>
                        {
                            Ok(ComponentPayload::new(ItemPayload {
                                id: id.clone(),
                                label: label.clone(),
                            }))
                        }
                        _ => Err("TreeItem expects non-empty id and label".into()),
                    },
                )])
                .with_methods(vec![
                    bool_method("expanded", ItemOp::Expanded),
                    bool_method("disabled", ItemOp::Disabled),
                ])
                .with_documentation(
                    "Typed native tree data item with a Tree-wide unique id, nested TreeItem \
                     children, and initial expanded/disabled state; style is rejected.",
                ),
        )
        .expect("the built-in TreeItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Tree", Arc::new(TreeMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Tree",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(TreePayload(id.clone())))
                        }
                        _ => Err("Tree expects a non-empty id".into()),
                    },
                )])
                .with_methods(Vec::new())
                .with_documentation(
                    "Native retained tree keyed by a stable id; label/structure data syncs by \
                     unique item id while native expansion, selection, focus, and scroll persist.",
                ),
        )
        .expect("the built-in Tree descriptor is valid");
}
