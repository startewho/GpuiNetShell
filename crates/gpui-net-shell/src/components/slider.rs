//! `Slider`, adapted from `component-shell`'s `retained_forms/mod.rs`.
//!
//! A retained numeric slider. It keeps a native `SliderState` in keyed state
//! and reports value changes through `on_change(number)`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _, Subscription,
};
use gpui_component::slider::{Slider, SliderEvent, SliderState, SliderValue};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct SliderPayload(String);

#[derive(Clone)]
enum SliderOp {
    Value(f32),
    Min(f32),
    Max(f32),
    Vertical,
    Reverse,
    Disabled(bool),
    OnChange(ComponentArgument),
}

struct Host {
    state: Entity<SliderState>,
    callback: Rc<RefCell<Option<ComponentCallback>>>,
    _selection: Subscription,
}

fn number_op(value: &ComponentArgument) -> Option<f32> {
    match value {
        ComponentArgument::Number(value)
            if value.is_finite() && *value >= f32::MIN as f64 && *value <= f32::MAX as f64 =>
        {
            Some(*value as f32)
        }
        _ => None,
    }
}

struct SliderMaterializer;

impl ComponentMaterializer for SliderMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<SliderPayload>()
            .ok_or_else(|| "Slider received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<SliderOp>().cloned())
            .collect::<Vec<_>>();
        let value = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                SliderOp::Value(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);
        let min = operations.iter().rev().find_map(|op| match op {
            SliderOp::Min(value) => Some(*value),
            _ => None,
        });
        let max = operations.iter().rev().find_map(|op| match op {
            SliderOp::Max(value) => Some(*value),
            _ => None,
        });
        let vertical = operations.iter().any(|op| matches!(op, SliderOp::Vertical));
        let reverse = operations.iter().any(|op| matches!(op, SliderOp::Reverse));
        let disabled = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                SliderOp::Disabled(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_else(|| request.disabled());
        let on_change = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                SliderOp::OnChange(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_callback(&argument))
            .transpose()?;
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-slider:{id}")));
        let on_change_for_init = on_change.clone();
        let host = request.use_keyed_state(key, move |window, cx| {
            let mut builder = SliderState::new();
            if let Some(min) = min {
                builder = builder.min(min);
            }
            if let Some(max) = max {
                builder = builder.max(max);
            }
            builder = builder.default_value(SliderValue::Single(value));
            let state = cx.new(|_| builder);
            let callback = Rc::new(RefCell::new(on_change_for_init));
            let event_callback = callback.clone();
            let selection = window.subscribe(
                &state,
                cx,
                move |_state: Entity<SliderState>, event: &SliderEvent, window, cx| {
                    if let SliderEvent::Change(slider_value) = event {
                        let number = match slider_value {
                            SliderValue::Single(value) => *value as f64,
                            SliderValue::Range(start, _) => *start as f64,
                        };
                        if let Some(callback) = event_callback.borrow().clone() {
                            callback.invoke_with(
                                "Slider.on_change callback failed",
                                &[ComponentCallbackArgument::Number(number)],
                                window,
                                cx,
                            );
                        }
                    }
                },
            );
            Host {
                state,
                callback,
                _selection: selection,
            }
        });
        request.update_entity(&host, |host, _| {
            *host.callback.borrow_mut() = on_change;
        });

        let state = request.with_window_app(|_, app| host.read(app).state.clone());
        let mut slider = Slider::new(&state);
        if vertical {
            slider = slider.vertical();
        }
        if reverse {
            slider = slider.reverse();
        }
        if disabled {
            slider = slider.disabled(true);
        }
        let mut wrapper = div().child(slider);
        wrapper.style().refine(&style);
        Ok(wrapper.into_any_element())
    }
}

fn number_method(
    name: &'static str,
    documentation: &'static str,
    make: fn(f32) -> SliderOp,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Number)],
        move |arguments| match arguments {
            [value] => number_op(value)
                .map(make)
                .map(ComponentPayload::new)
                .ok_or_else(|| format!("Slider.{name} expects a finite number")),
            _ => Err(format!("Slider.{name} expects a finite number")),
        },
    )
    .with_documentation(documentation)
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Slider", Arc::new(SliderMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Slider",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "Slider")
                            .map(SliderPayload)
                            .map(ComponentPayload::new),
                        _ => Err("Slider(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    number_method("value", "Sets the initial value.", SliderOp::Value),
                    number_method("min", "Sets the minimum value.", SliderOp::Min),
                    number_method("max", "Sets the maximum value.", SliderOp::Max),
                    MethodDescriptor::new("vertical", vec![], |_| {
                        Ok(ComponentPayload::new(SliderOp::Vertical))
                    })
                    .with_documentation("Uses a vertical track."),
                    MethodDescriptor::new("reverse", vec![], |_| {
                        Ok(ComponentPayload::new(SliderOp::Reverse))
                    })
                    .with_documentation("Reverses the filled side."),
                    MethodDescriptor::new(
                        "disabled",
                        vec![ArgumentDescriptor::new("disabled", ArgumentSchema::Boolean)],
                        |arguments| match arguments {
                            [ComponentArgument::Boolean(value)] => {
                                Ok(ComponentPayload::new(SliderOp::Disabled(*value)))
                            }
                            _ => Err("Slider.disabled(disabled) expects a boolean".into()),
                        },
                    )
                    .with_documentation("Disables the slider."),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                SliderOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("Slider.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the new value."),
                ])
                .with_documentation("A retained numeric slider."),
        )
        .expect("the built-in Slider descriptor is valid");
}
