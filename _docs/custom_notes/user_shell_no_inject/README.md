# user_shell_no_inject

## 目的

`!`（UserShell）で実行したコマンド内容や出力が、モデルのコンテキストに混入したり、ローカルのセッション履歴に残ることを防ぐ。

## 変更内容

`config.toml` に `custom.user_shell.no_inject` を追加する。

- `custom.user_shell.no_inject = true`
  - `!`（UserShell）のコマンド内容/出力を **モデルコンテキストへ inject しない**
  - `!`（UserShell）のコマンド内容/出力を **ローカルのセッション履歴へ保存しない**
- `custom.user_shell.no_inject = false`
  - 従来どおり inject/保存する
  - 起動時に注意のワーニングを出す（未設定で既定 `false` の場合も含む）

設定例:

```toml
[custom.user_shell]
no_inject = true
```

## 対象範囲

- 対象: `!`（UserShell）で実行したコマンドの記録（モデルコンテキストへの inject とローカルの会話履歴保存）
- 非対象:
  - `!` の実行結果が UI に表示されること（`ExecCommand*` イベント）は止めない
  - 端末や OS が持つ履歴（shell history 等）は別問題

## 注意点

- `no_inject = true` でも、実行中の表示（stdout/stderr）が完全に秘匿されるわけではない。
- 既定値は `false`（互換性維持）。意図せず秘密を含めないよう、秘密が絡む場合は `no_inject = true` を推奨。
- 起動時のワーニングは、`custom.user_shell.no_inject = false`（未設定で既定 `false` を含む）の場合に表示される。

## 動作確認手順

- 手動:
  1. `~/.codex/config.toml`（または `--config` 指定のファイル）に `[custom.user_shell] no_inject = true` を設定
  2. Codex を起動し、`! echo hello` を実行
  3. スレッドの履歴（保存されたセッション）を確認し、`<user_shell_command>` が保存されていないことを確認
- テスト:
  - `codex-rs/core/src/config/custom_tests.rs` の `custom__user_shell_no_inject__*`
  - `codex-rs/core/tests/suite/custom_user_shell_cmd.rs` の `custom__user_shell_no_inject__*`

## つまずきと対処

- `no_inject` 変更後に `config.schema.json` の更新が必要になる。
- 既存のテストが `Config` の struct literal を持つ場合、新フィールド追加でコンパイルエラーになりうる。

## 現在の実装メモ

- `core/src/config/mod.rs` の `Config::user_shell_no_inject()` で `custom.user_shell.no_inject` を解決し、`Config` に保持した `custom` を参照している。
- `core/src/tasks/user_shell.rs` の `persist_user_shell_output()` が `turn_context.user_shell_no_inject()` を最初に見て、`true` の場合はモデル注入とローカル履歴保存を止める。
- `ExecCommandBegin` / `ExecCommandEnd` は通常どおり流し、`no_inject` は保存だけを切り替える。
- そのため、`!` の表示は維持しつつ、履歴混入だけを抑える方針を保っている。
- 起動時 warning は `custom.user_shell.no_inject = false`（未設定の既定 `false` を含む）で出す。
- 回帰テストとして、`core/src/config/config_tests.rs` に warning 解決テストを追加し、`core/tests/suite/custom_user_shell_cmd.rs` に `no_inject` の履歴非保存テストを追加した。
- `custom.exec.worker_user` と組み合わせたときも、`!` の挙動は invoker 側で維持されることを `core/tests/suite/user_shell_cmd.rs` で確認している。

## 関連ファイル一覧

- `codex-rs/core/src/config/custom.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/tasks/user_shell.rs`
- `codex-rs/core/src/user_shell_command.rs`
- `codex-rs/core/config.schema.json`
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/core/tests/suite/custom_tests.rs`
