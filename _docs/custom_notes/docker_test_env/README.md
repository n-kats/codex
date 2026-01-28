# docker_test_env

## 目的

- ビルド/テストをホスト環境から切り離し、再現性のある Docker 環境で実行する。

## 変更内容（何がどう変わるか）

- `Makefile` のビルド/テスト/整形/検証/insta 系ターゲットは Docker で実行される。
- `docker/Dockerfile` で Ubuntu 24.04 ベースのビルド環境を作る。
- ランナーは `scripts/docker_run.sh` で、`bash scripts/docker_run.sh "<command>"` で実行する。

## 対象範囲（非対象も）

- 対象: `Makefile` の build/test/lint/fmt/verify/insta/run-tui など。
- 非対象: Docker を使わないローカル実行（必要なら直接 `cargo ...` を使う）。

## 注意点（環境差・既知の制約）

- Docker イメージは `make docker-build` で作成する。
- `scripts/docker_run.sh` は `bash` 経由で実行するため、実行権限は不要。
- コンテナ内の実行ユーザーは `ubuntu`（ベースイメージの標準ユーザー）。
- `CODEX_DOCKER_IMAGE_NAME` / `CODEX_DOCKER_PLATFORM` / `CODEX_DOCKER_CACHE_DIR` を必要に応じて設定する。
- `.env` に `CODEX_DOCKER_TARGET_DIR` を設定すると、`target` を別ストレージに分離できる。
- キャッシュは `/_cache/docker` 以下に保存される（`CARGO_HOME`/`RUSTUP_HOME`/`HOME` を分離）。
- `codex-rs/exec-server/tests` は `dotslash` を使ってテスト用 bash を用意するため、Docker イメージに `dotslash` を含めている。

## 動作確認手順（手動・テスト・スナップショット）

```sh
make docker-build
make fmt
make test-core
```

## つまずきと対処（警告や失敗の修正）

- `docker run` でイメージが見つからない: `make docker-build` を先に実行する。
- 依存キャッシュが壊れた: `_cache/docker` を削除して再実行する。

## 関連ファイル一覧

- `docker/Dockerfile`
- `scripts/docker_run.sh`
- `Makefile`
- `.env` は `bash` で読み込まれるため、シェル形式の `KEY=VALUE` を使う。
- `CODEX_DOCKER_TARGET_DIR` はホスト側のパスを指定する（相対パスはリポジトリ直下基準）。
- コンテナ内の `CARGO_TARGET_DIR` は `/var/cache/codex/target` 固定（必要なら `CODEX_DOCKER_TARGET_DIR_IN_CONTAINER` で変更）。
