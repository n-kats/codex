#![allow(clippy::unwrap_used)]

use codex_config::CONFIG_TOML_FILE;
use codex_core::CodexThread;
use codex_core::NewThread;
use codex_features::Feature;
use codex_login::CodexAuth;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::InitialHistory;
use codex_protocol::protocol::WarningEvent;
use codex_utils_absolute_path::AbsolutePathBuf;
use core::time::Duration;
use core_test_support::load_default_config_for_test;
use core_test_support::wait_for_event;
use tempfile::TempDir;
use tokio::time::timeout;
use toml::toml;

const USER_SHELL_NO_INJECT_WARNING: &str = "custom.user_shell.no_inject is false (default); `!` (UserShell) commands and their outputs will be injected into the model context and recorded to the local session history. Set custom.user_shell.no_inject=true to disable injection/recording, and avoid secrets in `!` commands/output.";

async fn wait_for_non_user_shell_warning(
    conversation: &std::sync::Arc<CodexThread>,
) -> WarningEvent {
    loop {
        let warning = wait_for_event(conversation, |ev| matches!(ev, EventMsg::Warning(_))).await;
        let EventMsg::Warning(warning) = warning else {
            unreachable!("wait_for_event should only return warning events here");
        };
        if warning.message == USER_SHELL_NO_INJECT_WARNING {
            continue;
        }
        return warning;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emits_warning_when_unstable_features_enabled_via_config() {
    let home = TempDir::new().expect("tempdir");
    let mut config = load_default_config_for_test(&home).await;
    config
        .features
        .enable(Feature::ApplyPatchStreamingEvents)
        .expect("test config should allow feature update");
    let user_config_path =
        AbsolutePathBuf::from_absolute_path(config.codex_home.join(CONFIG_TOML_FILE))
            .expect("absolute user config path");
    config.config_layer_stack = config.config_layer_stack.with_user_config(
        &user_config_path,
        toml! { features = { apply_patch_streaming_events = true } }.into(),
    );

    let thread_manager = codex_core::test_support::thread_manager_with_models_provider(
        CodexAuth::from_api_key("test"),
        config.model_provider.clone(),
    );
    let auth_manager =
        codex_core::test_support::auth_manager_from_auth(CodexAuth::from_api_key("test"));

    let NewThread {
        thread: conversation,
        ..
    } = thread_manager
        .resume_thread_with_history(
            config.clone(),
            InitialHistory::New,
            auth_manager,
            /*parent_trace*/ None,
            /*supports_openai_form_elicitation*/ false,
        )
        .await
        .expect("spawn conversation");

    let warning = wait_for_non_user_shell_warning(&conversation).await;
    let message = warning.message;
    assert!(message.contains("apply_patch_streaming_events"));
    assert!(message.contains("Under-development features enabled"));
    assert!(message.contains("suppress_unstable_features_warning = true"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn suppresses_warning_when_configured() {
    let home = TempDir::new().expect("tempdir");
    let mut config = load_default_config_for_test(&home).await;
    config
        .features
        .enable(Feature::ApplyPatchStreamingEvents)
        .expect("test config should allow feature update");
    config.suppress_unstable_features_warning = true;
    let user_config_path =
        AbsolutePathBuf::from_absolute_path(config.codex_home.join(CONFIG_TOML_FILE))
            .expect("absolute user config path");
    config.config_layer_stack = config.config_layer_stack.with_user_config(
        &user_config_path,
        toml! { features = { apply_patch_streaming_events = true } }.into(),
    );

    let thread_manager = codex_core::test_support::thread_manager_with_models_provider(
        CodexAuth::from_api_key("test"),
        config.model_provider.clone(),
    );
    let auth_manager =
        codex_core::test_support::auth_manager_from_auth(CodexAuth::from_api_key("test"));

    let NewThread {
        thread: conversation,
        ..
    } = thread_manager
        .resume_thread_with_history(
            config.clone(),
            InitialHistory::New,
            auth_manager,
            /*parent_trace*/ None,
            /*supports_openai_form_elicitation*/ false,
        )
        .await
        .expect("spawn conversation");

    let warning = timeout(
        Duration::from_millis(150),
        wait_for_non_user_shell_warning(&conversation),
    )
    .await;
    assert!(warning.is_err());
}
