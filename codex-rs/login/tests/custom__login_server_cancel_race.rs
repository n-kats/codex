#![allow(non_snake_case)]
use std::io;
use std::time::Duration;

use anyhow::Result;
use codex_core::auth::AuthCredentialsStoreMode;
use codex_login::ServerOptions;
use codex_login::run_login_server;
use tempfile::tempdir;

#[tokio::test]
async fn custom__ログインサーバ__起動直後にcancelしてもハングしない() -> Result<()> {
    let tmp = tempdir()?;
    let codex_home = tmp.path().to_path_buf();

    let opts = ServerOptions {
        codex_home,
        cli_auth_credentials_store_mode: AuthCredentialsStoreMode::File,
        client_id: codex_login::CLIENT_ID.to_string(),
        issuer: "http://example.com".to_string(),
        port: 0,
        open_browser: false,
        force_state: Some("immediate_cancel".to_string()),
        forced_chatgpt_workspace_id: None,
    };

    let server = run_login_server(opts)?;
    server.cancel();

    let result = tokio::time::timeout(Duration::from_secs(2), server.block_until_done()).await;
    assert!(
        result.is_ok(),
        "login server should exit promptly when cancelled immediately"
    );
    let err = result
        .expect("timeout future resolved")
        .expect_err("expected login server to report cancellation");
    assert_eq!(err.kind(), io::ErrorKind::Other);
    Ok(())
}
