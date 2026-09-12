//! `Accordion` and `AccordionItem`, ported from `component-shell`'s
//! `typed_compound/mod.rs`.
//!
//! A typed accordion accepting only `AccordionItem` children. `AccordionItem`
//! carries its native value to the parent through [`crate::typed_child::Carrier`]
//! and renders nothing on its own.
//!
//! `on_toggle` is not exposed: the real component requires a `Send + Sync`
//! handler while shell callbacks are runtime-local, matching `component-shell`.

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, ParentElement as _, Refineable as _, Styled as _};
use gpui_component::{
    accordion::{Accordion, AccordionItem},
    Sizable as _, Size,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone, Copy)]
struct AccordionItemPayload;

#[derive(Clone)]
enum AccordionItemOp {
    Title(ComponentArgument),
    Open(bool),
    Disabled(bool),
}

struct AccordionItemMaterializer;

impl ComponentMaterializer for AccordionItemMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<AccordionItemPayload>()
            .ok_or_else(|| "AccordionItem received an incompatible payload".to_string())?;
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<AccordionItemOp>().cloned())
            .collect::<Vec<_>>();
        let mut item = AccordionItem::new().disabled(request.disabled());
        for operation in operations {
            item = match operation {
                AccordionItemOp::Title(argument) => item.title(request.resolve_element(&argument)?),
                AccordionItemOp::Open(value) => item.open(value),
                AccordionItemOp::Disabled(value) => item.disabled(value),
            };
        }
        item.style().refine(&request.take_style());
        item.extend(request.take_children());
        Ok(Carrier::new(item).into_any_element())
    }
}

#[derive(Clone)]
struct AccordionPayload(String);

#[derive(Clone)]
enum AccordionOp {
    Multiple(bool),
    Bordered(bool),
    Size(Size),
}

struct AccordionMaterializer;

impl ComponentMaterializer for AccordionMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<AccordionPayload>()
            .ok_or_else(|| "Accordion received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<AccordionOp>().cloned())
            .collect::<Vec<_>>();
        let items = request.take_typed_children::<AccordionItem>(&["AccordionItem"])?;
        let style = request.take_style();

        let mut accordion = Accordion::new(id).disabled(request.disabled());
        for operation in operations {
            accordion = match operation {
                AccordionOp::Multiple(value) => accordion.multiple(value),
                AccordionOp::Bordered(value) => accordion.bordered(value),
                AccordionOp::Size(value) => accordion.with_size(value),
            };
        }
        for item in items {
            accordion = accordion.item(move |_| item);
        }
        accordion.style().refine(&style);
        Ok(accordion.into_any_element())
    }
}

fn bool_method<T: Send + Sync + 'static>(
    component: &'static str,
    name: &'static str,
    documentation: &'static str,
    make: impl Fn(bool) -> T + Send + Sync + 'static,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(make(*value))),
            _ => Err(format!("{component}.{name}({name}) expects one boolean")),
        },
    )
    .with_documentation(documentation)
}

fn size_method(
    component: &'static str,
    make: impl Fn(Size) -> AccordionOp + Send + Sync + 'static,
) -> MethodDescriptor {
    MethodDescriptor::new(
        "size",
        vec![ArgumentDescriptor::new(
            "size",
            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
        )],
        move |arguments| match arguments {
            [ComponentArgument::Enum(value)] => match value.as_str() {
                "xsmall" => Ok(ComponentPayload::new(make(Size::XSmall))),
                "small" => Ok(ComponentPayload::new(make(Size::Small))),
                "medium" => Ok(ComponentPayload::new(make(Size::Medium))),
                "large" => Ok(ComponentPayload::new(make(Size::Large))),
                _ => Err(format!("{component}.size(size) expects a semantic size")),
            },
            _ => Err(format!("{component}.size(size) expects a semantic size")),
        },
    )
    .with_documentation("Sets the semantic component size.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("AccordionItem", Arc::new(AccordionItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "AccordionItem",
                    vec![],
                    |_| Ok(ComponentPayload::new(AccordionItemPayload)),
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "title",
                        vec![ArgumentDescriptor::new("title", ArgumentSchema::Element)],
                        |arguments| match arguments {
                            [value @ ComponentArgument::Element(_)] => {
                                Ok(ComponentPayload::new(AccordionItemOp::Title(value.clone())))
                            }
                            _ => Err("AccordionItem.title(title) expects an element".into()),
                        },
                    )
                    .with_documentation("Sets the interactive title row element."),
                    bool_method(
                        "AccordionItem",
                        "open",
                        "Controls expanded state.",
                        AccordionItemOp::Open,
                    ),
                    bool_method(
                        "AccordionItem",
                        "disabled",
                        "Disables this item.",
                        AccordionItemOp::Disabled,
                    ),
                ])
                .with_documentation("An accordion part accepted only as a direct Accordion child."),
        )
        .expect("the built-in AccordionItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Accordion", Arc::new(AccordionMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Accordion",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(AccordionPayload(id.clone())))
                        }
                        _ => Err("Accordion(id) expects a nonempty string id".into()),
                    },
                )])
                .with_methods(vec![
                    bool_method(
                        "Accordion",
                        "multiple",
                        "Allows multiple items to remain open.",
                        AccordionOp::Multiple,
                    ),
                    bool_method(
                        "Accordion",
                        "bordered",
                        "Controls the joined outer border.",
                        AccordionOp::Bordered,
                    ),
                    size_method("Accordion", AccordionOp::Size),
                ])
                .with_documentation(
                    "A typed accordion container accepting only AccordionItem children.",
                ),
        )
        .expect("the built-in Accordion descriptor is valid");
}
