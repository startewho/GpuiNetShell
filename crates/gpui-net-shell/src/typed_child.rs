//! Handing a typed native value from a child materializer to its parent.
//!
//! The shell materializes a child into an erased [`gpui::AnyElement`], but a
//! native parent such as `TabBar`, `Accordion`, or `Stepper` needs the concrete
//! Rust value its builder takes, not an element. [`Carrier`] is the one element
//! type that carries such a value through that erasure, and [`take_typed`] is
//! the only way back out.
//!
//! Adapted from `component-shell`'s `typed_child.rs`, with `String` errors.

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window,
};

/// An element that renders nothing and exists to carry `T` to its parent.
pub(crate) struct Carrier<T: 'static>(Option<T>);

impl<T: 'static> Carrier<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T: 'static> IntoElement for Carrier<T> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl<T: 'static> Element for Carrier<T> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, AnyElement) {
        let mut element = gpui::div().into_any_element();
        let id = element.request_layout(window, cx);
        (id, element)
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut AnyElement,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// Takes the typed value a child carried, exactly once.
///
/// `name` names the child the parent expected, so nesting the wrong component
/// reads which one was wrong rather than a downcast failure.
pub(crate) fn take_typed<T: 'static>(element: &mut AnyElement, name: &str) -> Result<T, String> {
    element
        .downcast_mut::<Carrier<T>>()
        .ok_or_else(|| format!("{name} materialized an incompatible child"))?
        .0
        .take()
        .ok_or_else(|| format!("{name} child was already consumed"))
}

/// An element that renders `T` when it stands alone and yields `T` to a typed
/// parent when one takes it.
///
/// [`Carrier`] renders nothing, which is right for a part that only ever exists
/// inside its parent (a `CommandItem`, a `TableRow`). A `Radio` is also usable
/// on its own, so it must render itself unless a `RadioGroup` consumes it.
pub(crate) struct Part<T: IntoElement + 'static>(Option<T>);

impl<T: IntoElement + 'static> Part<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Some(value))
    }

    fn take(&mut self) -> Option<T> {
        self.0.take()
    }
}

impl<T: IntoElement + 'static> IntoElement for Part<T> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl<T: IntoElement + 'static> Element for Part<T> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, AnyElement) {
        let mut element = match self.take() {
            Some(value) => value.into_any_element(),
            None => gpui::div().into_any_element(),
        };
        let id = element.request_layout(window, cx);
        (id, element)
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut AnyElement,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// Takes the typed value a rendering part carried, exactly once.
pub(crate) fn take_part<T: IntoElement + 'static>(
    element: &mut AnyElement,
    name: &str,
) -> Result<T, String> {
    element
        .downcast_mut::<Part<T>>()
        .ok_or_else(|| format!("{name} materialized an incompatible child"))?
        .take()
        .ok_or_else(|| format!("{name} child was already consumed"))
}
