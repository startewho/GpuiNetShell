//! The simple structural `Table` family, ported from `component-shell`'s
//! `structured/table.rs`.
//!
//! `Table` accepts `TableHeader`/`TableBody`/`TableFooter`/`TableCaption`
//! children; the header/body/footer accept `TableRow`; a row accepts `TableHead`
//! and `TableCell`; the head/cell/caption accept ordinary children. Every part
//! is a typed child carried to its parent.

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, ParentElement as _, Refineable as _, Styled as _};
use gpui_component::table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
use gpui_component::{Sizable as _, Size};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::{take_typed, Carrier};

#[derive(Clone, Copy)]
struct Empty;

#[derive(Clone)]
enum TableOp {
    AccessibilityLabel(String),
    Size(Size),
}

#[derive(Clone, Copy)]
enum CellOp {
    ColSpan(usize),
    Center,
    Right,
}

fn nullary(export: &'static str) -> ConstructorDescriptor {
    ConstructorDescriptor::new(export, vec![], |_| Ok(ComponentPayload::new(Empty)))
}

macro_rules! container {
    ($ident:ident, $ty:ty, $label:literal) => {
        struct $ident;
        impl ComponentMaterializer for $ident {
            fn materialize(
                &self,
                mut request: MaterializeRequest<'_>,
            ) -> Result<AnyElement, String> {
                let mut component = <$ty>::new();
                component.style().refine(&request.take_style());
                let rows = request.take_typed_children::<TableRow>(&["TableRow"])?;
                for row in rows {
                    component = component.child(row);
                }
                Ok(Carrier::new(component).into_any_element())
            }
        }
    };
}

container!(HeaderMaterializer, TableHeader, "TableHeader");
container!(BodyMaterializer, TableBody, "TableBody");
container!(FooterMaterializer, TableFooter, "TableFooter");

struct RowMaterializer;

impl ComponentMaterializer for RowMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut row = TableRow::new();
        row.style().refine(&request.take_style());
        for (name, mut element) in request.take_children_named() {
            row = match name {
                "TableHead" => row.child(take_typed::<TableHead>(&mut element, name)?),
                "TableCell" => row.child(take_typed::<TableCell>(&mut element, name)?),
                other => {
                    return Err(format!(
                        "TableRow accepts only TableHead or TableCell children; received {other}"
                    ))
                }
            };
        }
        Ok(Carrier::new(row).into_any_element())
    }
}

macro_rules! leaf {
    ($ident:ident, $ty:ty) => {
        struct $ident;
        impl ComponentMaterializer for $ident {
            fn materialize(
                &self,
                mut request: MaterializeRequest<'_>,
            ) -> Result<AnyElement, String> {
                let mut component = <$ty>::new();
                for op in request
                    .methods()
                    .filter_map(|method| method.payload().downcast_ref::<CellOp>())
                {
                    component = match op {
                        CellOp::ColSpan(value) => component.col_span(*value),
                        CellOp::Center => component.text_center(),
                        CellOp::Right => component.text_right(),
                    };
                }
                component.style().refine(&request.take_style());
                component.extend(request.take_children());
                Ok(Carrier::new(component).into_any_element())
            }
        }
    };
}

leaf!(HeadMaterializer, TableHead);
leaf!(CellMaterializer, TableCell);

struct CaptionMaterializer;

impl ComponentMaterializer for CaptionMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut caption = TableCaption::new();
        caption.style().refine(&request.take_style());
        caption.extend(request.take_children());
        Ok(Carrier::new(caption).into_any_element())
    }
}

struct TableMaterializer;

impl ComponentMaterializer for TableMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut table = Table::new();
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<TableOp>())
        {
            table = match op {
                TableOp::AccessibilityLabel(value) => table.accessibility_label(value.clone()),
                TableOp::Size(value) => table.with_size(*value),
            };
        }
        table.style().refine(&request.take_style());
        for (name, mut element) in request.take_children_named() {
            table = match name {
                "TableHeader" => table.child(take_typed::<TableHeader>(&mut element, name)?),
                "TableBody" => table.child(take_typed::<TableBody>(&mut element, name)?),
                "TableFooter" => table.child(take_typed::<TableFooter>(&mut element, name)?),
                "TableCaption" => table.child(take_typed::<TableCaption>(&mut element, name)?),
                other => {
                    return Err(format!(
                        "Table accepts only TableHeader, TableBody, TableFooter, or TableCaption \
                         children; received {other}"
                    ))
                }
            };
        }
        Ok(table.into_any_element())
    }
}

fn cell_methods() -> Vec<MethodDescriptor> {
    vec![
        MethodDescriptor::new(
            "col_span",
            vec![ArgumentDescriptor::new("span", ArgumentSchema::Number)],
            |arguments| match arguments {
                [ComponentArgument::Number(value)]
                    if value.is_finite() && *value >= 1.0 && value.fract() == 0.0 =>
                {
                    Ok(ComponentPayload::new(CellOp::ColSpan(*value as usize)))
                }
                _ => Err("col_span(span) expects a positive integer".into()),
            },
        )
        .with_documentation("Sets the number of columns occupied by the cell."),
        MethodDescriptor::new("text_center", vec![], |_| {
            Ok(ComponentPayload::new(CellOp::Center))
        })
        .with_documentation("Centers the cell content."),
        MethodDescriptor::new("text_right", vec![], |_| {
            Ok(ComponentPayload::new(CellOp::Right))
        })
        .with_documentation("Right-aligns the cell content."),
    ]
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    for (name, materializer) in [
        (
            "TableHeader",
            Arc::new(HeaderMaterializer) as Arc<dyn ComponentMaterializer>,
        ),
        ("TableBody", Arc::new(BodyMaterializer)),
        ("TableFooter", Arc::new(FooterMaterializer)),
        ("TableRow", Arc::new(RowMaterializer)),
        ("TableHead", Arc::new(HeadMaterializer)),
        ("TableCell", Arc::new(CellMaterializer)),
        ("TableCaption", Arc::new(CaptionMaterializer)),
    ] {
        let mut descriptor = ComponentDescriptor::new(name, materializer)
            .with_constructors(vec![nullary(name)])
            .with_documentation("A typed structural child in a simple Table.");
        if name == "TableHead" || name == "TableCell" {
            descriptor = descriptor.with_methods(cell_methods());
        }
        registry
            .register(descriptor)
            .expect("the built-in table part descriptor is valid");
    }

    registry
        .register(
            ComponentDescriptor::new("Table", Arc::new(TableMaterializer))
                .with_constructors(vec![nullary("Table")])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "accessibility_label",
                        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(
                                TableOp::AccessibilityLabel(value.clone()),
                            )),
                            _ => Err("Table.accessibility_label(label) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the table's screen-reader accessible name."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => Ok(ComponentPayload::new(TableOp::Size(Size::XSmall))),
                                "small" => Ok(ComponentPayload::new(TableOp::Size(Size::Small))),
                                "medium" => Ok(ComponentPayload::new(TableOp::Size(Size::Medium))),
                                "large" => Ok(ComponentPayload::new(TableOp::Size(Size::Large))),
                                _ => Err(format!("unsupported Table size `{value}`")),
                            },
                            _ => Err("Table.size(size) expects a size literal".into()),
                        },
                    )
                    .with_documentation("Sets the table density."),
                ])
                .with_documentation(
                    "A simple stateless table composed from typed table-part children.",
                ),
        )
        .expect("the built-in Table descriptor is valid");
}
