//! The component registry, adapted from `gpui-shell`'s `component_registry.rs`.
//!
//! A registered component owns three things: the constructors and methods a
//! description can name, and the [`ComponentMaterializer`] that turns a
//! [`MaterializeRequest`] into an element. `materialize.rs` resolves a node's
//! ops into a request and hands it to the descriptor; it never names a concrete
//! component.
//!
//! This is the seam that keeps component knowledge out of the runtime: the
//! decoder knows only an id, the dispatcher knows only a descriptor, and a
//! component such as `Button` is one file with its payloads, methods, and
//! materializer.

use std::any::Any;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use gpui::{
    AnyElement, App, IntoElement, ParentElement, Refineable as _, StyleRefinement, Styled, Window,
};

use crate::context::HostContext;

/// A registered component's position in the registry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ComponentId(u32);

impl ComponentId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

/// An owned value created by a constructor or method recorder.
#[derive(Clone)]
pub struct ComponentPayload(Arc<dyn Any + Send + Sync>);

impl ComponentPayload {
    pub fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

impl fmt::Debug for ComponentPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("ComponentPayload").finish()
    }
}

/// One argument passed to a constructor or method recorder.
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentArgument {
    String(String),
    Number(f64),
    Boolean(bool),
    Enum(String),
    Callback(u64),
}

impl ComponentArgument {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) | Self::Enum(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Boolean(value) => Some(if *value { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Boolean(value) => *value,
            Self::Number(value) => *value != 0.0 && !value.is_nan(),
            Self::String(value) | Self::Enum(value) => !value.is_empty(),
            Self::Callback(_) => true,
        }
    }
}

/// The kind an argument is declared with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgumentSchema {
    String,
    Number,
    Boolean,
    /// A closed set of string literals a method accepts.
    Enum(&'static [&'static str]),
    Callback,
}

/// One declared argument of a constructor or method.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArgumentDescriptor {
    name: &'static str,
    schema: ArgumentSchema,
}

impl ArgumentDescriptor {
    pub const fn new(name: &'static str, schema: ArgumentSchema) -> Self {
        Self { name, schema }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn schema(&self) -> ArgumentSchema {
        self.schema
    }
}

type PayloadFactory =
    dyn Fn(&[ComponentArgument]) -> Result<ComponentPayload, String> + Send + Sync + 'static;

/// A constructor export: how a description records the component's payload.
#[derive(Clone)]
pub struct ConstructorDescriptor {
    export: &'static str,
    arguments: Vec<ArgumentDescriptor>,
    factory: Arc<PayloadFactory>,
}

impl ConstructorDescriptor {
    pub fn new(
        export: &'static str,
        arguments: Vec<ArgumentDescriptor>,
        factory: impl Fn(&[ComponentArgument]) -> Result<ComponentPayload, String>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            export,
            arguments,
            factory: Arc::new(factory),
        }
    }

    pub fn export(&self) -> &'static str {
        self.export
    }

    pub fn arguments(&self) -> &[ArgumentDescriptor] {
        &self.arguments
    }

    pub fn payload(&self, arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
        (self.factory)(arguments)
    }
}

impl fmt::Debug for ConstructorDescriptor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConstructorDescriptor")
            .field("export", &self.export)
            .field("arguments", &self.arguments)
            .finish_non_exhaustive()
    }
}

/// A method export: how a description records one call's payload.
#[derive(Clone)]
pub struct MethodDescriptor {
    name: &'static str,
    arguments: Vec<ArgumentDescriptor>,
    documentation: Option<&'static str>,
    recorder: Arc<PayloadFactory>,
}

impl MethodDescriptor {
    pub fn new(
        name: &'static str,
        arguments: Vec<ArgumentDescriptor>,
        recorder: impl Fn(&[ComponentArgument]) -> Result<ComponentPayload, String>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            name,
            arguments,
            documentation: None,
            recorder: Arc::new(recorder),
        }
    }

    pub fn with_documentation(mut self, documentation: &'static str) -> Self {
        self.documentation = Some(documentation);
        self
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn arguments(&self) -> &[ArgumentDescriptor] {
        &self.arguments
    }

    pub fn documentation(&self) -> Option<&'static str> {
        self.documentation
    }

    pub fn record(&self, arguments: &[ComponentArgument]) -> Result<ComponentPayload, String> {
        (self.recorder)(arguments)
    }
}

impl fmt::Debug for MethodDescriptor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MethodDescriptor")
            .field("name", &self.name)
            .field("arguments", &self.arguments)
            .field("documentation", &self.documentation)
            .finish_non_exhaustive()
    }
}

/// One recorded component method, in declaration order.
#[derive(Clone, Debug)]
pub struct RecordedComponentMethod {
    name: &'static str,
    payload: ComponentPayload,
}

impl RecordedComponentMethod {
    pub(crate) fn new(name: &'static str, payload: ComponentPayload) -> Self {
        Self { name, payload }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn payload(&self) -> &ComponentPayload {
        &self.payload
    }
}

/// One value carried back to a managed callback.
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentCallbackArgument {
    Boolean(bool),
    Number(f64),
    String(String),
}

/// A callback resolved from a component's recorded method arguments.
///
/// The token identifies a managed handler registered for the current rendering
/// generation; the [`HostContext`] carries the native callback table used to
/// invoke it. This is what lets a control whose value changes — a radio, a
/// popover's open state — report that value, rather than only that it was
/// interacted with.
#[derive(Clone)]
pub struct ComponentCallback {
    host: HostContext,
    token: u64,
}

impl ComponentCallback {
    pub fn token(&self) -> u64 {
        self.token
    }

    /// Invokes the managed handler with no value, then requests a repaint.
    pub fn invoke(&self, _window: &mut Window, cx: &mut App) {
        self.dispatch(&[], cx);
    }

    /// Invokes the managed handler with one value, then requests a repaint.
    pub fn invoke_with(
        &self,
        _context: &str,
        arguments: &[ComponentCallbackArgument],
        _window: &mut Window,
        cx: &mut App,
    ) {
        self.dispatch(arguments, cx);
    }

    fn dispatch(&self, arguments: &[ComponentCallbackArgument], cx: &mut App) {
        if let Some(invoke) = self.host.callbacks.invoke {
            for argument in arguments {
                let (kind, number, data, len) = match argument {
                    ComponentCallbackArgument::Boolean(value) => (
                        crate::schema::CALLBACK_VALUE_BOOLEAN,
                        if *value { 1.0 } else { 0.0 },
                        std::ptr::null(),
                        0,
                    ),
                    ComponentCallbackArgument::Number(value) => (
                        crate::schema::CALLBACK_VALUE_NUMBER,
                        *value,
                        std::ptr::null(),
                        0,
                    ),
                    ComponentCallbackArgument::String(value) => (
                        crate::schema::CALLBACK_VALUE_STRING,
                        0.0,
                        value.as_ptr(),
                        value.len() as u32,
                    ),
                };
                // SAFETY: the managed callback copies anything it retains before
                // returning; `data` points at a live String for the call.
                unsafe {
                    let _ = invoke(self.host.session_id, self.token, kind, number, data, len);
                }
            }
        }
        (self.host.invalidate)(cx);
    }
}

/// Turns a recorded component into an element.
pub trait ComponentMaterializer: Send + Sync + 'static {
    fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String>;
}

/// Inputs supplied while one registered component is materialized.
pub struct MaterializeRequest<'a> {
    component_name: &'static str,
    payload: &'a ComponentPayload,
    methods: &'a [RecordedComponentMethod],
    host: &'a HostContext,
    style: Option<StyleRefinement>,
    children: Vec<AnyElement>,
    /// Named children the component reads by name instead of as ordinary
    /// children (a popover's `trigger` and `content`).
    slots: Vec<(String, AnyElement)>,
    disabled: bool,
    selected: bool,
    on_click: Option<u64>,
    window: &'a mut Window,
    cx: &'a mut App,
}

impl<'a> MaterializeRequest<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        component_name: &'static str,
        payload: &'a ComponentPayload,
        methods: &'a [RecordedComponentMethod],
        host: &'a HostContext,
        style: StyleRefinement,
        children: Vec<AnyElement>,
        slots: Vec<(String, AnyElement)>,
        disabled: bool,
        selected: bool,
        on_click: Option<u64>,
        window: &'a mut Window,
        cx: &'a mut App,
    ) -> Self {
        Self {
            component_name,
            payload,
            methods,
            host,
            style: Some(style),
            children,
            slots,
            disabled,
            selected,
            on_click,
            window,
            cx,
        }
    }

    pub fn component_name(&self) -> &'static str {
        self.component_name
    }

    pub fn payload(&self) -> &ComponentPayload {
        self.payload
    }

    pub fn methods(&self) -> impl Iterator<Item = &RecordedComponentMethod> {
        self.methods.iter()
    }

    pub fn host(&self) -> &HostContext {
        self.host
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn on_click(&self) -> Option<u64> {
        self.on_click
    }

    /// Resolves a callback argument recorded by a component method into an
    /// invokable handle. Fails when the argument is not a callback token.
    pub fn resolve_callback(
        &self,
        argument: &ComponentArgument,
    ) -> Result<ComponentCallback, String> {
        match argument {
            ComponentArgument::Callback(token) => Ok(ComponentCallback {
                host: self.host.clone(),
                token: *token,
            }),
            other => Err(format!("expected a callback argument, got {other:?}")),
        }
    }

    pub fn children_len(&self) -> usize {
        self.children.len()
    }

    pub fn take_children(&mut self) -> Vec<AnyElement> {
        std::mem::take(&mut self.children)
    }

    /// Takes a named slot, if the description supplied one.
    pub fn take_slot(&mut self, name: &str) -> Option<AnyElement> {
        self.slots
            .iter()
            .position(|(held, _)| held == name)
            .map(|index| self.slots.remove(index).1)
    }

    /// Runs `body` with the current window and app, for components that need
    /// window-scoped state (scroll handles, keyed element state).
    pub fn with_window_app<R>(&mut self, body: impl FnOnce(&mut Window, &mut App) -> R) -> R {
        body(self.window, self.cx)
    }

    pub fn take_style(&mut self) -> StyleRefinement {
        self.style.take().unwrap_or_default()
    }

    /// Applies this node's style and ordinary children exactly once.
    pub fn finish<E>(mut self, mut element: E) -> Result<AnyElement, String>
    where
        E: Styled + ParentElement + IntoElement + 'static,
    {
        element.style().refine(&self.take_style());
        element.extend(self.take_children());
        Ok(element.into_any_element())
    }
}

/// One registered component: what a description can construct, what it can call
/// on the result, and who turns the recording into an element.
pub struct ComponentDescriptor {
    name: &'static str,
    constructors: Vec<ConstructorDescriptor>,
    methods: Vec<MethodDescriptor>,
    documentation: Option<&'static str>,
    materializer: Arc<dyn ComponentMaterializer>,
}

impl ComponentDescriptor {
    pub fn new(name: &'static str, materializer: Arc<dyn ComponentMaterializer>) -> Self {
        Self {
            name,
            constructors: Vec::new(),
            methods: Vec::new(),
            documentation: None,
            materializer,
        }
    }

    pub fn with_constructors(mut self, constructors: Vec<ConstructorDescriptor>) -> Self {
        self.constructors = constructors;
        self
    }

    pub fn with_methods(mut self, methods: Vec<MethodDescriptor>) -> Self {
        self.methods = methods;
        self
    }

    pub fn with_documentation(mut self, documentation: &'static str) -> Self {
        self.documentation = Some(documentation);
        self
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn constructors(&self) -> &[ConstructorDescriptor] {
        &self.constructors
    }

    pub fn methods(&self) -> &[MethodDescriptor] {
        &self.methods
    }

    pub fn method(&self, name: &str) -> Option<&MethodDescriptor> {
        self.methods.iter().find(|method| method.name == name)
    }

    pub fn documentation(&self) -> Option<&'static str> {
        self.documentation
    }

    pub(crate) fn materializer(&self) -> &Arc<dyn ComponentMaterializer> {
        &self.materializer
    }
}

impl fmt::Debug for ComponentDescriptor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ComponentDescriptor")
            .field("name", &self.name)
            .field("constructors", &self.constructors)
            .field("methods", &self.methods)
            .finish_non_exhaustive()
    }
}

/// A failure while building a registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryError {
    DuplicateComponent(&'static str),
    InvalidComponent(&'static str),
    EmptyConstructorList(&'static str),
    DuplicateMethod {
        component: &'static str,
        method: &'static str,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(name) => {
                write!(formatter, "component `{name}` is already registered")
            }
            Self::InvalidComponent(name) => {
                write!(
                    formatter,
                    "component name `{name}` is not a valid identifier"
                )
            }
            Self::EmptyConstructorList(name) => {
                write!(formatter, "component `{name}` has no constructor")
            }
            Self::DuplicateMethod { component, method } => {
                write!(
                    formatter,
                    "component `{component}` registers method `{method}` twice"
                )
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// A builder that validates and freezes a component catalog.
pub struct ComponentRegistry {
    descriptors: Vec<Arc<ComponentDescriptor>>,
    names: HashSet<&'static str>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
            names: HashSet::new(),
        }
    }

    pub fn register(
        &mut self,
        descriptor: ComponentDescriptor,
    ) -> Result<ComponentId, RegistryError> {
        if self.names.contains(descriptor.name) {
            return Err(RegistryError::DuplicateComponent(descriptor.name));
        }
        if descriptor.name.trim().is_empty() {
            return Err(RegistryError::InvalidComponent(descriptor.name));
        }
        if descriptor.constructors.is_empty() {
            return Err(RegistryError::EmptyConstructorList(descriptor.name));
        }
        let mut methods = HashSet::new();
        for method in &descriptor.methods {
            if !methods.insert(method.name) {
                return Err(RegistryError::DuplicateMethod {
                    component: descriptor.name,
                    method: method.name,
                });
            }
        }

        let id = ComponentId(self.descriptors.len() as u32);
        self.names.insert(descriptor.name);
        self.descriptors.push(Arc::new(descriptor));
        Ok(id)
    }

    /// Publishes the catalog, consuming the builder.
    pub fn freeze(self) -> FrozenComponentRegistry {
        FrozenComponentRegistry {
            descriptors: self.descriptors,
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A validated catalog. A registered component is looked up by the id a
/// description carries.
#[derive(Clone, Default)]
pub struct FrozenComponentRegistry {
    descriptors: Vec<Arc<ComponentDescriptor>>,
}

impl FrozenComponentRegistry {
    pub fn descriptors(&self) -> impl ExactSizeIterator<Item = &ComponentDescriptor> {
        self.descriptors.iter().map(Arc::as_ref)
    }

    pub fn descriptor(&self, id: u32) -> Option<&ComponentDescriptor> {
        self.descriptors.get(id as usize).map(Arc::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::div;

    struct EmptyMaterializer;

    impl ComponentMaterializer for EmptyMaterializer {
        fn materialize(&self, request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
            request.finish(div())
        }
    }

    fn nullary(export: &'static str) -> ConstructorDescriptor {
        ConstructorDescriptor::new(export, Vec::new(), |_| Ok(ComponentPayload::new(())))
    }

    #[test]
    fn a_registered_component_resolves_by_id() {
        let mut registry = ComponentRegistry::new();
        let id = registry
            .register(
                ComponentDescriptor::new("Button", Arc::new(EmptyMaterializer))
                    .with_constructors(vec![nullary("Button")]),
            )
            .unwrap();

        let frozen = registry.freeze();
        assert_eq!(
            frozen.descriptor(id.as_u32()).map(|d| d.name()),
            Some("Button")
        );
        assert!(frozen.descriptor(99).is_none());
    }

    #[test]
    fn duplicate_components_and_methods_are_rejected() {
        let mut registry = ComponentRegistry::new();
        registry
            .register(
                ComponentDescriptor::new("Button", Arc::new(EmptyMaterializer))
                    .with_constructors(vec![nullary("Button")]),
            )
            .unwrap();
        assert_eq!(
            registry
                .register(
                    ComponentDescriptor::new("Button", Arc::new(EmptyMaterializer))
                        .with_constructors(vec![nullary("Button")]),
                )
                .unwrap_err(),
            RegistryError::DuplicateComponent("Button")
        );

        let mut registry = ComponentRegistry::new();
        let duplicate =
            MethodDescriptor::new("label", Vec::new(), |_| Ok(ComponentPayload::new(())));
        assert_eq!(
            registry
                .register(
                    ComponentDescriptor::new("Button", Arc::new(EmptyMaterializer))
                        .with_constructors(vec![nullary("Button")])
                        .with_methods(vec![duplicate.clone(), duplicate]),
                )
                .unwrap_err(),
            RegistryError::DuplicateMethod {
                component: "Button",
                method: "label",
            }
        );
    }
}
