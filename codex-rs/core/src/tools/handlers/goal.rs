//! Built-in model tool handlers for persisted thread goals.
//!
//! The public tool contract intentionally splits goal creation from stopped
//! status updates: `create_goal` starts an active objective, while
//! `update_goal` can only mark the existing goal complete or blocked.

use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_protocol::protocol::ThreadGoal;
use codex_protocol::protocol::ThreadGoalStatus;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Write as _;

use crate::tools::handlers::goal_spec::CREATE_GOAL_TOOL_NAME;
use crate::tools::handlers::goal_spec::GET_GOAL_TOOL_NAME;
use crate::tools::handlers::goal_spec::UPDATE_GOAL_TOOL_NAME;
use crate::tools::handlers::goal_spec::create_get_goal_tool;

mod create_goal;
mod get_goal;
mod update_goal;

pub use create_goal::CreateGoalHandler;
pub use get_goal::GetGoalHandler;
pub use update_goal::UpdateGoalHandler;

pub struct GoalHandler;

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for GoalHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("goal")
    }

    fn spec(&self) -> ToolSpec {
        create_get_goal_tool()
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        match invocation.tool_name.name.as_str() {
            CREATE_GOAL_TOOL_NAME => CreateGoalHandler.handle(invocation).await,
            GET_GOAL_TOOL_NAME => GetGoalHandler.handle(invocation).await,
            UPDATE_GOAL_TOOL_NAME => UpdateGoalHandler.handle(invocation).await,
            other => Err(FunctionCallError::RespondToModel(format!(
                "unsupported goal tool `{other}`"
            ))),
        }
    }
}

impl CoreToolRuntime for GoalHandler {
    fn matches_kind(&self, payload: &crate::tools::context::ToolPayload) -> bool {
        matches!(payload, crate::tools::context::ToolPayload::Function { .. })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct CreateGoalArgs {
    objective: String,
    token_budget: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct UpdateGoalArgs {
    status: ThreadGoalStatus,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoalToolResponse {
    goal: Option<ThreadGoal>,
    remaining_tokens: Option<i64>,
    completion_budget_report: Option<String>,
}

#[derive(Clone, Copy)]
enum CompletionBudgetReport {
    Include,
    Omit,
}

impl GoalToolResponse {
    fn new(goal: Option<ThreadGoal>, report_mode: CompletionBudgetReport) -> Self {
        let remaining_tokens = goal.as_ref().and_then(|goal| {
            goal.token_budget
                .map(|budget| (budget - goal.tokens_used).max(0))
        });
        let completion_budget_report = match report_mode {
            CompletionBudgetReport::Include => goal
                .as_ref()
                .filter(|goal| goal.status == ThreadGoalStatus::Complete)
                .and_then(completion_budget_report),
            CompletionBudgetReport::Omit => None,
        };
        Self {
            goal,
            remaining_tokens,
            completion_budget_report,
        }
    }
}

fn format_goal_error(err: anyhow::Error) -> String {
    let mut message = err.to_string();
    for cause in err.chain().skip(1) {
        let _ = write!(message, ": {cause}");
    }
    message
}

fn goal_response(
    goal: Option<ThreadGoal>,
    completion_budget_report: CompletionBudgetReport,
) -> Result<FunctionToolOutput, FunctionCallError> {
    let response =
        serde_json::to_string_pretty(&GoalToolResponse::new(goal, completion_budget_report))
            .map_err(|err| FunctionCallError::Fatal(err.to_string()))?;
    Ok(FunctionToolOutput::from_text(response, Some(true)))
}

fn completion_budget_report(goal: &ThreadGoal) -> Option<String> {
    if goal.token_budget.is_none() && goal.time_used_seconds <= 0 {
        None
    } else {
        Some(
            "Goal achieved. Report final usage from this tool result's structured goal fields. If `goal.tokenBudget` is present, include token usage from `goal.tokensUsed` and `goal.tokenBudget`. If `goal.timeUsedSeconds` is greater than 0, summarize elapsed time in a concise, human-friendly form appropriate to the response language."
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::ThreadId;
    use pretty_assertions::assert_eq;

    #[test]
    fn completed_budgeted_goal_response_reports_final_usage() {
        let goal = ThreadGoal {
            thread_id: ThreadId::new(),
            objective: "Keep optimizing".to_string(),
            status: ThreadGoalStatus::Complete,
            token_budget: Some(10_000),
            tokens_used: 3_250,
            time_used_seconds: 75,
            created_at: 1,
            updated_at: 2,
        };

        let response = GoalToolResponse::new(Some(goal.clone()), CompletionBudgetReport::Include);

        assert_eq!(
            response,
            GoalToolResponse {
                goal: Some(goal),
                remaining_tokens: Some(6_750),
                completion_budget_report: Some(
                    "Goal achieved. Report final usage from this tool result's structured goal fields. If `goal.tokenBudget` is present, include token usage from `goal.tokensUsed` and `goal.tokenBudget`. If `goal.timeUsedSeconds` is greater than 0, summarize elapsed time in a concise, human-friendly form appropriate to the response language."
                        .to_string()
                ),
            }
        );
    }

    #[test]
    fn completed_unbudgeted_goal_response_omits_budget_report() {
        let goal = ThreadGoal {
            thread_id: ThreadId::new(),
            objective: "Write a poem".to_string(),
            status: ThreadGoalStatus::Complete,
            token_budget: None,
            tokens_used: 120,
            time_used_seconds: 0,
            created_at: 1,
            updated_at: 2,
        };

        let response = GoalToolResponse::new(Some(goal.clone()), CompletionBudgetReport::Include);

        assert_eq!(
            response,
            GoalToolResponse {
                goal: Some(goal),
                remaining_tokens: None,
                completion_budget_report: None,
            }
        );
    }
}
