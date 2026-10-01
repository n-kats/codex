#![allow(non_snake_case)]

use super::code_mode_yield_time_ms;

#[test]
fn custom__mcp_tool_wait__overrides_requested_intermediate_yield() {
    assert_eq!(code_mode_yield_time_ms(Some(10_000), true), Some(u64::MAX));
}

#[test]
fn custom__mcp_tool_wait__preserves_yield_without_waiting_tools() {
    assert_eq!(code_mode_yield_time_ms(Some(10_000), false), Some(10_000));
    assert_eq!(code_mode_yield_time_ms(None, false), None);
}
