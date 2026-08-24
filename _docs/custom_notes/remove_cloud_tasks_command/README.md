# `codex cloud` コマンドの機能除去

## 目的

ローカル CUI 利用では Codex Cloud Tasks を使わないため、Cloud Tasks の CLI/TUI と関連 crate を通常の CLI ビルド・workspace テストから外す。Cloud Tasks が `codex-core` を経由して code-mode/V8 までビルドへ持ち込むことを避け、ローカル利用時のビルド負荷を減らす。

## 変更内容

実装時は次の変更を行う。

- `codex-rs/cli/Cargo.toml` から `codex-cloud-tasks` の通常依存を削除する。
- `codex-rs/cli/src/main.rs` から `CloudTasksCli` の import、`Cloud` / `cloud-tasks` サブコマンド、`codex_cloud_tasks::run_main(...)` の dispatch を削除する。
- `codex-rs/Cargo.toml` の workspace members から `cloud-tasks`、`cloud-tasks-client`、`cloud-tasks-mock-client` を削除する。
- `codex-rs/Cargo.toml` の workspace dependencies から Cloud Tasks client 2 crate を削除する。
- Cloud Tasks 3 crate はファイルを削除せず、Cargo workspace の active member と通常の CLI 依存から外す。上流 rebase で再利用・復元しやすいよう、crate のソース、`Cargo.toml`、`BUILD.bazel` は保持する。
- `codex-rs/Cargo.lock` を更新する。
- `codex-rs/deny.toml` にある Cloud Tasks 用の依存例外を、不要になった場合に削除する。

## 対象範囲

### 対象

- CLI の `cloud` / `cloud-tasks` サブコマンド
- Cloud Tasks 専用の API client、mock client、UI crate
- それらを workspace と Bazel のビルド対象に含める定義

### 非対象

- 通常のローカル CUI/TUI 実行
- `codex-core` の code-mode/V8 自体
- Cloud Tasks サーバー側の仕様や API
- デスクトップアプリの Cloud 機能

## 注意点

- `codex cloud ...` は CLI surface から完全になくなる。
- Cloud Tasks crate のソース、`Cargo.toml`、`BUILD.bazel` は削除しない。
- Cloud Tasks crate を workspace member として残すだけでは、workspace 全体の `cargo test` 等で再びビルドされるため、関連 crate は workspace members から外す。
- `codex-cli` から依存だけを削っても、`main.rs` の clap 定義と dispatch が残るとビルドできない。
- `tui/src/public_widgets/composer_input.rs` の Cloud Tasks 言及はコメントであり、通常コードの依存ではない。
- `cli/src/mcp_cmd/cloud_config.rs` に残すコメントアウト済みの cloud-managed configuration 実装は意図的な残置である。通常の custom CLI ビルドから cloud 依存を外しつつ、上流との差分を小さく保ち、rebase 時に cloud 経路の変更やコンフリクトを見つけやすくするため、削除せず参照用に保持する。
- 上流 rebase では Cloud Tasks の crate 追加、CLI の `Cloud` サブコマンド、Cargo/Bazel の workspace 定義が競合候補になる。Cloud Tasks を戻さない方針を維持し、必要なら上流側の変更を custom から除外する。

## 動作確認手順

エージェント側では build/test を実行しない。利用者の手元で次を確認する。

```sh
rg -n "cloud.?tasks|CloudTasksCli|codex_cloud_tasks" codex-rs \
  --glob '!cloud-tasks/**' \
  --glob '!cloud-tasks-client/**' \
  --glob '!cloud-tasks-mock-client/**'

cd codex-rs
cargo metadata --no-deps --format-version 1
make almost
```

確認項目:

- `codex cloud` / `codex cloud-tasks` が CLI help に存在しない。
- workspace metadata に Cloud Tasks 3 crate が含まれない。
- `make almost` の compile/link ログに `codex-cloud-tasks` が出ない。
- ローカル CUI の通常起動と既存テストが通る。

## つまずきと対処

- `make almost` は workspace 全体の `cargo test` を実行するため、CLI 依存を外しただけでは Cloud Tasks crate が残っている限りビルドされることがある。workspace members からも外す。
- `Cargo.lock` には `codex-cli` の依存と Cloud Tasks 3 package のエントリが残るため、手編集で済ませず Cargo の lockfile 更新で整合性を確認する。
- Bazel の Cloud Tasks target は上流 rebase で戻しやすいようファイルとして保持する。通常の Cargo/`make almost` 経路からは外れるが、Bazel で明示的に target を指定すればビルドできる状態を残す。

## 関連ファイル一覧

- `CUSTOM.md`
- `_docs/custom_notes/README.md`
- `codex-rs/cli/Cargo.toml`
- `codex-rs/cli/src/main.rs`
- `codex-rs/Cargo.toml`
- `codex-rs/Cargo.lock`
- `codex-rs/deny.toml`
- `codex-rs/cloud-tasks/`
- `codex-rs/cloud-tasks-client/`
- `codex-rs/cloud-tasks-mock-client/`
