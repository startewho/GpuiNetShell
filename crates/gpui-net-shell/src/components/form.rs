//! `Form` and `Field`, ported from `component-shell`'s `structured/form.rs`.
//!
//! A vertical or horizontal form accepting `Field` children. `Field` carries
//! its native value to the parent through [`crate::typed_child::Carrier`].

use std::sync::Arc;

use gpui::{AnyElement, IntoElement as _, ParentElement as _, Refineable as _, Styled as _};
use gpui_component::{
    form::{Field, Form},
    Sizable as _, Size,
};

use super::common::{nonnegative_f32, positive_u16};
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone, Copy)]
struct FieldPayload;

#[derive(Clone)]
enum FieldOp {
    Label(String),
    Description(String),
    Required(bool),
    Visible(bool),
    LabelIndent(bool),
    Align(FieldAlign),
    ColSpan(u16),
}

#[derive(Clone, Copy)]
enum FieldAlign {
    Start,
    Center,
    End,
}

struct FieldMaterializer;

impl ComponentMaterializer for FieldMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<FieldPayload>()
            .ok_or_else(|| "Field received an incompatible payload".to_string())?;
        let mut field = Field::new();
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<FieldOp>())
        {
            field = match operation {
                FieldOp::Label(value) => field.label(value.clone()),
                FieldOp::Description(value) => field.description(value.clone()),
                FieldOp::Required(value) => field.required(*value),
                FieldOp::Visible(value) => field.visible(*value),
                FieldOp::LabelIndent(value) => field.label_indent(*value),
                FieldOp::Align(FieldAlign::Start) => field.items_start(),
                FieldOp::Align(FieldAlign::Center) => field.items_center(),
                FieldOp::Align(FieldAlign::End) => field.items_end(),
                FieldOp::ColSpan(value) => field.col_span(*value),
            };
        }
        field.style().refine(&request.take_style());
        field.extend(request.take_children());
        Ok(Carrier::new(field).into_any_element())
    }
}

#[derive(Clone, Copy)]
enum FormPayload {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy)]
enum FormOp {
    Columns(usize),
    LabelWidth(f32),
    Size(Size),
}

struct FormMaterializer;

impl ComponentMaterializer for FormMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<FormPayload>()
            .ok_or_else(|| "Form received an incompatible payload".to_string())?;
        let mut form = match payload {
            FormPayload::Vertical => Form::vertical(),
            FormPayload::Horizontal => Form::horizontal(),
        };
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<FormOp>())
        {
            form = match operation {
                FormOp::Columns(value) => form.columns(*value),
                FormOp::LabelWidth(value) => form.label_width(gpui::px(*value)),
                FormOp::Size(value) => form.with_size(*value),
            };
        }
        let fields = request.take_typed_children::<Field>(&["Field"])?;
        for field in fields {
            form = form.child(field);
        }
        form.style().refine(&request.take_style());
        Ok(form.into_any_element())
    }
}

fn bool_method(
    component: &'static str,
    name: &'static str,
    documentation: &'static str,
    make: fn(bool) -> FieldOp,
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

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Field", Arc::new(FieldMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new("Field", vec![], |_| {
                    Ok(ComponentPayload::new(FieldPayload))
                })])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "label",
                        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                Ok(ComponentPayload::new(FieldOp::Label(value.clone())))
                            }
                            _ => Err("Field.label(label) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the field label."),
                    MethodDescriptor::new(
                        "description",
                        vec![ArgumentDescriptor::new(
                            "description",
                            ArgumentSchema::String,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                Ok(ComponentPayload::new(FieldOp::Description(value.clone())))
                            }
                            _ => Err("Field.description(description) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets supporting text below the control."),
                    bool_method(
                        "Field",
                        "required",
                        "Marks the field as required.",
                        FieldOp::Required,
                    ),
                    bool_method(
                        "Field",
                        "visible",
                        "Controls field visibility.",
                        FieldOp::Visible,
                    ),
                    bool_method(
                        "Field",
                        "label_indent",
                        "Keeps unlabeled horizontal fields aligned with labeled fields.",
                        FieldOp::LabelIndent,
                    ),
                    MethodDescriptor::new(
                        "align",
                        vec![ArgumentDescriptor::new(
                            "align",
                            ArgumentSchema::Enum(&["start", "center", "end"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "start" => {
                                    Ok(ComponentPayload::new(FieldOp::Align(FieldAlign::Start)))
                                }
                                "center" => {
                                    Ok(ComponentPayload::new(FieldOp::Align(FieldAlign::Center)))
                                }
                                "end" => Ok(ComponentPayload::new(FieldOp::Align(FieldAlign::End))),
                                _ => Err(format!("unsupported Field alignment `{value}`")),
                            },
                            _ => Err("Field.align(align) expects an alignment literal".into()),
                        },
                    )
                    .with_documentation("Aligns the label and control within the field."),
                    MethodDescriptor::new(
                        "col_span",
                        vec![ArgumentDescriptor::new("span", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)] => {
                                positive_u16(*value, "Field.col_span")
                                    .map(|value| ComponentPayload::new(FieldOp::ColSpan(value)))
                            }
                            _ => Err(
                                "Field.col_span expects an exactly representable positive integer"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Sets the field's grid-column span."),
                ])
                .with_documentation("A typed form field containing ordinary control children."),
        )
        .expect("the built-in Field descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Form", Arc::new(FormMaterializer))
                .with_constructors(vec![
                    ConstructorDescriptor::new("Form", vec![], |_| {
                        Ok(ComponentPayload::new(FormPayload::Vertical))
                    }),
                    ConstructorDescriptor::new("VForm", vec![], |_| {
                        Ok(ComponentPayload::new(FormPayload::Vertical))
                    }),
                    ConstructorDescriptor::new("HForm", vec![], |_| {
                        Ok(ComponentPayload::new(FormPayload::Horizontal))
                    }),
                ])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "columns",
                        vec![ArgumentDescriptor::new("columns", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)] => {
                                positive_u16(*value, "Form.columns").map(|value| {
                                    ComponentPayload::new(FormOp::Columns(usize::from(value)))
                                })
                            }
                            _ => Err(
                                "Form.columns expects an exactly representable positive integer"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Sets the form grid's column count."),
                    MethodDescriptor::new(
                        "label_width",
                        vec![ArgumentDescriptor::new("width", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)] => {
                                nonnegative_f32(*value, "Form.label_width")
                                    .map(|value| ComponentPayload::new(FormOp::LabelWidth(value)))
                            }
                            _ => Err(
                                "Form.label_width(width) expects a nonnegative finite number"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Sets the horizontal form label width in pixels."),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => Ok(ComponentPayload::new(FormOp::Size(Size::XSmall))),
                                "small" => Ok(ComponentPayload::new(FormOp::Size(Size::Small))),
                                "medium" => Ok(ComponentPayload::new(FormOp::Size(Size::Medium))),
                                "large" => Ok(ComponentPayload::new(FormOp::Size(Size::Large))),
                                _ => Err(format!("unsupported Form size `{value}`")),
                            },
                            _ => Err("Form.size(size) expects a size literal".into()),
                        },
                    )
                    .with_documentation("Sets the form density."),
                ])
                .with_documentation("A vertical or horizontal form accepting Field children."),
        )
        .expect("the built-in Form descriptor is valid");
}
