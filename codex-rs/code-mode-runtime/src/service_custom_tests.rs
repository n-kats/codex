#![allow(non_snake_case)]

use super::observe_mode;
use super::runtime::ObserveMode;
use super::yield_timeout;
use std::time::Duration;

#[test]
fn custom__mcp_tool_wait__maximum_yield_time_waits_until_completion() {
    assert_eq!(
        observe_mode(u64::MAX, Duration::from_millis(10_000)),
        ObserveMode::UntilCompletion
    );
    assert!(matches!(
        observe_mode(10_000, yield_timeout(10_000)),
        ObserveMode::YieldAfter(_)
    ));
}
