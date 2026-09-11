//! The built-in component catalog.
//!
//! Each component is one module with its payloads, its registered methods, and
//! its `ComponentMaterializer`. The runtime knows none of them by name: adding
//! a component is `register` here plus one id in the wire schema.

pub mod badge;
pub mod button;
pub mod combobox;
mod common;
pub mod div;
pub mod label;
pub mod popover;
pub mod progress;
pub mod radio;
pub mod resizable;
pub mod scroll;
pub mod tabs;
pub mod text;

use crate::registry::{ComponentRegistry, FrozenComponentRegistry};

/// Registers every built-in component. The order is the wire id order.
pub fn register(registry: &mut ComponentRegistry) {
    div::register(registry);
    text::register(registry);
    button::register(registry);
    label::register(registry);
    badge::register(registry);
    progress::register(registry);
    combobox::register(registry);
    radio::register(registry);
    tabs::register(registry);
    scroll::register(registry);
    resizable::register(registry);
    popover::register(registry);
}

/// Builds and freezes the built-in catalog.
pub fn catalog() -> FrozenComponentRegistry {
    let mut registry = ComponentRegistry::new();
    register(&mut registry);
    registry.freeze()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalog_registers_components_in_stable_id_order() {
        let frozen = catalog();
        assert_eq!(
            frozen
                .descriptors()
                .map(|descriptor| descriptor.name())
                .collect::<Vec<_>>(),
            [
                "Div",
                "Text",
                "Button",
                "Label",
                "Badge",
                "Progress",
                "Combobox",
                "Radio",
                "Tabs",
                "Scroll",
                "Scrollbar",
                "Resizable",
                "Popover",
            ]
        );
    }
}
