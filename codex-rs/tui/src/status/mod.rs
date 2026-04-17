//! Status output formatting and display adapters for the TUI.
//!
//! This module turns protocol-level snapshots into stable display structures used by `/status`
//! output and footer/status-line helpers, while keeping rendering concerns out of transport-facing
//! code.
//!
//! `rate_limits` is the main integration point for status-line usage-limit items: it converts raw
//! window snapshots into local-time labels and classifies data as available, stale, or missing.
mod account;
mod card;
mod format;
pub(crate) mod helpers;
mod rate_limits;

pub(crate) type StatusAccountDisplay = account::StatusAccountDisplay;
#[cfg(test)]
pub(crate) fn new_status_output(
    config: &codex_core::config::Config,
    account_display: Option<&StatusAccountDisplay>,
    token_info: Option<&codex_protocol::protocol::TokenUsageInfo>,
    total_usage: &codex_protocol::protocol::TokenUsage,
    session_id: &Option<codex_protocol::ThreadId>,
    thread_name: Option<String>,
    forked_from: Option<codex_protocol::ThreadId>,
    rate_limits: Option<&RateLimitSnapshotDisplay>,
    plan_type: Option<codex_protocol::account::PlanType>,
    now: chrono::DateTime<chrono::Local>,
    model_name: &str,
    collaboration_mode: Option<&str>,
    reasoning_effort_override: Option<Option<codex_protocol::openai_models::ReasoningEffort>>,
) -> crate::history_cell::CompositeHistoryCell {
    card::new_status_output(
        config,
        account_display,
        token_info,
        total_usage,
        session_id,
        thread_name,
        forked_from,
        rate_limits,
        plan_type,
        now,
        model_name,
        collaboration_mode,
        reasoning_effort_override,
    )
}

pub(crate) fn new_status_output_with_rate_limits(
    config: &codex_core::config::Config,
    account_display: Option<&StatusAccountDisplay>,
    token_info: Option<&codex_protocol::protocol::TokenUsageInfo>,
    total_usage: &codex_protocol::protocol::TokenUsage,
    session_id: &Option<codex_protocol::ThreadId>,
    thread_name: Option<String>,
    forked_from: Option<codex_protocol::ThreadId>,
    rate_limits: &[RateLimitSnapshotDisplay],
    plan_type: Option<codex_protocol::account::PlanType>,
    now: chrono::DateTime<chrono::Local>,
    model_name: &str,
    collaboration_mode: Option<&str>,
    reasoning_effort_override: Option<Option<codex_protocol::openai_models::ReasoningEffort>>,
    refreshing_rate_limits: bool,
) -> crate::history_cell::CompositeHistoryCell {
    card::new_status_output_with_rate_limits(
        config,
        account_display,
        token_info,
        total_usage,
        session_id,
        thread_name,
        forked_from,
        rate_limits,
        plan_type,
        now,
        model_name,
        collaboration_mode,
        reasoning_effort_override,
        refreshing_rate_limits,
    )
}

pub(crate) fn format_directory_display(
    directory: &std::path::Path,
    max_width: Option<usize>,
) -> String {
    helpers::format_directory_display(directory, max_width)
}

pub(crate) fn format_tokens_compact(value: i64) -> String {
    helpers::format_tokens_compact(value)
}

pub(crate) type RateLimitSnapshotDisplay = rate_limits::RateLimitSnapshotDisplay;
pub(crate) type RateLimitWindowDisplay = rate_limits::RateLimitWindowDisplay;
#[cfg(test)]
pub(crate) fn rate_limit_snapshot_display(
    snapshot: &codex_protocol::protocol::RateLimitSnapshot,
    captured_at: chrono::DateTime<chrono::Local>,
) -> RateLimitSnapshotDisplay {
    rate_limits::rate_limit_snapshot_display(snapshot, captured_at)
}

pub(crate) fn rate_limit_snapshot_display_for_limit(
    snapshot: &codex_protocol::protocol::RateLimitSnapshot,
    limit_name: String,
    captured_at: chrono::DateTime<chrono::Local>,
) -> RateLimitSnapshotDisplay {
    rate_limits::rate_limit_snapshot_display_for_limit(snapshot, limit_name, captured_at)
}

#[cfg(test)]
mod tests;
