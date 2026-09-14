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
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, ElementId, Entity, IntoElement, ParentElement, Refineable as _,
    StyleRefinement, Styled, Window,
};

use crate::context::HostContext;
use crate::snapshot::Snapshot;

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
    /// A child node index supplied as an element argument, materialized lazily.
    Element(u32),
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
            Self::Callback(_) | Self::Element(_) => true,
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
    /// A child element materialized lazily and passed to the method.
    Element,
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

/// A callback that renders an element subtree (P7).
///
/// The managed side builds a small arena for one element per invocation; the
/// host decodes it and materializes the result. This is what lets a list row or
/// table cell be rendered by managed code rather than a built-in text row.
#[derive(Clone)]
pub struct ElementCallback {
    registry: FrozenComponentRegistry,
    host: HostContext,
    token: u64,
}

impl ElementCallback {
    /// Renders the subtree for `arguments`, one callback argument per entry.
    pub fn build(
        &self,
        arguments: &[String],
        window: &mut Window,
        cx: &mut App,
    ) -> Result<AnyElement, String> {
        let Some(render) = self.host.callbacks.render_element else {
            return Err("the host has no render_element callback".into());
        };
        let payload = arguments.join("\n");
        let bytes = payload.as_bytes();
        let mut arena = crate::abi::GpuiNetArena::empty();
        let mut root = 0u32;
        // SAFETY: the managed callback fills a caller-owned arena and copies
        // anything it keeps; the buffers it names are only read by `decode`.
        let status = unsafe {
            render(
                self.host.session_id,
                self.token,
                bytes.as_ptr(),
                bytes.len() as u32,
                &mut arena,
                &mut root,
            )
        };
        if status != crate::schema::STATUS_OK {
            return Err(format!("element callback failed with status {status}"));
        }
        let snapshot = Snapshot::decode(&arena, root)
            .map_err(|code| format!("element snapshot decode failed with status {code}"))?;
        let factory = NodeFactory::new(&self.registry, Rc::new(snapshot), &self.host);
        factory.build(root, window, cx)
    }

    /// Returns a copy that never requests a full repaint when a managed callback
    /// fires. An entity subtree repaints only through `notify_entity`, so its
    /// callbacks are driven by an explicit `Context::notify` on the managed side.
    pub fn without_invalidate(mut self) -> Self {
        self.host = self.host.without_invalidate();
        self
    }
}

/// A materialized ordinary child, carrying the name of the component it came
/// from so a typed parent can validate or route it.
pub struct ChildElement {
    component: &'static str,
    element: AnyElement,
}

impl ChildElement {
    pub(crate) fn new(component: &'static str, element: AnyElement) -> Self {
        Self { component, element }
    }

    pub fn component(&self) -> &'static str {
        self.component
    }
}

/// One row returned by a managed row-snapshot callback (P4).
pub type Row = Vec<String>;

/// Rebuilds a node subtree on demand.
///
/// A lazy slot or element argument cannot reuse the eager materialization
/// produced for a parent: the shell hands an element to a closure that must be
/// able to produce it again (a popover's content, a status bar's region). This
/// factory owns everything the materializer needs — the frozen catalog, the
/// rendered snapshot, and the host — so it can rebuild any node at any time and
/// still be `'static`.
#[derive(Clone)]
pub struct NodeFactory {
    registry: FrozenComponentRegistry,
    snapshot: Rc<Snapshot>,
    host: HostContext,
}

impl NodeFactory {
    pub fn new(
        registry: &FrozenComponentRegistry,
        snapshot: Rc<Snapshot>,
        host: &HostContext,
    ) -> Self {
        Self {
            registry: registry.clone(),
            snapshot,
            host: host.clone(),
        }
    }

    pub fn registry(&self) -> &FrozenComponentRegistry {
        &self.registry
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn host(&self) -> &HostContext {
        &self.host
    }

    pub fn build(
        &self,
        node: u32,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<AnyElement, String> {
        crate::materialize::materialize_node(self, node, window, cx)
    }
}

/// A named slot that rebuilds its node each time it is taken.
#[derive(Clone)]
pub struct SlotFactory {
    factory: NodeFactory,
    node: u32,
}

impl SlotFactory {
    pub fn build(&self, window: &mut Window, cx: &mut App) -> Result<AnyElement, String> {
        self.factory.build(self.node, window, cx)
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
    factory: NodeFactory,
    style: Option<StyleRefinement>,
    children: Vec<ChildElement>,
    /// Named slots, held as node ids so they materialize only when taken.
    slots: Vec<(String, u32)>,
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
        factory: NodeFactory,
        style: StyleRefinement,
        children: Vec<ChildElement>,
        slots: Vec<(String, u32)>,
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
            factory,
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
        self.factory.host()
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
                host: self.factory.host().clone(),
                token: *token,
            }),
            other => Err(format!("expected a callback argument, got {other:?}")),
        }
    }

    /// Materializes an element argument (P3) on first use.
    pub fn resolve_element(&mut self, argument: &ComponentArgument) -> Result<AnyElement, String> {
        match argument {
            ComponentArgument::Element(node) => self.factory.build(*node, self.window, self.cx),
            other => Err(format!("expected an element argument, got {other:?}")),
        }
    }

    /// Resolves an element callback (P7) recorded by a component method.
    pub fn resolve_element_callback(
        &self,
        argument: &ComponentArgument,
    ) -> Result<ElementCallback, String> {
        match argument {
            ComponentArgument::Callback(token) => Ok(ElementCallback {
                registry: self.factory.registry().clone(),
                host: self.factory.host().clone(),
                token: *token,
            }),
            other => Err(format!("expected an element callback, got {other:?}")),
        }
    }

    /// Invokes a managed row-snapshot callback (P4) and parses its rows.
    ///
    /// The managed side writes newline-separated rows of tab-separated fields;
    /// a [`crate::schema::STATUS_TRUNCATED`] reply means its buffer was too
    /// small, so the call is retried with the size it reported.
    pub fn resolve_rows(&self, argument: &ComponentArgument) -> Result<Vec<Row>, String> {
        let token = match argument {
            ComponentArgument::Callback(token) => *token,
            other => return Err(format!("expected a rows callback, got {other:?}")),
        };
        let host = self.factory.host();
        let Some(resolve) = host.callbacks.resolve_rows else {
            return Err("the host has no resolve_rows callback".into());
        };

        let mut capacity = 64 * 1024usize;
        loop {
            let mut buffer = vec![0u8; capacity];
            let mut length = 0u32;
            // SAFETY: the buffer is live for the call and the managed side
            // writes at most `capacity` bytes before returning.
            let status = unsafe {
                resolve(
                    host.session_id,
                    token,
                    buffer.as_mut_ptr(),
                    capacity as u32,
                    &mut length,
                )
            };
            if status == crate::schema::STATUS_TRUNCATED {
                capacity = (length as usize).max(capacity * 2);
                if capacity > 8 * 1024 * 1024 {
                    return Err("row snapshot is too large".into());
                }
                continue;
            }
            if status != crate::schema::STATUS_OK {
                return Err(format!("row snapshot failed with status {status}"));
            }
            buffer.truncate(length as usize);
            let text =
                String::from_utf8(buffer).map_err(|_| "row snapshot is not UTF-8".to_string())?;
            return Ok(parse_rows(&text));
        }
    }

    pub fn children_len(&self) -> usize {
        self.children.len()
    }

    /// The component name of each ordinary child, in declaration order.
    pub fn child_component_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.children.iter().map(|child| child.component)
    }

    pub fn take_children(&mut self) -> Vec<AnyElement> {
        self.children.drain(..).map(|child| child.element).collect()
    }

    /// Takes ordinary children paired with the component name each came from, for
    /// a parent that accepts several registered child types.
    pub fn take_children_named(&mut self) -> Vec<(&'static str, AnyElement)> {
        self.children
            .drain(..)
            .map(|child| (child.component, child.element))
            .collect()
    }

    /// Takes ordinary children, requiring each to be one of `expected`.
    ///
    /// This is the typed-parent contract (P5): a `Menu` accepts only `MenuItem`
    /// children, and a script that nests anything else is told which component
    /// was wrong rather than failing silently.
    pub fn take_children_of(&mut self, expected: &[&str]) -> Result<Vec<AnyElement>, String> {
        let mut result = Vec::with_capacity(self.children.len());
        for child in self.children.drain(..) {
            if !expected.contains(&child.component) {
                return Err(format!(
                    "{} accepts only {} children; received {}",
                    self.component_name,
                    expected.join(" or "),
                    child.component
                ));
            }
            result.push(child.element);
        }
        Ok(result)
    }

    /// Takes ordinary children as typed values carried by [`crate::typed_child`],
    /// requiring each to be one of `expected`.
    pub fn take_typed_children<T: 'static>(&mut self, expected: &[&str]) -> Result<Vec<T>, String> {
        let mut result = Vec::with_capacity(self.children.len());
        for child in self.children.drain(..) {
            if !expected.contains(&child.component) {
                return Err(format!(
                    "{} accepts only {} children; received {}",
                    self.component_name,
                    expected.join(" or "),
                    child.component
                ));
            }
            let mut element = child.element;
            result.push(crate::typed_child::take_typed::<T>(
                &mut element,
                child.component,
            )?);
        }
        Ok(result)
    }

    /// Materializes a named slot, if the description supplied one.
    pub fn take_slot(&mut self, name: &str) -> Result<Option<AnyElement>, String> {
        match self.take_slot_factory(name) {
            Some(factory) => Ok(Some(factory.build(self.window, self.cx)?)),
            None => Ok(None),
        }
    }

    /// Takes a named slot as a rebuildable factory, if one was supplied.
    pub fn take_slot_factory(&mut self, name: &str) -> Option<SlotFactory> {
        let index = self.slots.iter().position(|(held, _)| held == name)?;
        let (_, node) = self.slots.remove(index);
        Some(SlotFactory {
            factory: self.factory.clone(),
            node,
        })
    }

    /// Runs `body` with the current window and app, for components that need
    /// window-scoped state (scroll handles, keyed element state).
    pub fn with_window_app<R>(&mut self, body: impl FnOnce(&mut Window, &mut App) -> R) -> R {
        body(self.window, self.cx)
    }

    /// Creates or retrieves a retained native entity for this component.
    ///
    /// This is the runtime's equivalent of `component-shell`'s `StateDescriptor`
    /// plus `ComponentArgument::Entity`: a component such as `Input` owns one
    /// `InputState` entity that survives across renders and is shared by every
    /// render that names the same `key`. The `init` closure runs only on the
    /// first render for that key, so it should capture the first render's
    /// configuration; later configuration changes mutate the entity through
    /// [`MaterializeRequest::update_entity`].
    ///
    /// `key` must be window-unique among retained entities (a component id is
    /// the usual choice). Sibling parts that share one entity pass the same key.
    pub fn use_keyed_state<S: 'static>(
        &mut self,
        key: impl Into<ElementId>,
        init: impl FnOnce(&mut Window, &mut Context<S>) -> S,
    ) -> Entity<S> {
        self.window.use_keyed_state(key.into(), self.cx, init)
    }

    /// Applies `update` to a retained entity, then requests a repaint.
    pub fn update_entity<S: 'static, R>(
        &mut self,
        entity: &Entity<S>,
        update: impl FnOnce(&mut S, &mut Context<S>) -> R,
    ) -> R {
        entity.update(self.cx, update)
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

/// Parses newline-separated rows of tab-separated fields.
fn parse_rows(text: &str) -> Vec<Row> {
    text.lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect()
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

    #[test]
    fn row_snapshots_parse_tab_separated_fields() {
        assert!(parse_rows("").is_empty());
        assert_eq!(
            parse_rows("a\tb\nc\td\n"),
            vec![
                vec!["a".to_string(), "b".to_string()],
                vec!["c".to_string(), "d".to_string()],
            ]
        );
        assert_eq!(parse_rows("only"), vec![vec!["only".to_string()]]);
    }
}
