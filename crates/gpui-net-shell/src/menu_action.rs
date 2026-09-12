//! One action type for a managed native-menu selection.
//!
//! GPUI actions are Rust types, and `NativeMenu` items are built from
//! `Box<dyn Action>`. A managed host cannot name a Rust type, so every native
//! menu item shares [`ManagedMenuAction`], one type carrying the managed
//! callback token. Dispatch is by `TypeId`, so one global listener hears all of
//! them and the token decides which handler runs — the same collapse the shell
//! runtime uses for script actions.

use gpui::{Action, Result};

/// One native-menu selection, carrying the managed callback token to run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedMenuAction {
    token: u64,
}

impl ManagedMenuAction {
    pub fn new(token: u64) -> Self {
        Self { token }
    }

    pub fn token(&self) -> u64 {
        self.token
    }
}

impl Action for ManagedMenuAction {
    fn boxed_clone(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }

    /// Compares the tokens, not just the types.
    fn partial_eq(&self, action: &dyn Action) -> bool {
        action
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| other.token == self.token)
    }

    fn name(&self) -> &'static str {
        Self::name_for_type()
    }

    fn name_for_type() -> &'static str {
        "gpui_net_shell::ManagedMenuAction"
    }

    fn build(value: gpui::private::serde_json::Value) -> Result<Box<dyn Action>> {
        let token = value
            .get("token")
            .and_then(gpui::private::serde_json::Value::as_u64)
            .ok_or_else(|| {
                gpui::private::anyhow::anyhow!(
                    "a `gpui_net_shell::ManagedMenuAction` needs its callback token: \
                     `{{ \"token\": 1 }}`"
                )
            })?;
        Ok(Box::new(Self { token }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_action_round_trips_its_token() {
        let action = ManagedMenuAction::new(7);
        assert_eq!(action.token(), 7);
        assert_eq!(action.name(), "gpui_net_shell::ManagedMenuAction");

        let built =
            ManagedMenuAction::build(gpui::private::serde_json::json!({ "token": 42 })).unwrap();
        let built = built.as_any().downcast_ref::<ManagedMenuAction>().unwrap();
        assert_eq!(built.token(), 42);

        assert!(ManagedMenuAction::build(gpui::private::serde_json::json!({})).is_err());
    }

    #[test]
    fn equality_is_by_token_not_only_type() {
        assert!(ManagedMenuAction::new(1).partial_eq(&ManagedMenuAction::new(1)));
        assert!(!ManagedMenuAction::new(1).partial_eq(&ManagedMenuAction::new(2)));
    }
}
