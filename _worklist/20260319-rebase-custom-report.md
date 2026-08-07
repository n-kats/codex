# 2026-03-19 rebase report

## 実施結果

- 対象ブランチ: `custom`
- rebase 元: `fork-origin/main`（`01df50cf42`）
- rebase 後: `21da3e1f3b`
- 比較用退避ブランチ: `tmp-rebase`（`071aaa00ab`）
- 退避用バックアップ: `20260319`
- range-diff ログ: `_tmp/range-diff/20260319-rebase-range-diff.txt`

## 確認結果

- `custom` は `fork-origin/main` へ rebase 済み。
- `tmp-rebase` を残したまま比較できる状態を維持。
- 競合は解消済みで、未解決の merge conflict は残っていない。
- range-diff を目視確認し、想定外の差分は見当たらなかった。
- ローカルの `cargo` 実行環境が無いため、MCP の cargo 検証はこの環境では完走できなかった。

## custom 仕様レポート

### `--agents-md` / custom agents 復元

- 実装: `codex-rs/cli/src/main.rs:130`, `codex-rs/tui/src/lib.rs:334`, `codex-rs/core/src/project_doc.rs:186`
- 内容: `agents_md` を CLI から受け取り、TUI / app-server 側へ project doc path として伝搬する。
- 検証: rebase 後の `range-diff` で関連差分を確認。

### `--codex-home` / `--codex-memory`

- 実装: `codex-rs/cli/src/main.rs:85`, `codex-rs/cli/src/main.rs:97`
- 内容: `CODEX_HOME` と `CODEX_MEMORIES_HOME` の切り替えを CLI で上書きできる。
- 検証: `range-diff` でカスタム引数の保持を確認。

### `--config` / `--no-config`

- 実装: `codex-rs/cli/src/main.rs:119`, `codex-rs/core/src/config/mod.rs:2856`
- 内容: ユーザー config の読み込みパスを差し替え、無効化もできる。
- 検証: upstream の config 解決と custom オーバーライドを統合した差分を確認。

### `CODEX_ADDITIONAL_PROMPT_DIRS`

- 実装: `codex-rs/core/src/custom_prompts/custom_tests.rs:11`, `codex-rs/core/src/project_doc/custom_tests.rs:9`
- 内容: 追加プロンプト探索ディレクトリを受け付ける。
- 検証: custom 専用テストの存在と range-diff の整合を確認。

### `CODEX_SHELL_STARTUP_FILES`

- 実装: `codex-rs/core/src/shell_startup_files.rs:34`, `codex-rs/core/src/shell_startup_files/custom_tests.rs:37`
- 内容: clean モードで zsh だけを隔離し、再現性を上げる。
- 検証: custom テストの命名と range-diff を確認。

### `exec_run_as` / worker user

- 実装: `codex-rs/core/src/spawn/run_as.rs:31`, `codex-rs/core/src/tasks/user_shell.rs:175`, `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:29`
- 内容: モデル起因の実行を worker user に寄せる。
- 検証: custom 専用テストが分離されていることを確認。

### `user_shell_no_inject`

- 実装: `codex-rs/core/src/tasks/user_shell.rs:330`, `codex-rs/core/tests/suite/custom_user_shell_cmd.rs:78`
- 内容: `!` の内容と出力をモデルコンテキストへ注入しない。
- 検証: custom 専用テストの存在を確認。

### Enter / Ctrl+Enter / Ctrl+J

- 実装: `codex-rs/tui/src/public_widgets/composer_input.rs:5`, `codex-rs/tui/src/bottom_pane/chat_composer.rs:34`, `codex-rs/tui/src/chatwidget/tests.rs:8612`
- 内容: Enter は改行、Ctrl+Enter / Ctrl+J は送信。
- 検証: 競合解消で upstream の smart approvals 変更と統合されたことを確認。

### `x.y.z-custom-...` 更新チェック

- 実装: `codex-rs/tui/src/updates/custom_tests.rs:7`
- 内容: custom サフィックス付きバージョンでも新旧比較できる。
- 検証: custom 専用テストの存在を確認。

### diff palette override

- 実装: `codex-rs/tui/src/diff_render.rs:325`, `codex-rs/tui/src/diff_render/custom_tests.rs:52`
- 内容: `custom.theme.diff` による色上書きを行う。
- 検証: custom テストで追加/削除色の分岐を保持。

### shell snapshot export redaction

- 実装: `codex-rs/core/src/shell_snapshot.rs:207`, `codex-rs/core/src/shell_snapshot.rs:244`
- 内容: exports を許可リスト化して秘匿情報の大量出力を避ける。
- 検証: 実装箇所を range-diff で確認。

### unified exec deterministic end event

- 実装: `codex-rs/core/src/unified_exec/process_manager.rs:67`, `codex-rs/core/src/unified_exec/process_manager.rs:264`
- 内容: process id の決定性と end event の安定性を維持する。
- 検証: range-diff 上で変更の残存を確認。

### guardian / approvals

- 実装: `codex-rs/core/src/guardian.rs:1`, `codex-rs/core/tests/suite/personality.rs:1`
- 内容: upstream で削除された `guardian` を custom で維持しつつ、smart approvals 周辺の挙動を統合した。
- 検証: `codex-rs/tui/src/chatwidget/tests.rs` の競合解消内容と合わせて確認。

## 補足

- 今回は環境制約により、`make` / `cargo` ベースの再検証は実行していない。
- 実装の整合確認は `git range-diff` と競合解消後のワークツリー確認を基準にしている。
