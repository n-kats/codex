# `--codex-memory`（memories ルートの分離）

## 目的

`sandbox_mode = "workspace-write"` では、起動時に writable root の存在確認が行われます。
従来は memories の置き場が常に `$CODEX_HOME/memories` 固定だったため、`CODEX_HOME` を
書き込み禁止（または存在しない）場所にしている環境だと、`!`（UserShell）などの
コマンド実行が **サンドボックス初期化の時点で失敗**しやすくなっていました。

これを避けるため、memories のファイル成果物だけ別ディレクトリへ分離できる
`--codex-memory` を追加します。

## 変更内容（何がどう変わるか）

- `--codex-memory <PATH>` を指定すると、memories の filesystem 成果物（`raw_memories.md` や
  `rollout_summaries/`、`MEMORY.md`、`memory_summary.md` など）の保存先を
  `<PATH>` に変更します。
- 同等の環境変数 `CODEX_MEMORIES_HOME` でも指定できます。
- `sandbox_mode="workspace-write"` の writable roots に、`<PATH>` が自動で追加されます。
- デフォルトはこれまで通り `$CODEX_HOME/memories` です。

## 対象範囲 / 非対象

対象:

- memories の filesystem 成果物ディレクトリ
- memory consolidation サブエージェントの `cwd` / writable roots

非対象:

- state DB（SQLite）の保存先（これは `sqlite_home` / `CODEX_SQLITE_HOME` / `$CODEX_HOME` のまま）
- memories 機能の有効/無効（`[memories] use_memories` / `generate_memories` とは別）

## 注意点

- `--codex-memory` は「memories だけ分離」なので、`CODEX_HOME` 側へのアクセスを完全に避けたい場合は
  `--codex-home`（または `CODEX_SQLITE_HOME` など）も合わせて分離が必要です。
- 既定では `!` のコマンドと出力はモデルコンテキストに注入され、ローカル履歴にも保存されます。
  秘密情報が混ざる可能性がある場合は `custom.user_shell.no_inject=true` を推奨します。

## 動作確認（手動）

例: `CODEX_HOME` が書き込み禁止でも `id` を試したいケース

1. `codex --codex-memory /tmp/codex_memories_test ...` のように起動
2. `/tmp/codex_memories_test` が作成できること（作れない場合はサンドボックス初期化で失敗し得ます）
3. `! id` を実行してサンドボックス初期化エラー（`.../memories does not exist`）が出ないこと

## Makefile / Docker での既定

- `Makefile` の `run-tui` はデフォルトで `CODEX_MEMORIES_HOME=$$PWD/_cache/codex_memory_debug` を設定して起動します。
- `scripts/docker_run.sh` は `CODEX_MEMORIES_HOME` がセットされていればコンテナへ引き継ぎます（リポジトリ外のパスはコンテナに見えないため警告して無視します）。

## 関連ファイル

- `codex-rs/arg0/src/lib.rs`（`--codex-memory` を早期に `CODEX_MEMORIES_HOME` へ反映）
- `codex-rs/cli/src/main.rs`（ヘルプ/CLI で `--codex-memory` を露出）
- `codex-rs/core/src/config/mod.rs`（`CODEX_MEMORIES_HOME` を解決し、sandbox writable roots へ反映）
- `codex-rs/core/src/memories/phase2.rs`（consolidation サブエージェントの `cwd` / writable roots）
