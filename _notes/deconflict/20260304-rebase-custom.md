# 20260304-rebase-custom

- File: `codex-rs/core/src/codex.rs`
  - Line: 3748
  - Resolution: 手動マージ（上流追従 + custom 維持）
  - Note: 上流の `should_exit` ベースの dispatch 構造を維持しつつ、`Op::OverrideTurnContext` の `project_doc_paths` と `SessionSettingsUpdate` への伝播を戻した。

- File: `codex-rs/core/src/config_loader/tests.rs`
  - Line: 476
  - Resolution: 上流優先
  - Note: `allowed_web_search_modes` と feature requirements を含む上流の要件テストを採用し、cloud requirements 側の追加フィールドも保持した。

- File: `codex-rs/core/tests/suite/compact_resume_fork.rs`
  - Line: 544
  - Resolution: custom 維持
  - Note: `builder.build(server)` の戻り値から `cwd` を返す 5 要素タプル版を維持し、現行の呼び出し側に合わせた。

- File: `codex-rs/core/tests/suite/personality.rs`
  - Line: 140
  - Resolution: 手動マージ（上流追従 + custom 維持）
  - Note: custom 側の `RemoteModels` 有効/無効制御と remote personality 用の条件を残しつつ、`ManagedFeatures::enable/disable` の `expect(...)` を戻した。

- File: `codex-rs/exec/src/lib.rs`
  - Line: 96
  - Resolution: 手動マージ（上流追従 + custom 維持）
  - Note: 上流の `ThreadEventEnvelope` 形状（`suppress_output` を含む）を保ちつつ、custom の config loader helper と `run_main` シグネチャを維持した。

- File: `codex-rs/otel/src/traces/otel_manager.rs`
  - Line: 95
  - Resolution: custom 維持
  - Note: `apply_traceparent_parent()` と `attach_session_parent()` をそのまま残し、既存の telemetry 設定に追従した。

- File: `codex-rs/app-server/src/codex_message_processor.rs`
  - Line: 274
  - Resolution: 上流追従（rebase 後の残骸整理）
  - Note: 上流 `167158f93` で削除済みの app-server v1 request に対応していた `user_config_toml_path()` / `get_user_saved_config()` / `get_user_info()` / `set_default_model()` が rebase 後も残って warning になっていたため削除した。`config/read` / `config/value/write` / `config/batchWrite` は `message_processor.rs` + `config_api.rs` 側で処理される。`get_user_info` の 1:1 置き換え先は未確認なので、旧クライアント互換が必要なら別途検討する。

- File: `codex-rs/tui/src/app.rs`
  - Line: 4131, 4212, 4292, 4366, 4491, 4573, 4727
  - Resolution: 手動マージ（custom 仕様優先）
  - Note: rebase 後に upstream 由来テスト入力（`Enter` 送信期待）が混在し、custom 仕様（`Enter`=改行, `Ctrl+Enter`/`Ctrl+J`=送信）と不一致で `codex-tui` が失敗したため、送信意図の入力を `Ctrl+Enter` に統一した。

- File: `codex-rs/tui/src/chatwidget/tests.rs`
  - Line: 2867, 2908, 3737, 4080, 4129, 4165, 4207, 9395
  - Resolution: 手動マージ（custom 仕様優先）
  - Note: 同上。`plan popup`/`pending steer`/`review queue` 系の失敗テストで、送信操作を `Enter` から `Ctrl+Enter` に合わせた。実装本体は変更せず、rebase 後のテスト期待のみ整合させた。

- File: `_tmp/almost_test_result.txt`
  - Line: N/A（ログ）
  - Resolution: 検証記録
  - Note: 再実行ログ（更新時刻 `2026-03-04 19:15:12 UTC`）で `FAILED` / `error: test failed` / `make[*]: ***` の出現なしを確認。

## range-diff 目視レビュー（rebase 手順 0/8）

- コマンド:
  - `git -c safe.directory=/workspace -C /workspace range-diff 9b004e2d..tmp-rebase 8a593862..custom`
- 判定:
  - `<`（旧コミット消失）3件は、rebase 前 4 コミットを squash した影響として妥当。
    - `fix enter-to-send behavior in TUI`
    - `enterの挙動を修正`
    - `wip`
  - `>`（新規コミット追加）2件は、rebase 後の追加作業として妥当。
    - `Add 2026-03-04 rebase custom report`
    - `fix`
  - `!`（内容変更）1件は `custom changes -> custom changes`。interdiff の対象を目視し、意図説明可能な差分のみであることを確認。
    - `_notes/deconflict/20260304-rebase-custom.md`: rebase 記録追記
    - `codex-rs/Cargo.lock`: 依存解決差分の追従
    - `codex-rs/core/src/codex.rs`: `project_doc_paths` 伝播を含む手動マージ
    - `codex-rs/core/src/config_loader/tests.rs`: requirements テストの上流追従
    - `codex-rs/core/tests/suite/compact_resume_fork.rs`: 返却値/シグネチャ整合
    - `codex-rs/core/tests/suite/personality.rs`: `ManagedFeatures` API 追従 + custom 条件維持
    - `codex-rs/exec/src/lib.rs`: `run_exec_session` 化等の上流追従 + custom 維持
    - `codex-rs/otel/src/traces/otel_manager.rs`: custom OTEL 親子付け処理維持
    - `codex-rs/tui/src/chatwidget/tests.rs`: custom キー仕様（送信は `Ctrl+Enter`）への整合
