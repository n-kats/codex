#![allow(non_snake_case)]

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn custom__slash_command_popup__custom_agents_is_visible_in_default_popup() {
    let mut popup = CommandPopup::new(CommandPopupFlags::default(), Vec::new());
    popup.on_composer_text_change("/".to_string());

    let items = popup.filtered_items();
    assert!(
        items.contains(&CommandItem::Builtin(SlashCommand::CustomAgents)),
        "expected /custom-agents to appear in the default popup list"
    );
}

#[test]
fn custom__slash_command_popup__custom_agents_is_selected_for_prefix() {
    let mut popup = CommandPopup::new(CommandPopupFlags::default(), Vec::new());
    popup.on_composer_text_change("/cu".to_string());

    assert_eq!(
        popup.selected_item(),
        Some(CommandItem::Builtin(SlashCommand::CustomAgents))
    );
}
