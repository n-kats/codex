# `!`（UserShell）とモデル起動コマンドの環境変数ポリシー分離

## 目的

- `custom.exec.worker_user` を使うと、モデル起動コマンドは別ユーザー（例: `assistant`）で動く。
- その際に `HOME` などの環境変数を `assistant` 向けに調整したいが、既存の `shell_environment_policy` は `!`（ユーザー起点のコマンド）にも共有されていたため、`!echo $HOME` のような挙動まで変わってしまう。
- そこで、`!` とモデル起動コマンドで別々の `ShellEnvironmentPolicy` を指定できるようにする。

## 変更内容

- 新しい設定を追加:
  - `custom.user_shell_environment_policy`: `!`（UserShell）用
  - `custom.assistant_shell_environment_policy`: モデル起動のコマンド実行用（`shell` / `shell_command` / `exec_command` など）
- 解決ルール:
  - モデル起動側: `custom.assistant_shell_environment_policy` があればそれを使用し、なければ従来どおり `shell_environment_policy` を使用する。
  - `!` 側: `custom.user_shell_environment_policy` があればそれを使用し、なければモデル起動側（上で解決した policy）を使用する。

## 現在の実装メモ

- `core/src/config/mod.rs` に `assistant_shell_environment_policy()` / `user_shell_environment_policy()` を追加し、`Config` に保持した `custom` を元に解決している。
- `core/src/session/turn_context.rs` では turn-context 経由で両 policy を参照できるようにしている。
- `core/src/tools/handlers/shell.rs`、`core/src/tools/runtimes/shell.rs`、`core/src/tools/runtimes/unified_exec.rs`、`core/src/tools/js_repl/mod.rs`、`core/src/unified_exec/process_manager.rs` では assistant policy を使って env を構築している。
- `core/src/tasks/user_shell.rs` は user policy を使って `!` の env を構築している。
- `custom.exec.worker_user` の warning テストと一緒に、`custom.user_shell_environment_policy_overrides_user_shell_env` の既存テストで split が壊れていないことを確認している。
- 追加で `core/tests/suite/shell_command.rs` と `core/tests/suite/user_shell_cmd.rs` に runtime の E2E を置き、`shell_command` が assistant policy を、`!` が user policy を使うことを確認している。

## 対象範囲 / 非対象

- 対象: `!`（UserShell）とモデル起動コマンドの env 構築。
- 非対象: sandbox の権限制御・ユーザー分離（`custom.exec.*`）自体の挙動。

## 設定例

モデル起動側は `assistant` の `HOME` にしつつ、`!` の `HOME` は `ubuntu` のままにする例:

```toml
[custom.exec]
worker_user = "assistant"

[shell_environment_policy]
inherit = "core"
set = { HOME = "/home/assistant" }

[custom.user_shell_environment_policy]
inherit = "core"
set = { HOME = "/home/ubuntu" }
```

## 動作確認

- `MCP` 経由で `codex-core` の `custom_user_shell_environment_policy_overrides_user_shell_env` を実行して確認
- `MCP` 経由で `codex-core` の `custom_user_shell_no_inject_is_resolved` / `custom_exec_accepts_uid_gid_pair` / `custom_exec_requires_uid_and_gid_together` を併せて確認

## 関連ファイル

- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/codex.rs`
- `codex-rs/core/src/tasks/user_shell.rs`
