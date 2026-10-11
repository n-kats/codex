# 20260306-rebase-custom

- File: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
  - Line: 13
  - Resolution: 上流優先
  - Note: `compile_permission_profile` と `RunAsUser` の import を採用し、上流の `exec_run_as` 対応を維持した。custom 側では追加差分を持たず、競合マーカーのみ除去した。
