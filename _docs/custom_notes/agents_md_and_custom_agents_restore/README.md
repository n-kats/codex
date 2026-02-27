# agents_md_and_custom_agents_restore

## 目的

- `--agents-md` と `/custom-agents` を再び有効化し、明示した AGENTS 系ドキュメントをセッションに反映できるようにする。

## 変更内容（何がどう変わるか）

- `codex` CLI の `--agents-md` を interactive / exec の双方で `ConfigOverrides.project_doc_paths` に渡す配線を復元。
- `Config` / `ConfigOverrides` に `project_doc_paths` を復元し、`discover_project_doc_paths()` で明示指定を優先する挙動を復元。
- `Op::OverrideTurnContext` の `project_doc_paths` を core 側で再処理し、`get_user_instructions()` を再計算してセッション設定へ反映。
- TUI `/custom-agents` を「非対応メッセージ」から実処理へ復元。
  - `clear/off/none/auto/default` は auto-discovery に戻す
  - パス指定時は存在/種別チェック後に `OverrideTurnContext` で反映

## 追記（2026-02-27 回帰修正）

- `codex`（interactive）経路で `--agents-md` 引数が `run_interactive_tui` で破棄されていたため、TUI `run_main` まで配線を復元した。
- `codex-rs/tui/src/lib.rs` の `ConfigOverrides` に `project_doc_paths: agents_md` を再設定し、interactive 起動時にも明示 AGENTS パスが反映されるようにした。
- `codex-rs/tui/src/main.rs` は API 変更に合わせて `run_main(..., Vec::new())` を渡すよう更新した。
- 回帰防止として `cli/src/custom_tests.rs` に `custom__agents_md__interactive起動時にtuiへ引き継がれる` を追加し、interactive 起動時の引き継ぎを検証する。

## 対象範囲（非対象も）

- 対象:
  - `codex-rs/cli`
  - `codex-rs/tui`
  - `codex-rs/exec`
  - `codex-rs/core`
- 非対象:
  - AGENTS 自動探索アルゴリズム自体の仕様変更（明示パス優先以外）
  - UI 文言の大規模刷新

## 注意点（環境差・既知の制約）

- この実行環境では `cargo` / `just` が無いため、ローカルでのビルド・テスト実行は未確認。
- `/custom-agents` は指定パスが存在しない、または通常ファイル/シンボリックリンクでない場合にエラー表示する。

## 動作確認手順（手動・テスト・スナップショット）

- 手元環境で以下を実施:
  - `cd codex-rs && cargo test -p codex-tui`
  - `cd codex-rs && cargo test -p codex-core project_doc`
  - `cd codex-rs && cargo test -p codex-cli agents_md_flag_parses`
- 手動確認:
  - `codex --agents-md <path>` 起動時に対象ドキュメントが instructions に反映されること
  - TUI で `/custom-agents <path>` と `/custom-agents clear` が期待どおり動作すること

## つまずきと対処（警告や失敗の修正）

- core 側で `project_doc_paths` が削除されていたため、TUI だけ修正しても有効化されなかった。
- `SessionSettingsUpdate` で `user_instructions` を更新する経路を復元し、`OverrideTurnContext` から再計算結果を適用することで解消。

## 関連ファイル一覧

- `codex-rs/cli/src/main.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/main.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/chatwidget/tests.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/project_doc.rs`
- `codex-rs/core/src/codex.rs`
