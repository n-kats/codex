pub(crate) mod agent_resolver;
pub(crate) mod control;
#[cfg(test)]
pub(crate) mod mailbox;
mod registry;
pub(crate) mod role;
pub(crate) mod status;

pub(crate) type AgentStatus = codex_protocol::protocol::AgentStatus;
pub(crate) type AgentControl = control::AgentControl;

pub(crate) fn exceeds_thread_spawn_depth_limit(depth: i32, max_depth: i32) -> bool {
    registry::exceeds_thread_spawn_depth_limit(depth, max_depth)
}

pub(crate) fn next_thread_spawn_depth(
    session_source: &codex_protocol::protocol::SessionSource,
) -> i32 {
    registry::next_thread_spawn_depth(session_source)
}

pub(crate) fn agent_status_from_event(
    msg: &codex_protocol::protocol::EventMsg,
) -> Option<AgentStatus> {
    status::agent_status_from_event(msg)
}
