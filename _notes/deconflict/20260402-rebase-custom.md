# 20260402-rebase-custom

- File: `codex-rs/tui_app_server/src/app.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: 上流で `tui_app_server` クレートが削除されていたが、custom 側の旧 UI 実装をそのまま残す方針にした。以後の検証で不要なら別途整理する。

- File: `codex-rs/tui_app_server/src/chatwidget.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: 同様に旧パスの chatwidget 実装と snapshot 群を custom 側で復元した。新しい `tui/` 側の実装とは衝突しないため、rebase 通過を優先した。

- File: `codex-rs/core/src/custom_prompts.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: 上流側で削除されていた旧ファイルを custom 側で復元した。関連する `custom_prompts/custom_tests.rs` も合わせて保持した。

- File: `codex-rs/core/src/sandboxing/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: exec / sandbox 系の custom 分岐を優先し、rebase 後も worker 実行・capture 周りの既存挙動を維持するようにした。

- File: `codex-rs/tui/src/chatwidget.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: TUI の入力・queue 判定の custom 差分を優先し、上流の同名変更は次段のテストで再確認する前提で解消した。
