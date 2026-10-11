# 20260711-rebase-custom

- File: `codex-rs/exec-server/src/client.rs`
  - Line: 786
  - Resolution: 上流優先
  - Note: 上流の `in_current_span().with_current_subscriber()` を採用し、process-start のバックグラウンドタスクへ呼び出し元の tracing span と subscriber を伝播させた。custom 側の `info_span!("process-start")` は呼び出し元の trace context を置き換えるため採用しなかった。

- File: `codex-rs/exec-server/src/client.rs`
  - Line: 1848
  - Resolution: 手動修正（テスト安定化）
  - Note: Unix のプロセス終了判定で `kill -0` が zombie プロセスを実行中と誤判定していたため、`ps` の状態を確認し `Z` 状態を終了済みとして扱った。MCP の個別テストで確認した。
