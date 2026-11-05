# 20260907-rebase-custom.md

- File: `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - Line: generated binary fixture
  - Resolution: 手動マージ
  - Note: upstream の user-verification API と生成済み schema を維持し、custom の `ThreadSettingsUpdateParams.projectDocPaths` を再適用した。生成後の protocol fixture テストで TS／JSON の一致を確認した。

- File: `codex-rs/cli/src/main.rs`
  - Line: subcommand enum and dispatch
  - Resolution: upstream 優先
  - Note: upstream の通常 CLI、remote、managed-worktree の処理を維持し、upstream が削除した deprecated `codex mcp-server` サブコマンドは復元しなかった。

- File: `codex-rs/exec/src/lib.rs`
  - Line: CLI destructuring and managed-worktree/cloud configuration setup
  - Resolution: 手動マージ
  - Note: upstream の managed-worktree source validation を維持し、custom の PSP と cloud feature 無効時の config bundle fallback を両立した。

- File: `codex-rs/mcp-server/`、workspace Cargo manifests、`codex-rs/Cargo.lock`
  - Line: upstream deletion versus custom implementation
  - Resolution: upstream 優先
  - Note: custom 側に残っていた旧 MCP server crate は独立した custom 方針として列挙されておらず、upstream の削除に合わせて command、crate、tests、workspace 依存、lockfile の参照を削除した。

- File: `codex-rs/tui/src/startup_orchestration.rs`
  - Line: startup cloud bundle and managed-worktree initialization
  - Resolution: 手動マージ
  - Note: upstream の managed-worktree startup と source validation を維持し、custom の feature-gated cloud bundle loader を接続した。

- File: `codex-rs/tui/src/app/tests/startup_defaults_tests.rs`, `codex-rs/tui/src/app/tests.rs`
  - Line: `SessionTarget` fixtures
  - Resolution: upstream 追従
  - Note: upstream が追加した `SessionTarget.cwd` を既存の test fixture に `None` として補った。startup／worktree テストで確認した。

## 追加監査で削除した過去維持

- `codex-rs/core-skills/src/loader_tests.rs` と `codex-rs/config/src/mcp_edit_tests.rs` は、現行 workspace から参照されず、upstream にも対応する crate／module がない孤立テストだったため削除した。
- `codex-rs/deny.toml` の `codex-app-server`、`codex-app-server-daemon`、`codex-backend-client`、`codex-core`、`codex-ollama` 向け reqwest 移行例外は、現行の直接依存に対応しない古い例外だったため削除した。
- `codex-rs/core/src/util.rs` の `feedback_tags!` ドキュメント例と `codex-rs/core/src/spawn.rs` の空行だけの差分は、custom 要件のない差分だったため upstream に戻した。
- `codex-rs/features/src/lib.rs` の `ContentItemKinds` を `UnderDevelopment`／無効化していた古い custom override は、現行 `CUSTOM.md` に要件がなく、upstream が Stable／既定有効にしたため upstream 状態へ戻した。
- `codex-rs/codex-mcp/src/elicitation.rs` に残っていた削除済み `codex mcp-server` へのコメント参照は、挙動を変えず一般的な表現へ直した。
- `.github/workflows/issue-labeler.yml` の `mcp-server` ラベル説明も、削除済みコマンドではなく現行の MCP server 実装／設定を指す説明へ更新した。

## 監査で維持したもの

- Cloud Tasks のソース／コメントは、`CUSTOM.md` と `remove_cloud_tasks_command` の方針どおり CLI からは除去しつつ rebase 用参照として保持する対象なので削除していない。
- exec-server の zombie 判定、ExecutableFileBusy retry、HTTP/TLS の防御的処理、custom shell／theme／PSP などは、現行 custom ノートまたは上流不具合修正として根拠があり、過去維持の残骸とは判定していない。

## 検証

- MCP cargo fmt check: 成功（監査後にも再確認。stable rustfmt の `imports_granularity = Item` 警告のみ）
- MCP selected tests: app-server protocol、CLI、exec、TUI startup 158 tests、TUI worktree 14 tests: 成功
- MCP `codex-features` tests: 41 tests 成功（`ContentItemKinds` の Stable／既定有効化後を含む）。
- 削除対象の孤立テスト／旧 MCP command／旧 deny wrapper／`ContentItemKinds` override の参照なしを確認した。
- MCP `make almost` 相当: fmt と Linux sandbox build は成功。workspace test は `glib-sys` が要求する system library `glib-2.0.pc` が環境にないため開始時に終了した。

内容上の競合マーカーは除去済み。ただし `.git` が read-only mount のため、`git add`／rebase continue による index 更新は実行できず、Git からは6パスが未ステージ競合として残る。
