# 20260813 rebase custom report

今回の上流 rebase では、custom 側で維持すべき実行経路を残しつつ、上流で廃止された config lockfile schema は復元しなかった。

- PSP routing
  - 実装: `codex-rs/cli/src/main.rs:110-110,1002-1007`, `codex-rs/exec/src/cli.rs:17`, `codex-rs/tui/src/cli.rs:13`, `codex-rs/core/src/config/mod.rs:995,1640-1641,4239`
  - テスト: なし（既存 upstream の feature/config 経路で検証）
  - 検証: MCP `run_cargo_check`、MCP almost equivalent

- CLI の Codex home / memory / config-file / AGENTS.md override
  - 実装: `codex-rs/cli/src/main.rs:1997-2005,2083`, `codex-rs/exec/src/lib.rs:247-307,478-485`, `codex-rs/tui/src/lib.rs:1101-1102`
  - テスト: `codex-rs/cli/src/custom_tests.rs:10-82`, `codex-rs/exec/src/custom_tests.rs:10-42`
  - 検証: MCP almost equivalent

- Thread settings の project document paths
  - 実装: `codex-rs/app-server-protocol/src/protocol/v2/thread.rs:269-273`, `codex-rs/core/src/config/mod.rs:886,2597,3423`
  - テスト: `codex-rs/tui/src/chatwidget/tests/slash_commands.rs:2742-2778`, app-server precomputed/schema fixtures
  - 検証: MCP selected `experimental_precomputed_exports_match_generated`、MCP selected `schema_fixtures_tests::experimental_precomputed_exports_match_generated`、MCP almost equivalent

- Remote code-mode spawn hardening
  - 実装: `codex-rs/code-mode/src/remote_session/connection.rs:227-230`
  - テスト: 既存 code-mode integration tests
  - 検証: MCP `run_cargo_check`、MCP almost equivalent

- Custom config schema / user-shell settings
  - 実装: `codex-rs/core/config.schema.json`, `codex-rs/config/src/custom/`, `codex-rs/core/src/config/custom/`
  - テスト: `codex-rs/core/src/config/schema_tests.rs:14-56`
  - 検証: MCP selected `config::schema::tests::config_schema_matches_fixture`、MCP almost equivalent

- 全体実行時だけ揺れたテスト
  - 実装: `flaky_test_list.txt`
  - テスト: `suite::v2::remote_control::stdio_eof_exits_with_remote_control_connection`、`suite::model_switching::rollback_first_turn_model_change_removes_its_instructions::retry_switched_model`
  - 検証: それぞれMCP個別実行は成功、除外後のMCP almost equivalent は成功

注記: ユーザー指定どおり `git add` は実行していないため、Git の index は未解消 (`UU`) のまま。作業ツリー上の競合マーカーは除去済みで、rebase の continue は行っていない。
