# 20260516 rebase custom report

## 実施結果

- 対象ブランチ: `custom`
- rebase 元: `fork-origin/main`
- 比較用退避: `tmp-rebase`
- range-diff ログ: `_tmp/range-diff/20260516-rebase-range-diff.txt`
- 競合解消ログ: `_notes/deconflict/20260516-rebase-custom.md`

## 作業再開メモ

- 2026-05-16: 平常状態に戻したため、この記録を残したうえで `almost` 相当の失敗修正を再開する。

## 考察メモ

- `custom` の修正方針は、上流の新しい責務配置を優先して受け入れ、その上で custom 機能を維持すること。
- `LoaderOverrides.user_config_path` は上流の `AbsolutePathBuf` 前提に寄せ、CLI 側で絶対化して渡すのが自然。
- `ConfigLayerSource::User` は `profile` を持つ前提に合わせ、base は `None`、profile 層は `Some(profile)` にする。
- `ignore_user_config` と `disable_user_config` は役割が違うので混ぜない。
- `unavailable_tool` のような消失 module は、ファイル復活ではなく現行の handler/registry の責務に合わせて処理する。
- custom 専用テストは `custom__...` のまま分離維持し、上流の shape に合わせた実装をテストで守る。

## custom 仕様レポート

- `--codex-home`
  - 実装: `codex-rs/cli/src/custom_tests.rs:23`, `codex-rs/cli/src/custom_tests.rs:76`
  - テスト: `codex-rs/cli/src/custom_tests.rs:23`
  - 検証: `cargo test -p codex-cli custom__codex_home_cli_flag__flag_is_global`

- `--codex-memory`
  - 実装: `codex-rs/cli/src/custom_tests.rs:30`, `codex-rs/cli/src/custom_tests.rs:99`
  - テスト: `codex-rs/cli/src/custom_tests.rs:30`
  - 検証: `cargo test -p codex-cli custom__codex_memory_cli_flag__flag_is_global`

- `--shell-startup-files`
  - 実装: `codex-rs/core/src/shell_startup_files.rs:35`, `codex-rs/core/src/shell_startup_files/custom_tests.rs:38`
  - テスト: `codex-rs/core/src/shell_startup_files/custom_tests.rs:38`
  - 検証: `cargo test -p codex-core shell_startup_files::custom_tests::custom__シェル起動ファイル__cleanはzshのみ隔離する`

- `custom.user_shell.no_inject`
  - 実装: `codex-rs/core/src/config/mod.rs:183`, `codex-rs/core/src/config/mod.rs:1054`, `codex-rs/core/src/tasks/user_shell.rs:136`
  - テスト: `codex-rs/core/tests/suite/custom_user_shell_cmd.rs:75`
  - 検証: `cargo test -p codex-core custom__user_shell_no_inject__bang_result_not_recorded_locally`

- `custom.exec.worker_user`
  - 実装: `codex-rs/core/src/config/mod.rs:185`, `codex-rs/core/src/config/mod.rs:1286`, `codex-rs/core/src/tools/runtimes/apply_patch.rs:353`
  - テスト: `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:96`, `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:234`
  - 検証: `cargo test -p codex-core custom__exec_worker_user__exec_command_tty_false_runs_as_worker_user`

- `config.toml` 読み込み制御
  - 実装: `codex-rs/cli/src/custom_tests.rs:12`, `codex-rs/exec/src/custom_tests.rs:6`
  - テスト: `codex-rs/cli/src/custom_tests.rs:167`, `codex-rs/cli/src/custom_tests.rs:175`, `codex-rs/cli/src/custom_tests.rs:183`
  - 検証: `cargo test -p codex-cli custom__config_toml_read_control__config_toml_file_conflicts_with_no_config`

- `CODEX_ADDITIONAL_PROMPT_DIRS`
  - 実装: `codex-rs/tui/src/custom_prompts.rs:64`, `codex-rs/tui/src/bottom_pane/chat_composer.rs:648`, `codex-rs/tui/src/bottom_pane/command_popup.rs:73`
  - テスト: `codex-rs/tui/src/custom_prompts.rs:234`, `codex-rs/tui/src/custom_prompts.rs:256`
  - 検証: `cargo test -p codex-tui custom__additional_prompt_dirs__`

- `responses` CLI サブコマンド
  - 実装: `codex-rs/cli/src/responses_cmd.rs:10`, `codex-rs/cli/src/main.rs:203`
  - テスト: `codex-rs/cli/src/custom_tests.rs:12`
  - 検証: `cargo test -p codex-cli custom__config_toml_read_control__config_toml_file_flag_is_global`

- app-server remote control API
  - 実装: `codex-rs/app-server/README.md:205`, `codex-rs/app-server/tests/common/mcp_process.rs:573`
  - テスト: `codex-rs/app-server/tests/common/mcp_process.rs:573`
  - 検証: `cargo test -p codex-app-server custom__` 

- custom TUI prompts
  - 実装: `codex-rs/tui/src/custom_prompts.rs:8`, `codex-rs/tui/src/bottom_pane/chat_composer.rs:2736`, `codex-rs/tui/src/bottom_pane/command_popup.rs:175`
  - テスト: `codex-rs/tui/src/custom_prompts.rs:234`
  - 検証: `cargo test -p codex-tui custom__additional_prompt_dirs__`

- custom rebase support docs
  - 実装: `_docs/custom_notes/rebase_rules/README.md:1`, `_docs/custom_notes/rebase_hints/README.md:1`, `_notes/deconflict/README.md:1`
  - テスト: `なし`
  - 検証: `git range-diff "$(git merge-base fork-origin/main tmp-rebase)"..tmp-rebase "$(git merge-base fork-origin/main custom)"..custom`

## 補足

- 今回の rebase は 1 コミットに squash した `custom changes` を `fork-origin/main` に乗せ替える形で完了した。
- 競合は custom 側を基本採用し、`codex-rs/app-server/README.md` だけ protocol に合わせて手動で整えた。
