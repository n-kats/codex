# 20260714-rebase-custom

- File: `codex-rs/app-server/src/request_processors/turn_processor.rs`
  - Line: 709
  - Resolution: 手動マージ
  - Note: 上流の environment selections と runtime workspace roots の流れを優先し、custom の project doc path の相対パス解決・存在確認だけを再適用した。

- File: `codex-rs/connectors/src/connector_runtime/tests.rs`
  - Line: 30
  - Resolution: upstream 優先
  - Note: custom 側の旧 `TestTool` 用フィールドは現行のテスト型に存在しないため削除し、上流の `TestTool` 構造を維持した。

- File: `codex-rs/core/src/config/mod.rs`
  - Line: 2843
  - Resolution: 手動マージ
  - Note: custom の `append_usage_hint_text` と project doc path 解決 helper は独立しているため、両方を保持した。

- File: `codex-rs/core/tests/suite/network_approval.rs`
  - Line: 1351
  - Resolution: custom 維持
  - Note: Guardian の判定 helper と custom の `Network access JSON` メッセージ形式の抽出処理を維持し、trigger と action の両方で同じ形式を扱うようにした。
