# 20260324-rebase-custom

- File: `codex-rs/cli/src/debug_sandbox.rs`
  - Line: 15
  - Resolution: custom 維持 + 片側採用
  - Note: Linux 実行経路は `codex_core::landlock::spawn_command_under_linux_sandbox` を復元し、macOS の seatbelt 補助 import は `codex_sandboxing` 側に統一した。

- File: `codex-rs/core/src/config/mod.rs`
  - Line: 866 / 2783
  - Resolution: 手動マージ
  - Note: `load_config_as_toml_with_cli_overrides_and_loader_overrides` を保持しつつ、上流の smart approvals alias migration と `helper_readable_roots` 計算を両立させた。`original_sandbox_policy` も custom 側の比較ロジックとして復元した。

- File: `codex-rs/core/src/exec.rs`
  - Line: 292 / 853
  - Resolution: 手動マージ
  - Note: `ExecParams` / `CommandSpec` の構築で `capture_policy` と `run_as` の両方を保持し、`spawn_child_async_with_run_as` 経路を維持した。

- File: `codex-rs/core/src/memories/phase2.rs`
  - Line: 5
  - Resolution: custom 維持
  - Note: consolidation は `memory_root` を使う既存経路を維持し、重複した `Feature` import は削除した。

- File: `codex-rs/core/src/sandboxing/mod.rs`
  - Line: 50
  - Resolution: 手動マージ
  - Note: `CommandSpec` に `capture_policy` と `run_as` の両方を残し、実行時の capture/権限分離を崩さない形にした。

- File: `codex-rs/core/src/sandboxing/mod_tests.rs`
  - Line: 81 / 135 / 202
  - Resolution: 手動マージ
  - Note: `CommandSpec` 初期化を新しいフィールド構成に合わせ、テストでも `capture_policy` と `run_as: None` を明示した。

- File: `codex-rs/core/src/tools/js_repl/mod.rs`
  - Line: 1041
  - Resolution: 手動マージ
  - Note: JS REPL の `CommandSpec` に `capture_policy` を残しつつ `run_as` を追加した。

- File: `codex-rs/core/src/tools/runtimes/mod.rs`
  - Line: 53
  - Resolution: 手動マージ
  - Note: `build_command_spec` で `capture_policy` と `run_as` の両方を設定し、custom の権限分離を維持した。

- File: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
  - Line: 130 / 917 / 1061
  - Resolution: 手動マージ
  - Note: `ExecRequest` の分解で `capture_policy` を捨てずに `run_as` を取り出し、再実行用 `CommandSpec` にも両方を設定した。

- File: `codex-rs/core/tests/suite/personality.rs`
  - Line: 3
  - Resolution: custom 維持
  - Note: `CollaborationModesConfig` の追加 import を残し、重複した `Feature` import は削除した。

- File: `codex-rs/tui/src/app.rs`
  - Line: 72 / 5272 / 5358 / 5443 / 5522
  - Resolution: custom 維持
  - Note: 入力確定は `Ctrl+Enter` を維持し、`Feature` import の衝突だけ解消した。

- File: `codex-rs/tui/src/lib.rs`
  - Line: 353
  - Resolution: custom 維持
  - Note: `run_main(..., agents_md)` を保持し、project doc 配線を落とさなかった。

- File: `codex-rs/tui_app_server/src/app.rs`
  - Line: 82 / 1369 / 4069 / 4105
  - Resolution: 手動マージ
  - Note: `LoaderOverrides` を復元し、`AppCommand::override_turn_context` の引数列に `project_doc_paths: None` を追加して custom のドキュメント配線を維持した。

- File: `codex-rs/app-server/tests/suite/v2/turn_start.rs`
  - Line: 1829
  - Resolution: 手動マージ
  - Note: spawn 完了時の child state は upstream の `agents_states.len() == 1` も含めた新しい期待値に合わせ、状態の確認メッセージだけ custom に寄せた。

- File: `codex-rs/tui/src/chatwidget.rs`
  - Line: 4398 / 4561
  - Resolution: 手動マージ
  - Note: `/compact` 直後の入力判定を見直し、専用フラグ案は採用せず既存の `task_running` ベースの queue 判定へ戻した。active turn / replay を巻き込む広い条件は避けた。

- File: `codex-rs/tui/src/chatwidget/tests.rs`
  - Line: 1999 / 3872 / 3905 / 4413
  - Resolution: 手動マージ
  - Note: `ChatWidget` の直書き初期化と replay / compact 送信テストを、最終的な queue 判定に合わせて整理した。

- File: `codex-rs/tui_app_server/src/chatwidget.rs`
  - Line: 4550 / 4716
  - Resolution: 手動マージ
  - Note: app-server 側も TUI と同じ queue 判定にそろえ、`/compact` 直後だけを別管理する案は外した。

- File: `codex-rs/tui_app_server/src/chatwidget/tests.rs`
  - Line: 2019 / 3858 / 3891 / 4403
  - Resolution: 手動マージ
  - Note: `ChatWidget` 初期化と replay / compact テストを、最終的な queue 判定に合わせて整理した。
