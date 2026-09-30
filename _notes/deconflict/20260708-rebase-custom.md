# 20260708-rebase-custom.md

- File: `codex-rs/core/src/lib.rs`
  - Line: 43
  - Resolution: 手動マージ
  - Note: 上流の `elicitation` モジュールと custom 側の `custom` モジュールを両方残した。

- File: `codex-rs/code-mode-host/tests/stdio.rs`
  - Line: 685
  - Resolution: custom 維持
  - Note: host proxy ディレクトリ生成は `TempDir::new()` を採用し、custom 側の変更を残した。
