//! `RadioGroup`, ported from `component-shell`'s `typed_compound/mod.rs`.
//!
//! A controlled radio set that composes [`Radio`] children and reports the
//! selected zero-based index. `Radio` carries its native value through
//! [`crate::typed_child::Part`], so the same child renders on its own and is
//! consumed by the group.

use std::sync::Arc;

use gpui::{AnyElement, Axis, IntoElement as _, Refineable as _, Styled as _};
use gpui_component::radio::{Radio, RadioGroup};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallbackArgument,
    ComponentDescriptor, ComponentMaterializer, ComponentPayload, ComponentRegistry,
    ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::take_part;

#[derive(Clone)]
struct RadioGroupPayload {
    id: String,
    axis: Axis,
}

#[derive(Clone)]
enum RadioGroupOp {
    Selected(usize),
    Disabled(bool),
    OnChange(ComponentArgument),
}

struct RadioGroupMaterializer;

impl ComponentMaterializer for RadioGroupMaterializer {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let payload = request
            .payload()
            .downcast_ref::<RadioGroupPayload>()
            .ok_or_else(|| "RadioGroup received an incompatible payload".to_string())?
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<RadioGroupOp>().cloned())
            .collect::<Vec<_>>();
        let mut group = match payload.axis {
            Axis::Horizontal => RadioGroup::horizontal(payload.id),
            Axis::Vertical => RadioGroup::vertical(payload.id),
        }
        .disabled(request.disabled());
        let mut request = request;
        for operation in operations {
            group = match operation {
                RadioGroupOp::Selected(value) => group.selected_index(Some(value)),
                RadioGroupOp::Disabled(value) => group.disabled(value),
                RadioGroupOp::OnChange(argument) => {
                    let callback = request.resolve_callback(&argument)?;
                    group.on_click(move |index, window, cx| {
                        callback.invoke_with(
                            "RadioGroup.on_change callback failed",
                            &[ComponentCallbackArgument::Number(*index as f64)],
                            window,
                            cx,
                        )
                    })
                }
            };
        }
        for (name, mut element) in request.take_children_named() {
            if name != "Radio" {
                return Err(format!(
                    "RadioGroup accepts only Radio children; received {name}"
                ));
            }
            group = group.child(take_part::<Radio>(&mut element, name)?);
        }
        group.style().refine(&request.take_style());
        Ok(group.into_any_element())
    }
}

fn bool_method(name: &'static str, make: fn(bool) -> RadioGroupOp) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(make(*value))),
            _ => Err(format!("RadioGroup.{name}({name}) expects a boolean")),
        },
    )
    .with_documentation("Sets native RadioGroup behavior.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("RadioGroup", Arc::new(RadioGroupMaterializer))
                .with_constructors(vec![
                    ConstructorDescriptor::new(
                        "RadioGroup",
                        vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                                Ok(ComponentPayload::new(RadioGroupPayload {
                                    id: id.clone(),
                                    axis: Axis::Vertical,
                                }))
                            }
                            _ => Err("RadioGroup(id) expects a non-empty string id".into()),
                        },
                    ),
                    ConstructorDescriptor::new(
                        "HorizontalRadioGroup",
                        vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(id)] if !id.trim().is_empty() => {
                                Ok(ComponentPayload::new(RadioGroupPayload {
                                    id: id.clone(),
                                    axis: Axis::Horizontal,
                                }))
                            }
                            _ => {
                                Err("HorizontalRadioGroup(id) expects a non-empty string id".into())
                            }
                        },
                    ),
                ])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "selected_index",
                        vec![ArgumentDescriptor::new("index", ArgumentSchema::Number)],
                        |arguments| match arguments {
                            [ComponentArgument::Number(value)]
                                if value.is_finite()
                                    && value.fract() == 0.0
                                    && *value >= 0.0
                                    && *value <= usize::MAX as f64 =>
                            {
                                Ok(ComponentPayload::new(RadioGroupOp::Selected(
                                    *value as usize,
                                )))
                            }
                            _ => Err(
                                "RadioGroup.selected_index(index) expects a non-negative integer"
                                    .into(),
                            ),
                        },
                    )
                    .with_documentation("Controls the selected zero-based radio index."),
                    bool_method("disabled", RadioGroupOp::Disabled),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [argument @ ComponentArgument::Callback(_)] => Ok(
                                ComponentPayload::new(RadioGroupOp::OnChange(argument.clone())),
                            ),
                            _ => Err("RadioGroup.on_change expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the selected zero-based index."),
                ])
                .with_documentation(
                    "A controlled radio set accepting only registered Radio children.",
                ),
        )
        .expect("the built-in RadioGroup descriptor is valid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_radio_group_registers_both_orientations() {
        let mut registry = ComponentRegistry::new();
        register(&mut registry);
        let frozen = registry.freeze();
        let descriptor = frozen
            .descriptors()
            .find(|descriptor| descriptor.name() == "RadioGroup")
            .expect("RadioGroup is registered");
        let exports = descriptor
            .constructors()
            .iter()
            .map(|constructor| constructor.export())
            .collect::<Vec<_>>();
        assert_eq!(exports, ["RadioGroup", "HorizontalRadioGroup"]);
    }
}
