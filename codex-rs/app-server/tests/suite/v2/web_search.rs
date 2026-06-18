use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use app_test_support::ChatGptAuthFixture;
use app_test_support::McpProcess;
use app_test_support::to_response;
use app_test_support::write_chatgpt_auth;
use axum::Router;
use codex_app_server_protocol::ItemCompletedNotification;
use codex_app_server_protocol::ItemStartedNotification;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::ThreadReadParams;
use codex_app_server_protocol::ThreadReadResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_app_server_protocol::WebSearchAction;
use codex_config::types::AuthCredentialsStoreMode;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use rmcp::handler::server::ServerHandler;
use rmcp::model::JsonObject;
use rmcp::model::ListToolsResult;
use rmcp::model::Meta;
use rmcp::model::ServerCapabilities;
use rmcp::model::ServerInfo;
use rmcp::model::Tool;
use rmcp::model::ToolAnnotations;
use rmcp::service::RequestContext;
use rmcp::service::RoleServer;
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::transport::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

// macOS and Windows Bazel CI can spend tens of seconds starting app-server
// subprocesses or processing test RPCs under load.
#[cfg(any(target_os = "macos", windows))]
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(60);
#[cfg(not(any(target_os = "macos", windows)))]
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::test]
async fn standalone_web_search_round_trips_encrypted_output() -> Result<()> {
    let call_id = "web-run-1";
    let responses_server = responses::start_mock_server().await;
    mount_search_response(&responses_server).await;
    let (apps_server_url, apps_server_handle) = start_apps_server().await?;

    let response_mock = responses::mount_sse_sequence(
        &responses_server,
        vec![
            responses::sse(vec![
                responses::ev_response_created("resp-1"),
                responses::ev_function_call_with_namespace(
                    call_id,
                    "web",
                    "run",
                    &json!({
                        "search_query": [{"q": "standalone web search"}],
                    })
                    .to_string(),
                ),
                responses::ev_completed("resp-1"),
            ]),
            responses::sse(vec![
                responses::ev_assistant_message("msg-1", "Done"),
                responses::ev_completed("resp-2"),
            ]),
        ],
    )
    .await;

    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &responses_server.uri(), &apps_server_url)?;
    write_chatgpt_auth(
        codex_home.path(),
        ChatGptAuthFixture::new("access-chatgpt"),
        AuthCredentialsStoreMode::File,
    )?;

    let mut mcp = McpProcess::new_with_env(codex_home.path(), &[("OPENAI_API_KEY", None)]).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let thread_req = mcp
        .send_thread_start_request(ThreadStartParams::default())
        .await?;
    let thread_resp: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_req)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(thread_resp)?;
    let thread_id = thread.id.clone();

    let turn_req = mcp
        .send_turn_start_request(TurnStartParams {
            thread_id: thread_id.clone(),
            client_user_message_id: None,
            input: vec![V2UserInput::Text {
                text: "Search the web".to_string(),
                text_elements: Vec::new(),
            }],
            ..Default::default()
        })
        .await?;
    let turn_resp: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(turn_req)),
    )
    .await??;
    let _turn: TurnStartResponse = to_response::<TurnStartResponse>(turn_resp)?;

    let started = timeout(DEFAULT_READ_TIMEOUT, wait_for_web_search_started(&mut mcp)).await??;
    let completed = timeout(
        DEFAULT_READ_TIMEOUT,
        wait_for_web_search_completed(&mut mcp),
    )
    .await??;

    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 2);

    let first_response = requests[0].body_json();
    let web_run = requests[0]
        .tool_by_name("web", "run")
        .context("web.run should be sent to the model")?;
    assert_eq!(
        web_run.pointer("/parameters/properties/time/description"),
        Some(&json!("Get time for the given UTC offsets."))
    );
    assert!(
        !has_hosted_web_search(&first_response),
        "standalone web search should replace hosted web search"
    );

    let search_body = search_request_body(&responses_server).await?;
    assert_eq!(search_body["model"], json!("mock-model"));
    assert_eq!(
        search_body["commands"],
        json!({
            "search_query": [{"q": "standalone web search"}],
        })
    );
    assert_eq!(
        search_body["settings"]["allowed_callers"],
        json!(["direct"])
    );
    assert_eq!(
        search_body["input"]
            .as_array()
            .context("search input should be an array")?
            .last(),
        Some(&json!({
            "type": "message",
            "role": "user",
            "content": [{"type": "input_text", "text": "Search the web"}],
        }))
    );

    assert_eq!(
        requests[1].function_call_output(call_id),
        json!({
            "type": "function_call_output",
            "call_id": call_id,
            "output": [{
                "type": "encrypted_content",
                "encrypted_content": "ciphertext",
            }],
        })
    );
    assert_eq!(
        started.item,
        ThreadItem::WebSearch {
            id: call_id.to_string(),
            query: String::new(),
            action: Some(WebSearchAction::Other),
        }
    );
    let expected_completed_item = ThreadItem::WebSearch {
        id: call_id.to_string(),
        query: "standalone web search".to_string(),
        action: Some(WebSearchAction::Search {
            query: Some("standalone web search".to_string()),
            queries: None,
        }),
    };
    assert_eq!(completed.item, expected_completed_item);

    drop(mcp);
    let mut reloaded_mcp =
        McpProcess::new_with_env(codex_home.path(), &[("OPENAI_API_KEY", None)]).await?;
    timeout(DEFAULT_READ_TIMEOUT, reloaded_mcp.initialize()).await??;
    let read_req = reloaded_mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id,
            include_turns: true,
        })
        .await?;
    let read_resp: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        reloaded_mcp.read_stream_until_response_message(RequestId::Integer(read_req)),
    )
    .await??;
    let ThreadReadResponse { thread, .. } = to_response::<ThreadReadResponse>(read_resp)?;
    let persisted_web_searches: Vec<&ThreadItem> = thread
        .turns
        .iter()
        .flat_map(|turn| &turn.items)
        .filter(|item| matches!(item, ThreadItem::WebSearch { .. }))
        .collect();
    assert_eq!(persisted_web_searches, vec![&expected_completed_item]);

    apps_server_handle.abort();
    let _ = apps_server_handle.await;
    Ok(())
}

async fn wait_for_web_search_started(mcp: &mut McpProcess) -> Result<ItemStartedNotification> {
    loop {
        let notification = mcp
            .read_stream_until_notification_message("item/started")
            .await?;
        let started: ItemStartedNotification = serde_json::from_value(
            notification
                .params
                .context("item/started notification should include params")?,
        )?;
        if matches!(&started.item, ThreadItem::WebSearch { .. }) {
            return Ok(started);
        }
    }
}

async fn wait_for_web_search_completed(mcp: &mut McpProcess) -> Result<ItemCompletedNotification> {
    loop {
        let notification = mcp
            .read_stream_until_notification_message("item/completed")
            .await?;
        let completed: ItemCompletedNotification = serde_json::from_value(
            notification
                .params
                .context("item/completed notification should include params")?,
        )?;
        if matches!(&completed.item, ThreadItem::WebSearch { .. }) {
            return Ok(completed);
        }
    }
}

async fn mount_search_response(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/codex/alpha/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "encrypted_output": "ciphertext",
        })))
        .expect(1)
        .mount(server)
        .await;
}

fn has_hosted_web_search(body: &Value) -> bool {
    body.get("tools")
        .and_then(Value::as_array)
        .is_some_and(|tools| {
            tools
                .iter()
                .any(|tool| tool.get("type").and_then(Value::as_str) == Some("web_search"))
        })
}

async fn search_request_body(server: &MockServer) -> Result<Value> {
    server
        .received_requests()
        .await
        .context("failed to fetch received requests")?
        .into_iter()
        .find(|request| request.url.path() == "/api/codex/alpha/search")
        .context("expected standalone search request")?
        .body_json()
        .context("search request body should be JSON")
}

fn create_config_toml(
    codex_home: &Path,
    responses_server_uri: &str,
    apps_server_uri: &str,
) -> std::io::Result<()> {
    std::fs::write(
        codex_home.join("config.toml"),
        format!(
            r#"
model = "mock-model"
approval_policy = "never"
sandbox_mode = "read-only"
model_provider = "openai-custom"
chatgpt_base_url = "{apps_server_uri}"

[features]
standalone_web_search = true

[model_providers.openai-custom]
name = "OpenAI"
base_url = "{responses_server_uri}/api/codex"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0
supports_websockets = false
requires_openai_auth = true
"#
        ),
    )
}

async fn start_apps_server() -> Result<(String, JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let apps_server_url = format!("http://{addr}");

    let mcp_service = StreamableHttpService::new(
        move || Ok(EmptyAppsMcpServer),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );
    let router = Router::new().nest_service("/api/codex/apps", mcp_service);
    let apps_server_handle = tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });

    Ok((apps_server_url, apps_server_handle))
}

#[derive(Clone, Default)]
struct EmptyAppsMcpServer;

impl ServerHandler for EmptyAppsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(rmcp::model::ProtocolVersion::V_2025_06_18)
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        let input_schema: JsonObject = serde_json::from_value(json!({
            "type": "object",
            "additionalProperties": false
        }))
        .map_err(|err| rmcp::ErrorData::internal_error(err.to_string(), None))?;

        let mut tool = Tool::new(
            Cow::Borrowed("noop"),
            Cow::Borrowed("noop"),
            Arc::new(input_schema),
        );
        tool.annotations = Some(ToolAnnotations::new().read_only(true));
        let mut meta = Meta::new();
        meta.0.insert("connector_id".to_string(), json!("noop"));
        meta.0.insert("connector_name".to_string(), json!("No-op"));
        tool.meta = Some(meta);

        Ok(ListToolsResult {
            tools: vec![tool],
            next_cursor: None,
            meta: None,
        })
    }
}
