# tool parallelism テストの安定化

## 目的

`core/tests/suite/tool_parallelism.rs` の並列実行テストが Docker 環境で
スケジューリング遅延の影響を受け、時間閾値によって不安定になる問題を解消する。

## 変更内容

- 並列実行の判定を「所要時間」ではなく「tool 出力」によって確認する。
- `test_sync_tool` の barrier が成立すれば `ok` を返すため、`call-1`/`call-2`
  の tool output を確認することで並列成立を判定する。

## 対象範囲

- `codex-rs/core/tests/suite/tool_parallelism.rs` の
  `read_file_tools_run_in_parallel` テストのみ。

## 非対象

- 並列実行の実装（`ToolCallRuntime` 等）自体の挙動は変更しない。
- 他のテストのタイミング条件は変更しない。

## 注意点

- barrier 成立＝並列成立の前提があるため、tool 出力が `ok` であることを確認する。
- 速度の指標は残らないため、性能劣化検知には別のベンチマークが必要。

## 動作確認手順

- `make test-core`
- もしくは `cargo test -p codex-core --test all`

## つまずきと対処

- Docker では `read_file_tools_run_in_parallel` が 900ms を超えることがあり、
  時間閾値で false negative になり得るため、判定方式を見直した。

## 関連ファイル

- `codex-rs/core/tests/suite/tool_parallelism.rs`
