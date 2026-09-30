# 2026-03-04 Rebase Custom Report (fork-origin/main `8a593862`)

- Rebase base: local `fork-origin/main` (`8a593862736e5a3dc701b05838b682299551c36b`)
- Pre-rebase snapshot branch: `20260304`
- Comparison branch: `tmp-rebase`
- Note: network access is restricted in this environment, so this rebase used the locally available `fork-origin/main`.

## Range-diff review

- Reviewed: `git range-diff 9b004e2db126995307bc32735ad3ee20e6fe5ced..tmp-rebase 8a593862736e5a3dc701b05838b682299551c36b..custom`
- Summary: one `!` patch (`custom changes` -> `custom changes`) due conflict resolution while moving the squashed custom patch onto the newer base.
- Main conflict resolution areas:
  - `codex-rs/core/src/codex.rs`
  - `codex-rs/core/src/config_loader/tests.rs`
  - `codex-rs/core/tests/suite/compact_resume_fork.rs`
  - `codex-rs/core/tests/suite/personality.rs`
  - `codex-rs/exec/src/lib.rs`
  - `codex-rs/otel/src/traces/otel_manager.rs`

## Custom spec checklist

- `CODEX_ADDITIONAL_PROMPT_DIRS`
  - 実装: `codex-rs/core/src/custom_prompts.rs:8`, `codex-rs/core/src/custom_prompts.rs:24`
  - テスト: `codex-rs/core/src/custom_prompts/custom_tests.rs:11`
  - 検証: `make verify-additional-prompt-dirs-env`

- Custom diff palette override
  - 実装: `codex-rs/tui/src/diff_render.rs:171`, `codex-rs/tui/src/diff_render.rs:324`
  - テスト: `codex-rs/tui/src/diff_render/custom_tests.rs:52`
  - 検証: `cd codex-rs && cargo test -p codex-tui custom__差分テーマ色__`

- MCP `--no-config` / `--config-toml-file` loader overrides
  - 実装: `codex-rs/cli/src/mcp_cmd.rs:42`, `codex-rs/cli/src/mcp_cmd.rs:172`
  - テスト: なし
  - 検証: `cd codex-rs && cargo test -p codex-cli`

- Worker-user command execution
  - 実装: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs:13`, `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs:639`
  - テスト: `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:29`
  - 検証: `make verify-command-exec-worker-user`

- Shell startup file isolation
  - 実装: `codex-rs/core/src/shell_startup_files.rs:24`, `codex-rs/core/src/shell_startup_files.rs:34`
  - テスト: `codex-rs/core/src/shell_startup_files/custom_tests.rs:12`
  - 検証: `make verify-linux-default-shell`

- Custom agents / project doc override
  - 実装: `codex-rs/tui/src/chatwidget.rs:4172`, `codex-rs/core/src/codex.rs:3759`
  - テスト: `codex-rs/tui/src/chatwidget/custom_tests.rs:27`, `codex-rs/core/src/codex/custom_tests.rs:10`
  - 検証: `cd codex-rs && cargo test custom__custom_agents__`

- Custom test segregation rule
  - 実装: `_docs/custom_notes/custom_tests/README.md:10`, `_docs/custom_notes/custom_tests/README.md:26`
  - テスト: `codex-rs/tui/src/diff_render/custom_tests.rs:52`, `codex-rs/core/src/codex/custom_tests.rs:10`
  - 検証: `cd codex-rs && cargo test custom__`

## Post-rebase follow-up

- App-server v1 legacy handler cleanup
  - 背景: rebase 後、`codex-rs/app-server/src/codex_message_processor.rs` に `user_config_toml_path()` / `get_user_saved_config()` / `get_user_info()` / `set_default_model()` が残っていたが、現行の `ClientRequest` には対応する v1 request variant が無く未使用になっていた。
  - 根拠: 上流 `167158f93` (`chore(app-server): delete v1 RPC methods and notifications (#13375)`) で v1 request の削除方針が入っており、現行の request 入口は `codex-rs/app-server-protocol/src/protocol/common.rs` の v2 `config/*` に移っている。
  - 対応: `codex-rs/app-server/src/codex_message_processor.rs` から未使用の v1 handler と専用 helper / import を削除した。
  - 現在の置き換え先:
    - user saved config 読み取り: `config/read`
    - default model の永続化: `config/value/write` / `config/batchWrite`
    - user info (`alleged_user_email` のみ返す旧 API): 1:1 の v2 置き換え先は未確認
  - 注意: 旧 v1 の `getUserInfo` 相当を外部クライアントが独自利用していた場合は、互換 API の要否を別途確認すること。

## Rebase rerun (after fetch)

- Date: 2026-03-04
- Rebase base: local `fork-origin/main` (`b200a5f45bf6a81b16c7b2e6f04dbe5eb172863c`)
- Pre-rerun snapshot branch: `20260304-rerun-1`
- Archived comparison branch: `tmp-rebase-20260304-rerun-1`
- New comparison branch: `tmp-rebase`
- Reviewed:
  - `git range-diff ae506da078c46fa437b968ec86cb90ece886e367..tmp-rebase b200a5f45bf6a81b16c7b2e6f04dbe5eb172863c..custom`
- Summary:
  - `1: ae506da07 ! 1: 6f04a7ea2 custom changes`
  - Patch差分は app-server-protocol schema JSON の末尾改行差分のみ（`No newline at end of file` 表示の解消）。
  - 機能差分に相当する変更は確認されなかった。
