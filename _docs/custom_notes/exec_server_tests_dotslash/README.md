# exec-server テストで dotslash を必須化（Docker イメージに同梱）

## 目的

`codex-rs/exec-server/tests` のテストが `dotslash` 未導入環境で失敗する問題を回避する。

## 変更内容

- `dotslash` を Docker イメージ（`docker/Dockerfile`）に同梱し、テスト実行環境では `dotslash` が存在する前提にする。
- `create_transport` は DotSlash ファイル（`exec-server/tests/suite/bash`）を `dotslash fetch` してから利用する。

## 対象範囲

- Docker イメージ（`docker/Dockerfile`）。
- `codex-rs/exec-server/tests/common/lib.rs` の `create_transport`（DotSlash を前提）。

## 非対象

- 本番コード（exec-server 本体）は変更しない。

## 注意点

- `dotslash` がインストールされていることが前提。

## 動作確認手順

- `cargo test -p codex-exec-server --test all`
- もしくは `make test-core`（統合テスト経由）

## つまずきと対処

- `Error: No such file or directory (os error 2)`（`dotslash` 実行時）は `dotslash` 未導入が原因。

## 関連ファイル

- `docker/Dockerfile`
- `codex-rs/exec-server/tests/common/lib.rs`
