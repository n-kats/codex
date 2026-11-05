#![allow(non_snake_case)]

use super::SlashCommand;

#[test]
fn custom__slash_command__custom_agents_available_during_task() {
    assert!(SlashCommand::CustomAgents.available_during_task());
}
