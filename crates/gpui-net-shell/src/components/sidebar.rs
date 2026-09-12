//! `Sidebar` family, ported from `component-shell`'s `navigation/sidebar.rs`.
//!
//! `Sidebar` accepts `SidebarMenu` children and named `header`/`footer` slots;
//! `SidebarMenu` accepts `SidebarMenuItem` children, which may nest.

use std::sync::Arc;

use gpui::{div, AnyElement, IntoElement as _, ParentElement as _, Refineable as _, Styled as _};
use gpui_component::sidebar::{
    Sidebar, SidebarCollapsible, SidebarFooter, SidebarHeader, SidebarMenu, SidebarMenuItem,
    SidebarToggleButton,
};
use gpui_component::{IconName, Selectable as _, Side};

use super::common::nonempty_id;
use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};
use crate::typed_child::Carrier;

#[derive(Clone)]
struct IdPayload(String);

#[derive(Clone, Copy)]
struct Empty;

#[derive(Clone)]
enum SidebarOp {
    Side(Side),
    Collapsible(SidebarCollapsible),
    Collapsed(bool),
}

#[derive(Clone)]
enum ToggleOp {
    Side(Side),
    Collapsed(bool),
}

#[derive(Clone)]
enum ItemOp {
    DefaultOpen(bool),
    ClickToOpen(bool),
    ClickToToggle(bool),
    Icon(IconName),
}

impl ItemOp {
    fn icon(value: &str) -> Result<Self, String> {
        Ok(ItemOp::Icon(match value {
            "home" => IconName::SquareTerminal,
            "components" => IconName::LayoutDashboard,
            "settings" => IconName::Settings2,
            "archive" => IconName::BookOpen,
            "account" => IconName::User,
            _ => return Err(format!("unsupported SidebarMenuItem icon `{value}`")),
        }))
    }
}

struct MenuItemMaterializer;

impl ComponentMaterializer for MenuItemMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let label = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "SidebarMenuItem received an incompatible payload".to_string())?
            .0
            .clone();
        let mut item = SidebarMenuItem::new(label)
            .active(request.selected())
            .disable(request.disabled());
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ItemOp>())
        {
            item = match operation {
                ItemOp::DefaultOpen(value) => item.default_open(*value),
                ItemOp::ClickToOpen(value) => item.click_to_open(*value),
                ItemOp::ClickToToggle(value) => item.click_to_toggle(*value),
                ItemOp::Icon(value) => item.icon(value.clone()),
            };
        }
        if let Some(token) = request.on_click() {
            let host = request.host().clone();
            item = item.on_click(move |_event, _window, cx| {
                if let Some(click) = host.callbacks.click {
                    // SAFETY: managed callback; it copies anything it keeps.
                    unsafe {
                        let _ = click(host.session_id, token);
                    }
                }
                (host.invalidate)(cx);
            });
        }
        let nested = request.take_typed_children::<SidebarMenuItem>(&["SidebarMenuItem"])?;
        if !nested.is_empty() {
            item = item.children(nested);
        }
        Ok(Carrier::new(item).into_any_element())
    }
}

struct MenuMaterializer;

impl ComponentMaterializer for MenuMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut menu = SidebarMenu::new();
        menu.style().refine(&request.take_style());
        let items = request.take_typed_children::<SidebarMenuItem>(&["SidebarMenuItem"])?;
        for item in items {
            menu = menu.child(item);
        }
        Ok(Carrier::new(menu).into_any_element())
    }
}

struct HeaderMaterializer;

impl ComponentMaterializer for HeaderMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut header = SidebarHeader::new().selected(request.selected());
        header.style().refine(&request.take_style());
        header.extend(request.take_children());
        Ok(header.into_any_element())
    }
}

struct FooterMaterializer;

impl ComponentMaterializer for FooterMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut footer = SidebarFooter::new().selected(request.selected());
        footer.style().refine(&request.take_style());
        footer.extend(request.take_children());
        Ok(footer.into_any_element())
    }
}

struct SidebarMaterializer;

impl ComponentMaterializer for SidebarMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let id = request
            .payload()
            .downcast_ref::<IdPayload>()
            .ok_or_else(|| "Sidebar received an incompatible payload".to_string())?
            .0
            .clone();
        let operations = request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<SidebarOp>().cloned())
            .collect::<Vec<_>>();
        let mut sidebar = Sidebar::<SidebarMenu>::new(id);
        for operation in operations {
            sidebar = match operation {
                SidebarOp::Side(value) => sidebar.side(value),
                SidebarOp::Collapsible(value) => sidebar.collapsible(value),
                SidebarOp::Collapsed(value) => sidebar.collapsed(value),
            };
        }
        if let Some(header) = request.take_slot("header")? {
            sidebar = sidebar.header(header);
        }
        if let Some(footer) = request.take_slot("footer")? {
            sidebar = sidebar.footer(footer);
        }
        sidebar.style().refine(&request.take_style());
        let menus = request.take_typed_children::<SidebarMenu>(&["SidebarMenu"])?;
        for menu in menus {
            sidebar = sidebar.child(menu);
        }
        Ok(sidebar.into_any_element())
    }
}

struct ToggleMaterializer;

impl ComponentMaterializer for ToggleMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let mut toggle = SidebarToggleButton::new();
        for operation in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ToggleOp>())
        {
            toggle = match operation {
                ToggleOp::Side(value) => toggle.side(*value),
                ToggleOp::Collapsed(value) => toggle.collapsed(*value),
            };
        }
        if let Some(token) = request.on_click() {
            let host = request.host().clone();
            toggle = toggle.on_click(move |_event, _window, cx| {
                if let Some(click) = host.callbacks.click {
                    // SAFETY: managed callback; it copies anything it keeps.
                    unsafe {
                        let _ = click(host.session_id, token);
                    }
                }
                (host.invalidate)(cx);
            });
        }
        if request.children_len() != 0 {
            return Err("SidebarToggleButton does not accept children".to_string());
        }
        let mut wrapper = div().child(toggle);
        wrapper.style().refine(&request.take_style());
        Ok(wrapper.into_any_element())
    }
}

fn nullary(export: &'static str) -> ConstructorDescriptor {
    ConstructorDescriptor::new(export, vec![], |_| Ok(ComponentPayload::new(Empty)))
}

fn bool_method<T: Send + Sync + 'static>(
    owner: &'static str,
    name: &'static str,
    docs: &'static str,
    make: fn(bool) -> T,
) -> MethodDescriptor {
    MethodDescriptor::new(
        name,
        vec![ArgumentDescriptor::new(name, ArgumentSchema::Boolean)],
        move |arguments| match arguments {
            [ComponentArgument::Boolean(value)] => Ok(ComponentPayload::new(make(*value))),
            _ => Err(format!("{owner}.{name} expects one boolean")),
        },
    )
    .with_documentation(docs)
}

fn side_method<T: Send + Sync + 'static>(
    owner: &'static str,
    make: fn(Side) -> T,
) -> MethodDescriptor {
    MethodDescriptor::new(
        "side",
        vec![ArgumentDescriptor::new(
            "side",
            ArgumentSchema::Enum(&["left", "right"]),
        )],
        move |arguments| match arguments {
            [ComponentArgument::Enum(value)] => match value.as_str() {
                "left" => Ok(ComponentPayload::new(make(Side::Left))),
                "right" => Ok(ComponentPayload::new(make(Side::Right))),
                _ => Err(format!("unsupported {owner} side `{value}`")),
            },
            _ => Err(format!("{owner}.side expects `left` or `right`")),
        },
    )
    .with_documentation("Sets the physical side occupied by the sidebar control.")
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("SidebarMenuItem", Arc::new(MenuItemMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "SidebarMenuItem",
                    vec![ArgumentDescriptor::new("label", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(value)] if !value.is_empty() => {
                            Ok(ComponentPayload::new(IdPayload(value.clone())))
                        }
                        _ => Err("SidebarMenuItem(label) expects a non-empty string".into()),
                    },
                )])
                .with_methods(vec![
                    bool_method(
                        "SidebarMenuItem",
                        "default_open",
                        "Sets the initial submenu disclosure state.",
                        ItemOp::DefaultOpen,
                    ),
                    bool_method(
                        "SidebarMenuItem",
                        "click_to_open",
                        "Lets a row click open its submenu.",
                        ItemOp::ClickToOpen,
                    ),
                    bool_method(
                        "SidebarMenuItem",
                        "click_to_toggle",
                        "Lets a row click toggle its submenu.",
                        ItemOp::ClickToToggle,
                    ),
                    MethodDescriptor::new(
                        "icon",
                        vec![ArgumentDescriptor::new(
                            "icon",
                            ArgumentSchema::Enum(&[
                                "home",
                                "components",
                                "settings",
                                "archive",
                                "account",
                            ]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => {
                                ItemOp::icon(value).map(ComponentPayload::new)
                            }
                            _ => Err("SidebarMenuItem.icon expects one icon name".into()),
                        },
                    )
                    .with_documentation("Sets the navigation icon."),
                ])
                .with_documentation(
                    "A typed navigation row accepted by SidebarMenu. Shell style is unsupported.",
                ),
        )
        .expect("the built-in SidebarMenuItem descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("SidebarMenu", Arc::new(MenuMaterializer))
                .with_constructors(vec![nullary("SidebarMenu")])
                .with_methods(vec![])
                .with_documentation("A typed Sidebar menu accepting SidebarMenuItem children."),
        )
        .expect("the built-in SidebarMenu descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("SidebarHeader", Arc::new(HeaderMaterializer))
                .with_constructors(vec![nullary("SidebarHeader")])
                .with_methods(vec![])
                .with_documentation("A styled sidebar header accepting ordinary children."),
        )
        .expect("the built-in SidebarHeader descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("SidebarFooter", Arc::new(FooterMaterializer))
                .with_constructors(vec![nullary("SidebarFooter")])
                .with_methods(vec![])
                .with_documentation("A styled sidebar footer accepting ordinary children."),
        )
        .expect("the built-in SidebarFooter descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("Sidebar", Arc::new(SidebarMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Sidebar",
                    vec![ArgumentDescriptor::new("id", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(id)] => nonempty_id(id, "Sidebar")
                            .map(IdPayload)
                            .map(ComponentPayload::new),
                        _ => Err("Sidebar(id) expects a string id".into()),
                    },
                )])
                .with_methods(vec![
                    side_method("Sidebar", SidebarOp::Side),
                    MethodDescriptor::new(
                        "collapsible",
                        vec![ArgumentDescriptor::new(
                            "mode",
                            ArgumentSchema::Enum(&["icon", "offcanvas", "none"]),
                        )],
                        |arguments| match arguments {
                            [ComponentArgument::Enum(value)] => match value.as_str() {
                                "icon" => Ok(ComponentPayload::new(SidebarOp::Collapsible(
                                    SidebarCollapsible::Icon,
                                ))),
                                "offcanvas" => Ok(ComponentPayload::new(SidebarOp::Collapsible(
                                    SidebarCollapsible::Offcanvas,
                                ))),
                                "none" => Ok(ComponentPayload::new(SidebarOp::Collapsible(
                                    SidebarCollapsible::None,
                                ))),
                                _ => Err(format!("unsupported Sidebar collapsible mode `{value}`")),
                            },
                            _ => {
                                Err("Sidebar.collapsible expects `icon`, `offcanvas`, or `none`"
                                    .into())
                            }
                        },
                    )
                    .with_documentation("Sets the sidebar collapse behavior."),
                    bool_method(
                        "Sidebar",
                        "collapsed",
                        "Sets the controlled collapsed state.",
                        SidebarOp::Collapsed,
                    ),
                ])
                .with_documentation(
                    "A typed application sidebar accepting SidebarMenu children and named \
                     header/footer slots.",
                ),
        )
        .expect("the built-in Sidebar descriptor is valid");

    registry
        .register(
            ComponentDescriptor::new("SidebarToggleButton", Arc::new(ToggleMaterializer))
                .with_constructors(vec![nullary("SidebarToggleButton")])
                .with_methods(vec![
                    side_method("SidebarToggleButton", ToggleOp::Side),
                    bool_method(
                        "SidebarToggleButton",
                        "collapsed",
                        "Sets the icon for the current collapsed state.",
                        ToggleOp::Collapsed,
                    ),
                ])
                .with_documentation(
                    "A button that reflects sidebar side and collapsed state; on_click is \
                     forwarded.",
                ),
        )
        .expect("the built-in SidebarToggleButton descriptor is valid");
}
