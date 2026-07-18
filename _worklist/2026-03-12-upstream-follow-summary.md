# 2026-03-12 上流追従（`fork-origin/main` → `custom`）説明

このドキュメントは、`custom` ブランチが `fork-origin/main`（上流）へ追従するために行った変更の意図・差分理由・検証ログへの導線をまとめたもの。

タスクリストは `_worklist/2026-03-12-upstream-follow-tasks.md` を参照。

## 対象（固定したベース）

- 上流 tip（ローカルに存在する `fork-origin/main`）: `cec211cabc158532459b0c522a0cf855a891bd40`（2026-03-10, `render local file links from target paths (#13857)`）
- `custom` tip: `0bee91fd3dd7addd53f6c34b577eaa50035ac127`（2026-03-12, `codex-rs/core/src/spawn.rsの最適化`）
- `custom` の追従作業ログ（range-diff）:
  - `_tmp/range-diff/20260312-rebase-range-diff.fork-origin-main-cec211cab.custom-0bee91fd3.txt`
  - 直前のログ: `_tmp/range-diff/20260311-rebase-range-diff.fork-origin-main-cec211cab.custom-7820b8470.txt`
- rebase 時のコンフリクト解消ログ: `_notes/deconflict/`（例: `_notes/deconflict/20260311-rebase-custom.md`）

## ねらい（今回の追従方針）

主目的は「上流追従の再発コスト（range-diff のノイズと rebase コンフリクト）を下げる」こと。

特に差分が肥大化しやすい箇所を **上流の骨格へ寄せつつ、fork 要件は局所に閉じる** 方針で整理した。

## 何を変えたか（重要度順）

## 作業内容（コミットのまとまり）

`fork-origin/main` から `custom` までの主要コミット（要点のみ）:

- `b8d13ff8d`: `codex-rs/core/src/unified_exec/process_manager.rs` の差分縮小（fork 固有を隔離）
- `47893b9f7`: `codex-rs/core/src/config/mod.rs` の差分縮小（fork 固有を `custom.rs` 側へ寄せる）
- `cfaabe292`: `codex-rs/exec/src/lib.rs` の本家追従（app-server client ルートへ寄せる）
- `0bee91fd3`: `codex-rs/core/src/spawn.rs` の最適化（`run_as` 分割を含む）

### 1) `codex-rs/exec` を app-server client ルートへ寄せる

- 対象: `codex-rs/exec/src/lib.rs`（+呼び出し側）
- 方針:
  - 上流の `codex-app-server-client`（in-process app-server）経由のイベントループへ移行し、core 直結の構造差を縮める。
  - 既存 fork の event processor（human / JSONL）を接続し、stdout 制約（stdout は最終メッセージのみ）を維持する。
- 影響（意図した差分）:
  - `ClientRequest::*` / `InProcessServerEvent::*` へ寄せた request / event の受け渡し
  - `run_main()` シグネチャを upstream 互換へ戻し、fork 専用の導線として `run_main_with_agents_md()` を追加
  - 呼び出し側の追従（例: `codex-rs/cli/src/main.rs`）

### 2) `codex-rs/core/src/config/mod.rs` の fork 差分を局所化

- 対象: `codex-rs/core/src/config/mod.rs`, `codex-rs/core/src/config/custom.rs`, `codex-rs/core/config.schema.json`
- 方針:
  - fork 独自キー／解決ロジックは `codex-rs/core/src/config/custom.rs` 側へ寄せ、`mod.rs` は upstream の構造（型/導線/テスト取り回し）へ近づける。
  - `deny_unknown_fields` 前提で、未知キーはロード時にエラー（互換レイヤは追加しない）。
- 生成物:
  - `codex-rs/core/config.schema.json` を更新（`just write-config-schema` 相当の更新が含まれる前提）

### 3) unified exec の fork 要件（`run_as` / sudo / argv0 / PTY）を拡張点へ分離

- 対象: `codex-rs/core/src/unified_exec/process_manager.rs`, `codex-rs/core/src/unified_exec/custom.rs`
- 方針:
  - 上流の `process_manager` の骨格へ寄せ、fork 固有は `custom.rs` 側へ隔離してコンフリクトを減らす。
  - `run_as` が必要な場合のみ custom 経路へ分岐する。

### 4) spawn の `run_as` を分割し、上流の `spawn_child_async()` 骨格に寄せる足場を作る

- 対象: `codex-rs/core/src/spawn.rs`, `codex-rs/core/src/spawn/run_as.rs`
- 方針:
  - `run_as` / sudo などの fork 要件は `run_as.rs` 側へ寄せる。
  - `spawn_child_async()` は upstream 同形を維持し、`run_as.is_some()` の場合のみ別経路へ分岐する。
- 注意:
  - `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` / `CODEX_SANDBOX_ENV_VAR` 周辺は不用意にいじらない（差分が必要なら理由を明示）。

## “維持すべきカスタム要件” の扱い

fork の要件は `_docs/custom_notes/` に集約し、追従作業で破壊しないためのガードレールにしている。

例（代表）:

- `--codex-home`（起動直後反映）: `_docs/custom_notes/codex_home_cli_flag/README.md`
- worker user 固定（`custom.exec.*`）: `_docs/custom_notes/command_exec_worker_user/README.md`
- `exec_command` の login / dotfiles 制御（`CODEX_SHELL_STARTUP_FILES`）: `_docs/custom_notes/exec_command_default_login/README.md`
- `!` とモデル起動コマンドの env policy 分離: `_docs/custom_notes/user_shell_environment_policy_split/README.md`
- unified_exec の `ExecCommandEnd` 確実化: `_docs/custom_notes/unified_exec_end_event_deterministic/README.md`

## 検証（ログ）

- `make all` が PASS（2026-03-12）
  - ログ: `_tmp/all_test_result.txt`
- 追加の動作確認: 手元で実施済み（実施者が別途チェック項目を持つ前提）

## 差分理由（主要ファイル）

差分が大きい／追従コストに効くファイルは、基本的に以下のどれか:

1) fork 要件（上流に寄せると要件が壊れるため差分維持が必要）
2) 追従コスト低減（上流の骨格へ寄せるための整理。最終的に差分が縮む方向）
3) ドキュメント／作業ログ（追従と保守のための導線）

代表例:

- `codex-rs/exec/src/lib.rs`: 追従コスト低減（app-server client ルートへ移行、stdout/JSONL 制約維持）
- `codex-rs/core/src/config/mod.rs`: 追従コスト低減（fork の設定差分を `custom.rs` へ局所化）
- `codex-rs/core/src/unified_exec/custom.rs`: fork 要件（run_as / sudo / argv0 / PTY）
- `codex-rs/core/src/spawn/run_as.rs`: fork 要件（run_as / sudo）
- `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs`: fork 要件の回帰防止（worker user）
- `Makefile` / `_docs/custom_notes/*`: ドキュメント／検証導線（fork 要件の説明と再現性）
- `codex-rs/app-server-protocol/schema/json/*` / `codex-rs/core/config.schema.json`: 生成物（追従による schema 反映）

## 次回の追従で見るべきポイント

- `codex-rs/core/src/spawn.rs` は `CODEX_SANDBOX_*` 周辺の制約が強いので、差分縮小は段階的に進める（骨格は upstream 同形に寄せるが、環境変数周りの不用意な並べ替えは避ける）。
- “fork 要件” は `_docs/custom_notes/` とテストで守る（追従時の目視点検を減らす）。
