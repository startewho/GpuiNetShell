//! The built-in component catalog.
//!
//! Each component is one module with its payloads, its registered methods, and
//! its `ComponentMaterializer`. The runtime knows none of them by name: adding
//! a component is `register` here plus one id in the wire schema.

pub mod accordion;
pub mod alert;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod clipboard;
pub mod collapsible;
pub mod combobox;
mod common;
pub mod data_table;
pub mod description_list;
pub mod div;
pub mod dropdown_button;
pub mod dropdown_menu;
pub mod form;
pub mod group_box;
pub mod hover_card;
pub mod icon;
pub mod input;
pub mod kbd;
pub mod label;
pub mod link;
pub mod list;
pub mod number_input;
pub mod otp_input;
pub mod pagination;
pub mod popover;
pub mod progress;
pub mod radio;
pub mod rating;
pub mod resizable;
pub mod scroll;
pub mod select;
pub mod separator;
pub mod skeleton;
pub mod slider;
pub mod spinner;
pub mod status_bar;
pub mod stepper;
pub mod tab_bar;
pub mod tabs;
pub mod tag;
pub mod text;
pub mod textarea;
pub mod tooltip;

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
    spinner::register(registry);
    separator::register(registry);
    skeleton::register(registry);
    tag::register(registry);
    link::register(registry);
    kbd::register(registry);
    avatar::register(registry);
    icon::register(registry);
    collapsible::register(registry);
    pagination::register(registry);
    rating::register(registry);
    clipboard::register(registry);
    breadcrumb::register(registry);
    group_box::register(registry);
    status_bar::register(registry);
    alert::register(registry);
    tooltip::register(registry);
    hover_card::register(registry);
    dropdown_menu::register(registry);
    dropdown_button::register(registry);
    tab_bar::register(registry);
    list::register(registry);
    select::register(registry);
    data_table::register(registry);
    accordion::register(registry);
    stepper::register(registry);
    description_list::register(registry);
    form::register(registry);
    input::register(registry);
    number_input::register(registry);
    textarea::register(registry);
    otp_input::register(registry);
    slider::register(registry);
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
                "Spinner",
                "Separator",
                "Skeleton",
                "Tag",
                "Link",
                "Kbd",
                "Avatar",
                "Icon",
                "Collapsible",
                "Pagination",
                "Rating",
                "Clipboard",
                "Breadcrumb",
                "GroupBox",
                "StatusBar",
                "Alert",
                "Tooltip",
                "HoverCard",
                "DropdownMenu",
                "DropdownButton",
                "Tab",
                "TabBar",
                "List",
                "Select",
                "DataTable",
                "AccordionItem",
                "Accordion",
                "StepperItem",
                "Stepper",
                "DescriptionItem",
                "DescriptionList",
                "Field",
                "Form",
                "Input",
                "NumberInput",
                "Textarea",
                "OtpInput",
                "Slider",
            ]
        );
    }
}
