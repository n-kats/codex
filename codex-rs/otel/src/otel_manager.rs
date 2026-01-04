use crate::otel_provider::traceparent_context_from_env;
use chrono::SecondsFormat;
use chrono::Utc;
use codex_api::Prompt as ApiPrompt;
use codex_api::ResponseEvent;
use codex_app_server_protocol::AuthMode;
use codex_protocol::ConversationId;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionSource;
use codex_protocol::user_input::UserInput;
use eventsource_stream::Event as StreamEvent;
use eventsource_stream::EventStreamError as StreamError;
use reqwest::Error;
use reqwest::Response;
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::Display;
use std::future::Future;
use std::time::Duration;
use std::time::Instant;
use strum_macros::Display;
use tokio::time::error::Elapsed;
use tracing::Instrument;
use tracing::Span;
use tracing::trace_span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

const MAX_LANGFUSE_OBSERVATION_JSON_CHARS: usize = 32_000;

#[derive(Debug)]
pub struct LlmGenerationRecorder {
    span: Span,
    wire_api: String,
    model: String,
    output_text: String,
    reasoning_content: String,
    reasoning_summary: String,
    output_items: Vec<ResponseItem>,
    response_id: Option<String>,
    token_usage: Option<serde_json::Value>,
}

impl LlmGenerationRecorder {
    fn new(otel: &OtelManager, wire_api: &str, model: &str, prompt: &ApiPrompt) -> Self {
        let prompt_json = serde_json::json!({
            "wire_api": wire_api,
            "model": model,
            "instructions": &prompt.instructions,
            "input": &prompt.input,
            "tools": &prompt.tools,
            "parallel_tool_calls": prompt.parallel_tool_calls,
            "output_schema": &prompt.output_schema,
        });

        let span = trace_span!("llm_generation", wire_api = wire_api, model = model);
        otel.attach_session_parent(&span);

        span.set_attribute("codex.observation.type", "generation".to_string());
        span.set_attribute("codex.observation.model.name", model.to_string());
        span.set_attribute(
            "codex.observation.input",
            serde_json::to_string(&prompt_json)
                .unwrap_or_else(|_| "{\"error\":\"failed to serialize prompt\"}".to_string()),
        );

        Self {
            span,
            wire_api: wire_api.to_string(),
            model: model.to_string(),
            output_text: String::new(),
            reasoning_content: String::new(),
            reasoning_summary: String::new(),
            output_items: Vec::new(),
            response_id: None,
            token_usage: None,
        }
    }

    pub fn on_event(&mut self, event: &ResponseEvent) {
        match event {
            ResponseEvent::OutputTextDelta(delta) => {
                self.output_text.push_str(delta);
            }
            ResponseEvent::ReasoningContentDelta { delta, .. } => {
                self.reasoning_content.push_str(delta);
            }
            ResponseEvent::ReasoningSummaryDelta { delta, .. } => {
                self.reasoning_summary.push_str(delta);
            }
            ResponseEvent::ReasoningSummaryPartAdded { .. } => {}
            ResponseEvent::OutputItemAdded(item) | ResponseEvent::OutputItemDone(item) => {
                self.output_items.push(item.clone());
            }
            _ => {}
        }
    }

    pub fn on_completed(
        &mut self,
        response_id: &str,
        token_usage: Option<&codex_protocol::protocol::TokenUsage>,
    ) {
        self.response_id = Some(response_id.to_string());
        self.token_usage = token_usage.and_then(|usage| serde_json::to_value(usage).ok());

        let output_json = serde_json::json!({
            "wire_api": &self.wire_api,
            "model": &self.model,
            "response_id": self.response_id,
            "output_text": &self.output_text,
            "reasoning_content": &self.reasoning_content,
            "reasoning_summary": &self.reasoning_summary,
            "output_items": &self.output_items,
            "token_usage": self.token_usage,
        });

        self.span.set_attribute(
            "codex.observation.output",
            serde_json::to_string(&output_json)
                .unwrap_or_else(|_| "{\"error\":\"failed to serialize output\"}".to_string()),
        );

        // Ensure the span is ended immediately (so it materializes in Langfuse even during long-lived TUI sessions).
        self.span.in_scope(|| {});
    }

    pub fn on_error(&mut self, error: &str) {
        let output_json = serde_json::json!({
            "wire_api": &self.wire_api,
            "model": &self.model,
            "response_id": self.response_id,
            "output_text": &self.output_text,
            "reasoning_content": &self.reasoning_content,
            "reasoning_summary": &self.reasoning_summary,
            "output_items": &self.output_items,
            "token_usage": self.token_usage,
            "error": error,
        });

        self.span.set_attribute(
            "codex.observation.output",
            serde_json::to_string(&output_json)
                .unwrap_or_else(|_| "{\"error\":\"failed to serialize output\"}".to_string()),
        );

        self.span.in_scope(|| {});
    }
}

#[derive(Debug, Clone, Serialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum ToolDecisionSource {
    Config,
    User,
}

#[derive(Debug, Clone)]
pub struct OtelEventMetadata {
    conversation_id: ConversationId,
    auth_mode: Option<String>,
    account_id: Option<String>,
    account_email: Option<String>,
    model: String,
    slug: String,
    log_user_prompts: bool,
    app_version: &'static str,
    terminal_type: String,
}

#[derive(Debug, Clone)]
pub struct OtelManager {
    metadata: OtelEventMetadata,
    session_parent_context: Option<opentelemetry::Context>,
}

impl OtelManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        conversation_id: ConversationId,
        model: &str,
        slug: &str,
        account_id: Option<String>,
        account_email: Option<String>,
        auth_mode: Option<AuthMode>,
        log_user_prompts: bool,
        terminal_type: String,
        session_source: SessionSource,
    ) -> OtelManager {
        // `new_session` is a trace root for the lifetime of the Codex session.
        //
        // Langfuse materializes OTEL spans/observations only once they have an end time. If we
        // keep the root span open for the entire interactive TUI session, child spans that
        // reference it can show up with a missing parent, and the trace row can be missing until
        // the session exits.
        //
        // To avoid this, we create a short-lived root span, capture its OTel context, and
        // immediately drop it. It is valid for a parent span to end before its children.
        let session_span = trace_span!(
            "new_session",
            conversation_id = %conversation_id,
            session_source = %session_source
        );

        // Make the Langfuse trace name match the resume session id for easier correlation.
        //
        // Collector remaps this to `langfuse.trace.name` for Langfuse ingest.
        let trace_name = format!("codex_{conversation_id}");
        session_span.set_attribute("codex.trace.name", trace_name.clone());
        session_span.set_attribute("session.id", conversation_id.to_string());

        if let Some(context) = traceparent_context_from_env() {
            let _ = session_span.set_parent(context);
        }

        let session_parent_context = Some(session_span.context());
        session_span.in_scope(|| {});

        Self {
            metadata: OtelEventMetadata {
                conversation_id,
                auth_mode: auth_mode.map(|m| m.to_string()),
                account_id,
                account_email,
                model: model.to_owned(),
                slug: slug.to_owned(),
                log_user_prompts,
                app_version: env!("CARGO_PKG_VERSION"),
                terminal_type,
            },
            session_parent_context,
        }
    }

    pub fn with_model(&self, model: &str, slug: &str) -> Self {
        let mut manager = self.clone();
        manager.metadata.model = model.to_owned();
        manager.metadata.slug = slug.to_owned();
        manager
    }

    pub fn session_parent_context(&self) -> Option<opentelemetry::Context> {
        self.session_parent_context.clone()
    }

    pub fn attach_session_parent(&self, span: &Span) {
        let conversation_id = self.metadata.conversation_id.to_string();
        span.set_attribute("session.id", conversation_id.clone());
        span.set_attribute("codex.trace.name", format!("codex_{conversation_id}"));

        if let Some(parent_context) = self.session_parent_context() {
            let _ = span.set_parent(parent_context);
        }
    }

    /// Starts a Langfuse generation observation for a model call, capturing full input and output.
    ///
    /// This is intentionally unredacted because it's meant for debugging how Codex calls the LLM.
    pub fn start_llm_generation(
        &self,
        wire_api: &str,
        model: &str,
        prompt: &ApiPrompt,
    ) -> LlmGenerationRecorder {
        LlmGenerationRecorder::new(self, wire_api, model, prompt)
    }

    pub fn record_responses(&self, handle_responses_span: &Span, event: &ResponseEvent) {
        handle_responses_span.record("otel.name", OtelManager::responses_type(event));

        match event {
            ResponseEvent::OutputItemDone(item) => {
                handle_responses_span.record("from", "output_item_done");
                if let ResponseItem::FunctionCall { name, .. } = &item {
                    handle_responses_span.record("tool_name", name.as_str());
                }
            }
            ResponseEvent::OutputItemAdded(item) => {
                handle_responses_span.record("from", "output_item_added");
                if let ResponseItem::FunctionCall { name, .. } = &item {
                    handle_responses_span.record("tool_name", name.as_str());
                }
            }
            ResponseEvent::Completed {
                response_id,
                token_usage,
            } => {
                handle_responses_span.set_attribute("response_id", response_id.clone());
                if let Some(usage) = token_usage {
                    handle_responses_span
                        .set_attribute("input_token_count", usage.input_tokens as i64);
                    handle_responses_span
                        .set_attribute("output_token_count", usage.output_tokens as i64);
                    handle_responses_span
                        .set_attribute("cached_token_count", usage.cached_input_tokens as i64);
                    handle_responses_span.set_attribute(
                        "reasoning_token_count",
                        usage.reasoning_output_tokens as i64,
                    );
                    handle_responses_span
                        .set_attribute("tool_token_count", usage.total_tokens as i64);
                }
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn conversation_starts(
        &self,
        provider_name: &str,
        reasoning_effort: Option<ReasoningEffort>,
        reasoning_summary: ReasoningSummary,
        context_window: Option<i64>,
        auto_compact_token_limit: Option<i64>,
        approval_policy: AskForApproval,
        sandbox_policy: SandboxPolicy,
        mcp_servers: Vec<&str>,
        active_profile: Option<String>,
    ) {
        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.conversation_starts",
            event.timestamp = %timestamp(),
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            provider_name = %provider_name,
            reasoning_effort = reasoning_effort.map(|e| e.to_string()),
            reasoning_summary = %reasoning_summary,
            context_window = context_window,
            auto_compact_token_limit = auto_compact_token_limit,
            approval_policy = %approval_policy,
            sandbox_policy = %sandbox_policy,
            mcp_servers = mcp_servers.join(", "),
            active_profile = active_profile,
        )
    }

    pub async fn log_request<F, Fut>(&self, attempt: u64, f: F) -> Result<Response, Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Response, Error>>,
    {
        let start = std::time::Instant::now();
        let response = f().await;
        let duration = start.elapsed();

        let (status, error) = match &response {
            Ok(response) => (Some(response.status().as_u16()), None),
            Err(error) => (error.status().map(|s| s.as_u16()), Some(error.to_string())),
        };
        self.record_api_request(attempt, status, error.as_deref(), duration);

        response
    }

    pub fn record_api_request(
        &self,
        attempt: u64,
        status: Option<u16>,
        error: Option<&str>,
        duration: Duration,
    ) {
        let api_request_span = trace_span!(
            "api_request",
            attempt = attempt,
            status = status,
            duration_ms = %duration.as_millis(),
            error.message = error,
        );
        self.attach_session_parent(&api_request_span);
        api_request_span.in_scope(|| {});

        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.api_request",
            event.timestamp = %timestamp(),
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            duration_ms = %duration.as_millis(),
            http.response.status_code = status,
            error.message = error,
            attempt = attempt,
        );
    }

    pub fn log_sse_event<E>(
        &self,
        response: &Result<Option<Result<StreamEvent, StreamError<E>>>, Elapsed>,
        duration: Duration,
    ) where
        E: Display,
    {
        match response {
            Ok(Some(Ok(sse))) => {
                if sse.data.trim() == "[DONE]" {
                    self.sse_event(&sse.event, duration);
                } else {
                    match serde_json::from_str::<serde_json::Value>(&sse.data) {
                        Ok(error) if sse.event == "response.failed" => {
                            self.sse_event_failed(Some(&sse.event), duration, &error);
                        }
                        Ok(content) if sse.event == "response.output_item.done" => {
                            match serde_json::from_value::<ResponseItem>(content) {
                                Ok(_) => self.sse_event(&sse.event, duration),
                                Err(_) => {
                                    self.sse_event_failed(
                                        Some(&sse.event),
                                        duration,
                                        &"failed to parse response.output_item.done",
                                    );
                                }
                            };
                        }
                        Ok(_) => {
                            self.sse_event(&sse.event, duration);
                        }
                        Err(error) => {
                            self.sse_event_failed(Some(&sse.event), duration, &error);
                        }
                    }
                }
            }
            Ok(Some(Err(error))) => {
                self.sse_event_failed(None, duration, error);
            }
            Ok(None) => {}
            Err(_) => {
                self.sse_event_failed(None, duration, &"idle timeout waiting for SSE");
            }
        }
    }

    fn sse_event(&self, kind: &str, duration: Duration) {
        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.sse_event",
            event.timestamp = %timestamp(),
            event.kind = %kind,
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            duration_ms = %duration.as_millis(),
        );
    }

    pub fn sse_event_failed<T>(&self, kind: Option<&String>, duration: Duration, error: &T)
    where
        T: Display,
    {
        match kind {
            Some(kind) => tracing::event!(
                tracing::Level::INFO,
                event.name = "codex.sse_event",
                event.timestamp = %timestamp(),
                event.kind = %kind,
                conversation.id = %self.metadata.conversation_id,
                app.version = %self.metadata.app_version,
                auth_mode = self.metadata.auth_mode,
                user.account_id = self.metadata.account_id,
                user.email = self.metadata.account_email,
                terminal.type = %self.metadata.terminal_type,
                model = %self.metadata.model,
                slug = %self.metadata.slug,
                duration_ms = %duration.as_millis(),
                error.message = %error,
            ),
            None => tracing::event!(
                tracing::Level::INFO,
                event.name = "codex.sse_event",
                event.timestamp = %timestamp(),
                conversation.id = %self.metadata.conversation_id,
                app.version = %self.metadata.app_version,
                auth_mode = self.metadata.auth_mode,
                user.account_id = self.metadata.account_id,
                user.email = self.metadata.account_email,
                terminal.type = %self.metadata.terminal_type,
                model = %self.metadata.model,
                slug = %self.metadata.slug,
                duration_ms = %duration.as_millis(),
                error.message = %error,
            ),
        }
    }

    pub fn see_event_completed_failed<T>(&self, error: &T)
    where
        T: Display,
    {
        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.sse_event",
            event.kind = %"response.completed",
            event.timestamp = %timestamp(),
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            error.message = %error,
        )
    }

    pub fn sse_event_completed(
        &self,
        input_token_count: i64,
        output_token_count: i64,
        cached_token_count: Option<i64>,
        reasoning_token_count: Option<i64>,
        tool_token_count: i64,
    ) {
        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.sse_event",
            event.timestamp = %timestamp(),
            event.kind = %"response.completed",
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            input_token_count = %input_token_count,
            output_token_count = %output_token_count,
            cached_token_count = cached_token_count,
            reasoning_token_count = reasoning_token_count,
            tool_token_count = %tool_token_count,
        );
    }

    pub fn user_prompt(&self, items: &[UserInput]) {
        let prompt = items
            .iter()
            .flat_map(|item| match item {
                UserInput::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<String>();

        let prompt_to_log = if self.metadata.log_user_prompts {
            prompt.as_str()
        } else {
            "[REDACTED]"
        };

        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.user_prompt",
            event.timestamp = %timestamp(),
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            prompt_length = %prompt.chars().count(),
            prompt = %prompt_to_log,
        );
    }

    pub fn tool_decision(
        &self,
        tool_name: &str,
        call_id: &str,
        decision: &ReviewDecision,
        source: ToolDecisionSource,
    ) {
        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.tool_decision",
            event.timestamp = %timestamp(),
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            tool_name = %tool_name,
            call_id = %call_id,
            decision = %decision.clone().to_string().to_lowercase(),
            source = %source.to_string(),
        );
    }

    pub fn tool_call(&self, tool_name: &str, call_id: &str, item: &ResponseItem, payload: &str) {
        let (payload_truncated, payload_was_truncated, payload_original_len) =
            truncate_for_langfuse_observation_json(payload);

        let item_json = serde_json::to_string(item).unwrap_or_else(|_| "{}".to_string());
        let (item_truncated, item_was_truncated, item_original_len) =
            truncate_for_langfuse_observation_json(&item_json);

        let input_json = serde_json::json!({
            "tool_name": tool_name,
            "call_id": call_id,
            "payload": payload_truncated,
            "payload_truncated": payload_was_truncated,
            "payload_original_len": payload_original_len,
            "response_item_json": item_truncated,
            "response_item_truncated": item_was_truncated,
            "response_item_original_len": item_original_len,
        });

        let tool_call_span = trace_span!("tool_call", tool_name = tool_name, call_id = call_id);
        self.attach_session_parent(&tool_call_span);
        tool_call_span.set_attribute("codex.observation.type", "span".to_string());
        tool_call_span.set_attribute(
            "codex.observation.input",
            serde_json::to_string(&input_json)
                .unwrap_or_else(|_| "{\"error\":\"failed to serialize tool call\"}".to_string()),
        );

        tool_call_span.in_scope(|| {});
    }

    pub async fn log_tool_result<F, Fut, E>(
        &self,
        tool_name: &str,
        call_id: &str,
        arguments: &str,
        f: F,
    ) -> Result<(String, bool), E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(String, bool), E>>,
        E: Display,
    {
        let (arguments_truncated, arguments_was_truncated, arguments_original_len) =
            truncate_for_langfuse_observation_json(arguments);

        let input_json = serde_json::json!({
            "tool_name": tool_name,
            "call_id": call_id,
            "arguments": arguments_truncated,
            "arguments_truncated": arguments_was_truncated,
            "arguments_original_len": arguments_original_len,
        });

        let tool_exec_span = trace_span!("tool_exec", tool_name = tool_name, call_id = call_id);
        self.attach_session_parent(&tool_exec_span);
        tool_exec_span.set_attribute("codex.observation.type", "span".to_string());
        tool_exec_span.set_attribute(
            "codex.observation.input",
            serde_json::to_string(&input_json)
                .unwrap_or_else(|_| "{\"error\":\"failed to serialize tool input\"}".to_string()),
        );

        let start = Instant::now();
        let result = f().instrument(tool_exec_span.clone()).await;
        let duration = start.elapsed();

        let (output, success) = match &result {
            Ok((preview, success)) => (Cow::Borrowed(preview.as_str()), *success),
            Err(error) => (Cow::Owned(error.to_string()), false),
        };

        let (output_truncated, output_was_truncated, output_original_len) =
            truncate_for_langfuse_observation_json(output.as_ref());

        let output_json = serde_json::json!({
            "tool_name": tool_name,
            "call_id": call_id,
            "success": success,
            "duration_ms": duration.as_millis(),
            "output": output_truncated,
            "output_truncated": output_was_truncated,
            "output_original_len": output_original_len,
        });

        tool_exec_span.set_attribute(
            "codex.observation.output",
            serde_json::to_string(&output_json)
                .unwrap_or_else(|_| "{\"error\":\"failed to serialize tool output\"}".to_string()),
        );

        self.tool_result(
            tool_name,
            call_id,
            arguments,
            duration,
            success,
            output.as_ref(),
        );

        result
    }

    pub fn log_tool_failed(&self, tool_name: &str, error: &str) {
        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.tool_result",
            event.timestamp = %timestamp(),
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            tool_name = %tool_name,
            duration_ms = %Duration::ZERO.as_millis(),
            success = %false,
            output = %error,
        );
    }

    pub fn tool_result(
        &self,
        tool_name: &str,
        call_id: &str,
        arguments: &str,
        duration: Duration,
        success: bool,
        output: &str,
    ) {
        let success_str = if success { "true" } else { "false" };

        tracing::event!(
            tracing::Level::INFO,
            event.name = "codex.tool_result",
            event.timestamp = %timestamp(),
            conversation.id = %self.metadata.conversation_id,
            app.version = %self.metadata.app_version,
            auth_mode = self.metadata.auth_mode,
            user.account_id = self.metadata.account_id,
            user.email = self.metadata.account_email,
            terminal.type = %self.metadata.terminal_type,
            model = %self.metadata.model,
            slug = %self.metadata.slug,
            tool_name = %tool_name,
            call_id = %call_id,
            arguments = %arguments,
            duration_ms = %duration.as_millis(),
            success = %success_str,
            output = %output,
        );
    }

    fn responses_type(event: &ResponseEvent) -> String {
        match event {
            ResponseEvent::Created => "created".into(),
            ResponseEvent::OutputItemDone(item) => OtelManager::responses_item_type(item),
            ResponseEvent::OutputItemAdded(item) => OtelManager::responses_item_type(item),
            ResponseEvent::Completed { .. } => "completed".into(),
            ResponseEvent::OutputTextDelta(_) => "text_delta".into(),
            ResponseEvent::ReasoningSummaryDelta { .. } => "reasoning_summary_delta".into(),
            ResponseEvent::ReasoningContentDelta { .. } => "reasoning_content_delta".into(),
            ResponseEvent::ReasoningSummaryPartAdded { .. } => {
                "reasoning_summary_part_added".into()
            }
            ResponseEvent::RateLimits(_) => "rate_limits".into(),
            ResponseEvent::ModelsEtag(_) => "models_etag".into(),
        }
    }

    fn responses_item_type(item: &ResponseItem) -> String {
        match item {
            ResponseItem::Message { role, .. } => format!("message_from_{role}"),
            ResponseItem::Reasoning { .. } => "reasoning".into(),
            ResponseItem::LocalShellCall { .. } => "local_shell_call".into(),
            ResponseItem::FunctionCall { .. } => "function_call".into(),
            ResponseItem::FunctionCallOutput { .. } => "function_call_output".into(),
            ResponseItem::CustomToolCall { .. } => "custom_tool_call".into(),
            ResponseItem::CustomToolCallOutput { .. } => "custom_tool_call_output".into(),
            ResponseItem::WebSearchCall { .. } => "web_search_call".into(),
            ResponseItem::GhostSnapshot { .. } => "ghost_snapshot".into(),
            ResponseItem::Compaction { .. } => "compaction".into(),
            ResponseItem::Other => "other".into(),
        }
    }
}

fn timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn truncate_for_langfuse_observation_json(value: &str) -> (String, bool, usize) {
    let original_len = value.chars().count();
    if original_len <= MAX_LANGFUSE_OBSERVATION_JSON_CHARS {
        return (value.to_string(), false, original_len);
    }

    let truncated = value
        .chars()
        .take(MAX_LANGFUSE_OBSERVATION_JSON_CHARS)
        .collect::<String>();
    (truncated, true, original_len)
}
