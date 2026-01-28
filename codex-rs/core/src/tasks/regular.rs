use std::sync::Arc;

use crate::codex::TurnContext;
use crate::codex::run_turn;
use crate::state::TaskKind;
use async_trait::async_trait;
use codex_protocol::user_input::UserInput;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;
use tracing::trace_span;

use super::SessionTask;
use super::SessionTaskContext;

#[derive(Clone, Copy, Default)]
pub(crate) struct RegularTask;

#[async_trait]
impl SessionTask for RegularTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    async fn run(
        self: Arc<Self>,
        session: Arc<SessionTaskContext>,
        ctx: Arc<TurnContext>,
        input: Vec<UserInput>,
        cancellation_token: CancellationToken,
    ) -> Option<String> {
        let sess = session.clone_session();
        let turn_span = trace_span!("turn", turn_id = %ctx.sub_id);
        sess.set_server_reasoning_included(false).await;
        sess.services
            .otel_manager
            .apply_traceparent_parent(&turn_span);
        sess.services.otel_manager.attach_session_parent(&turn_span);
        run_turn(sess, ctx, input, cancellation_token)
            .instrument(turn_span)
            .await
    }
}
