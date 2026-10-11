# 20260313-rebase-custom

- File: `codex-rs/tui/src/multi_agents.rs`
  - Line: 80
  - Resolution: 手動マージ
  - Note: `agent_picker_status_dot_spans`（custom 側）を維持しつつ、上流の agent fast-switch（Alt+←/→ + macOS fallback）用の shortcut helpers を取り込んだ。重複していた関数定義は整理した。

- File: `codex-rs/core/tests/suite/personality.rs`
  - Line: 68
  - Resolution: 上流優先
  - Note: 「初回 turn まで context 挿入を遅延」変更に合わせ、`assert_personality_applied` ではなく developer 入力に personality update が含まれることを検証する形に更新した。

- File: `codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_compact_resume_restates_realtime_end_shapes.snap`
  - Line: 3
  - Resolution: 手動マージ
  - Note: `assertion_line` の競合のみだったため、実際の `core/tests/suite/compact_remote.rs` 側の `assert_snapshot!` 行番号に合わせて更新した。

- File: `codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_restates_realtime_end_shapes.snap`
  - Line: 3
  - Resolution: 手動マージ
  - Note: `assertion_line` の競合のみだったため、実際の `core/tests/suite/compact_remote.rs` 側の `assert_snapshot!` 行番号に合わせて更新した。

- File: `codex-rs/core/src/state/session.rs`
  - Line: 206
  - Resolution: 手動マージ
  - Note: 上流で追加された permissions / session start source の取り回しを取り込みつつ、`set_pending_session_start_source` / `take_pending_session_start_source` が重複定義にならないよう、既存の実装に寄せて整理した。

- File: `codex-rs/app-server/src/codex_message_processor.rs`
  - Line: 5883
  - Resolution: 手動マージ
  - Note: tracing 伝播のため `thread.submit(...)` ではなく `submit_core_op(...)` を使う上流変更を採用しつつ、`Op::OverrideTurnContext` の `project_doc_paths: None` は欠けないように補った。

- File: `codex-rs/cli/src/debug_sandbox.rs`
  - Line: 257
  - Resolution: 上流優先
  - Note: Linux sandbox の既定を bubblewrap に寄せる上流変更に合わせ、`features.use_legacy_landlock()` を使って helper 呼び出しを行うよう統一した。

- File: `codex-rs/core/src/sandboxing/mod.rs`
  - Line: 664
  - Resolution: 手動マージ
  - Note: `create_linux_sandbox_command_args_for_policies` 呼び出しの引数競合を解消し、`use_legacy_landlock` + `allow_network_for_proxy(enforce_managed_network)` を渡す形に統一した。

- File: `codex-rs/linux-sandbox/tests/suite/landlock.rs`
  - Line: 147
  - Resolution: 上流優先
  - Note: bubblewrap 既定化に伴うテスト更新（/dev 周りの bwrap 専用テスト追加など）を取り込み、関連する `run_cmd_result_with_writable_roots(..., use_legacy_landlock, network_access)` の引数は bwrap 経路（`false`）で動くように揃えた。

