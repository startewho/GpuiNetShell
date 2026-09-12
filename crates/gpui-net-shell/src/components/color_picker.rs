//! `ColorPicker`, adapted from `component-shell`'s `retained_forms/mod.rs`.
//!
//! A retained color picker. It keeps a native `ColorPickerState` in keyed state
//! and reports the committed color as a hex string through `on_change(string)`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, AppContext as _, Entity, IntoElement as _, ParentElement as _,
    Refineable as _, SharedString, Styled as _, Subscription,
};
use gpui_component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_component::Colorize as _;

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentCallback,
    ComponentCallbackArgument, ComponentDescriptor, ComponentMaterializer, ComponentPayload,
    ComponentRegistry, ConstructorDescriptor, MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct ColorPickerPayload(String);

#[derive(Clone)]
enum ColorPickerOp {
    Label(String),
    AccessibilityLabel(String),
    OnChange(ComponentArgument),
}

struct Host {
    state: Entity<ColorPickerState>,
    callback: Rc<RefCell<Option<ComponentCallback>>>,
    _selection: Subscription,
}

struct ColorPickerMaterializer;

impl ComponentMaterializer for ColorPickerMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<ColorPickerPayload>()
            .ok_or_else(|| "ColorPicker received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ColorPickerOp>().cloned())
            .collect::<Vec<_>>();
        let label = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                ColorPickerOp::Label(value) => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let accessibility_label = operations.iter().rev().find_map(|op| match op {
            ColorPickerOp::AccessibilityLabel(value) => Some(value.clone()),
            _ => None,
        });
        let on_change = operations
            .iter()
            .rev()
            .find_map(|op| match op {
                ColorPickerOp::OnChange(argument) => Some(argument.clone()),
                _ => None,
            })
            .map(|argument| request.resolve_callback(&argument))
            .transpose()?;
        let style = request.take_style();

        let key = gpui::ElementId::Name(SharedString::from(format!("shell-color-picker:{id}")));
        let on_change_for_init = on_change.clone();
        let host = request.use_keyed_state(key, move |window, cx| {
            let state = cx.new(|cx| ColorPickerState::new(window, cx));
            let callback = Rc::new(RefCell::new(on_change_for_init));
            let event_callback = callback.clone();
            let selection = window.subscribe(
                &state,
                cx,
                move |_state: Entity<ColorPickerState>, event: &ColorPickerEvent, window, cx| {
                    let ColorPickerEvent::Change(color) = event;
                    let text = color.map(|color| color.to_hex()).unwrap_or_default();
                    if let Some(callback) = event_callback.borrow().clone() {
                        callback.invoke_with(
                            "ColorPicker.on_change callback failed",
                            &[ComponentCallbackArgument::String(text)],
                            window,
                            cx,
                        );
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
        let mut picker = ColorPicker::new(&state);
        if !label.is_empty() {
            picker = picker.label(label);
        }
        if let Some(accessibility_label) = accessibility_label {
            picker = picker.accessibility_label(accessibility_label);
        }
        let mut wrapper = div().child(picker);
        wrapper.style().refine(&style);
        Ok(wrapper.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("ColorPicker", Arc::new(ColorPickerMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "ColorPicker",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "ColorPicker")
                            .map(ColorPickerPayload)
                            .map(ComponentPayload::new),
                        _ => Err("ColorPicker(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    MethodDescriptor::new(
                        "label",
                        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => {
                                Ok(ComponentPayload::new(ColorPickerOp::Label(value.clone())))
                            }
                            _ => Err("ColorPicker.label(label) expects a string".into()),
                        },
                    )
                    .with_documentation("Sets the visible label above the picker."),
                    MethodDescriptor::new(
                        "accessibility_label",
                        vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                        |arguments| match arguments {
                            [ComponentArgument::String(value)] => Ok(ComponentPayload::new(
                                ColorPickerOp::AccessibilityLabel(value.clone()),
                            )),
                            _ => {
                                Err("ColorPicker.accessibility_label(label) expects a string"
                                    .into())
                            }
                        },
                    )
                    .with_documentation(
                        "Sets the announced name independently of the visible label.",
                    ),
                    MethodDescriptor::new(
                        "on_change",
                        vec![ArgumentDescriptor::new(
                            "callback",
                            ArgumentSchema::Callback,
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Callback(token)] => Ok(ComponentPayload::new(
                                ColorPickerOp::OnChange(ComponentArgument::Callback(*token)),
                            )),
                            _ => Err("ColorPicker.on_change(callback) expects a callback".into()),
                        },
                    )
                    .with_documentation("Reports the committed color as a hex string."),
                ])
                .with_documentation("A retained color picker with preview and commit behavior."),
        )
        .expect("the built-in ColorPicker descriptor is valid");
}
