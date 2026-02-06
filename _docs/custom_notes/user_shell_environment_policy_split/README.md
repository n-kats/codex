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

- `cd codex-rs && cargo test -p codex-core --lib config::custom_user_shell_environment_policy_overrides_user_shell_env`

## 関連ファイル

- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/codex.rs`
- `codex-rs/core/src/tasks/user_shell.rs`

