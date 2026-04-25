# agents_md_and_custom_agents_restore

## 目的

- `--agents-md` と `/custom-agents` を再び有効化し、明示した AGENTS 系ドキュメントをセッションに反映できるようにする。

## 変更内容（何がどう変わるか）

- `codex` CLI の `--agents-md` を interactive / exec の双方で `ConfigOverrides.project_doc_paths` に渡す配線を復元。
- `Config` / `ConfigOverrides` に `project_doc_paths` を復元し、`discover_project_doc_paths()` で明示指定を優先する挙動を復元。
- `Op::OverrideTurnContext` の `project_doc_paths` を core 側で再処理し、`get_user_instructions()` を再計算してセッション設定へ反映。
- TUI の `/custom-agents` から app-server の `thread/metadata/update` を経由して、loaded thread の live context も更新する。
- TUI `/custom-agents` を「非対応メッセージ」から実処理へ復元。
  - `clear/off/none/auto/default` は auto-discovery に戻す
  - パス指定時は存在/種別チェック後に `OverrideTurnContext` で反映
- session 側で `project_doc_paths` を更新したとき、`user_instructions` を再生成して次ターンへ反映する。
- `project_doc_paths` に存在しないパスやディレクトリが含まれる場合は `BadRequest` として拒否し、既存の設定を維持する。
- `project_doc_paths` を更新したあと、再計算用の config 側でも `cwd` を現在の session cwd に揃えてから `user_instructions` を再生成する。
- `project_doc_paths` を更新したときは reference context をクリアし、次ターンで新しい project docs を full context として再注入できるようにする。

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
- `project_doc_paths` の再計算で `user_instructions(None)` を使うと自動探索が落ちるケースがあったため、local filesystem を使うように修正した。
- `/custom-agents clear` は TUI では空の `project_doc_paths` を送るだけなので、core 側では `project_doc_paths` の空化を確認する形で回帰を抑えた。
- `project_doc_paths` 更新時に再計算対象の config が古い cwd を保持していると、相対パスが解決できず `user_instructions` が `None` になったため、`cwd` の同期を追加した。
- `project_doc_paths` 更新で baseline を残すと次ターンの full context 再注入が止まるため、reference context を明示的にクリアするようにした。
- `/custom-agents` の適用結果は、TUI の history に `custom-agents applied: ...` として出し、実際に読み込まれた instruction source を cwd 基準で解決したパスで見せる。backend の warning `custom-agents loaded ...` は重複表示しない。
- `/custom-agents` の内容が次 turn の model-visible context に入るかは、`thread/metadata/update` を通した app-server E2E で検証する。具体的には、AGENTS.md の内容が最初の turn で見え、`/custom-agents` 後の次 turn では置き換わることを確認する。
- startup warnings は bootstrap の `config_warnings` へ二重に流さず、session 側の通知で一度だけ見せるようにした。

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
- `codex-rs/core/src/session/mod.rs`
- `codex-rs/core/src/session/session.rs`
- `codex-rs/core/src/session/tests.rs`
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
- `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`
