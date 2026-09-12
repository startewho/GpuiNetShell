//! `Stepper` and `StepperItem`, ported from `component-shell`'s
//! `typed_compound/mod.rs`.
//!
//! A typed progress stepper accepting only `StepperItem` children.
//! `StepperItem` carries its native value to the parent through
//! [`crate::typed_child::Carrier`] and renders nothing on its own.

use std::sync::Arc;

use gpui::{AnyElement, Axis, IntoElement as _, ParentElement as _, Refineable as _, Styled as _};
use gpui_component::{
    stepper::{Stepper, StepperItem},
    Sizable as _, Size,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallbackArgument,
    ComponentDescriptor, ComponentMaterializer, ComponentPayload, ComponentRegistry,
    ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone, Copy)]
struct StepperItemPayload;

#[derive(Clone, Copy)]
enum StepperItemOp {
    Disabled(bool),
}

struct StepperItemMaterializer;

impl ComponentMaterializer for StepperItemMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        request
            .payload()
            .downcast_ref::<StepperItemPayload>()
            .ok_or_else(|| "StepperItem received an incompatible payload".to_string())?;
        let mut item = StepperItem::new().disabled(request.disabled());
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<StepperItemOp>())
        {
            match operation {
                StepperItemOp::Disabled(value) => item = item.disabled(*value),
            }
        }
        item.style().refine(&request.take_style());
        item.extend(request.take_children());
        Ok(Carrier::new(item).into_any_element())
    }
}

#[derive(Clone)]
struct StepperPayload(String);

#[derive(Clone)]
enum StepperOp {
    Selected(usize),
    Vertical(bool),
    TextCenter(bool),
    Disabled(bool),
    Size(Size),
    OnChange(ComponentArgument),
}

struct StepperMaterializer;

impl ComponentMaterializer for StepperMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<StepperPayload>()
            .ok_or_else(|| "Stepper received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<StepperOp>().cloned())
            .collect::<Vec<_>>();
        let items = request.take_typed_children::<StepperItem>(&["StepperItem"])?;
        let style = request.take_style();

        let mut stepper = Stepper::new(id).disabled(request.disabled());
        for operation in operations {
            stepper = match operation {
                StepperOp::Selected(value) => stepper.selected_index(value),
                StepperOp::Vertical(value) => stepper.layout(if value {
                    Axis::Vertical
                } else {
                    Axis::Horizontal
                }),
                StepperOp::TextCenter(value) => stepper.text_center(value),
                StepperOp::Disabled(value) => stepper.disabled(value),
                StepperOp::Size(value) => stepper.with_size(value),
                StepperOp::OnChange(argument) => {
                    let callback = request.resolve_callback(&argument)?;
                    stepper.on_click(move |index, window, cx| {
                        callback.invoke_with(
                            "Stepper.on_change callback failed",
                            &[ComponentCallbackArgument::Number(*index as f64)],
                            window,
                            cx,
                        );
                    })
                }
            };
        }
        for item in items {
            stepper = stepper.item(item);
        }
        stepper.style().refine(&style);
        Ok(stepper.into_any_element())
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

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("StepperItem", Arc::new(StepperItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "StepperItem",
                    vec![],
                    |_| Ok(ComponentPayload::new(StepperItemPayload)),
                )])
                .with_methods(vec![bool_method(
                    "StepperItem",
                    "disabled",
                    "Disables this step independently of its parent.",
                    StepperItemOp::Disabled,
                )])
                .with_documentation("A step part accepted only as a direct Stepper child."),
        )
        .expect("the built-in StepperItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Stepper", Arc::new(StepperMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Stepper",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                            Ok(ComponentPayload::new(StepperPayload(id.clone())))
                        }
                        _ => Err("Stepper(id) expects a nonempty string id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "selected_index",
                        vec![ArgumentDescriptor::new("index", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 =>
                            {
                                Ok(ComponentPayload::new(StepperOp::Selected(*value as usize)))
                            }
                            _ => Err(
                                "Stepper.selected_index(index) expects a nonnegative integer"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Controls the current zero-based step."),
                    bool_method(
                        "Stepper",
                        "vertical",
                        "Switches between vertical and horizontal layout.",
                        StepperOp::Vertical,
                    ),
                    bool_method(
                        "Stepper",
                        "text_center",
                        "Centers each step's text in horizontal layouts.",
                        StepperOp::TextCenter,
                    ),
                    bool_method(
                        "Stepper",
                        "disabled",
                        "Disables every step.",
                        StepperOp::Disabled,
                    ),
                    MethodDescriptor::new(
                        "size",
                        vec![ArgumentDescriptor::new(
                            "size",
                            ArgumentSchema::Enum(&["xsmall", "small", "medium", "large"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "xsmall" => {
                                    Ok(ComponentPayload::new(StepperOp::Size(Size::XSmall)))
                                }
                                "small" => Ok(ComponentPayload::new(StepperOp::Size(Size::Small))),
                                "medium" => {
                                    Ok(ComponentPayload::new(StepperOp::Size(Size::Medium)))
                                }
                                "large" => Ok(ComponentPayload::new(StepperOp::Size(Size::Large))),
                                _ => Err(format!("unsupported Stepper size `{value}`")),
                            },
                            _ => Err("Stepper.size(size) expects a semantic size".into()),
                        },
                    )
                    .with_documentation("Sets the semantic component size."),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                StepperOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("Stepper.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the newly selected zero-based index."),
                ])
                .with_documentation(
                    "A typed progress stepper accepting only StepperItem children.",
                ),
        )
        .expect("the built-in Stepper descriptor is valid");
}
