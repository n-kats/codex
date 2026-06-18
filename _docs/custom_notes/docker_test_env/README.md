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
- `scripts/docker_run.sh` は `--security-opt seccomp=unconfined` と `--security-opt apparmor=unconfined` を付けて、bwrap / user namespace 系の検証を通しやすくしている。
- `CODEX_DOCKER_IMAGE_NAME` / `CODEX_DOCKER_PLATFORM` / `CODEX_DOCKER_CACHE_DIR` を必要に応じて設定する。
- `.env` に `CODEX_DOCKER_TARGET_DIR` を設定すると、`target` を別ストレージに分離できる。
- `scripts/docker_run.sh` はリポジトリ直下の `.env` を host 側で `source` する。
- キャッシュは `/_cache/docker` 以下に保存される（`CARGO_HOME`/`RUSTUP_HOME`/`HOME` を分離）。
- `codex-rs/exec-server/tests` は `dotslash` を使ってテスト用 bash を用意するため、Docker イメージに `dotslash` を含めている。
- `codex-rs/linux-sandbox` / `codex-rs/core` の bwrap 系テストがコンテナ内でも前提を満たせるように、Docker イメージに `bubblewrap` を含めている。
- `codex-rs/exec` / `codex-rs/linux-sandbox` の一部テストは `python3` コマンドを使うため、Docker イメージに `python3` を含めている。
- 非 release の `cargo build` / `cargo test` は `V8_FROM_SOURCE=1` を付けて、`rusty_v8` の prebuilt 404 に依存しない source-build 経路を使う。
- `scripts/docker_run.sh` は Docker 実行前に、bind mount する `cargo` / `rustup` / `home` / `target` ディレクトリがコンテナ内 `ubuntu` ユーザーで書けるかを確認し、必要なら root で `chown` / `chmod` して補正する。

## 動作確認手順（手動・テスト・スナップショット）

```sh
make docker-build
make fmt
make test-core
```

## つまずきと対処（警告や失敗の修正）

- `docker run` でイメージが見つからない: `make docker-build` を先に実行する。
- 依存キャッシュが壊れた: `_cache/docker` を削除して再実行する。
- `_cache/docker/target` などが `root:root` になって `Permission denied` になる: `make` 実行時に `scripts/docker_run.sh` が自動補正する。Docker 自体が使えない場合は手元で owner / mode を確認する。
- `make all` の `codex-linux-sandbox` テスト群で `1 passed; 17 failed` になり、`landlock` / `managed_proxy` がまとめて落ちる:
  - 症状:
    - `suite::landlock::test_root_read` / `suite::landlock::test_no_new_privs_is_enabled` / `suite::managed_proxy::*` などが一斉に失敗する。
    - ログは `expected sandbox denied error, got: Err(Io(Os { code: 2, kind: NotFound, message: "No such file or directory" }))` や `sandbox command should execute: No such file or directory (os error 2)` になる。
  - 切り分け:
    - これは `Permission denied` ではなく、`codex-linux-sandbox` のテストが補助バイナリ解決に失敗している可能性が高い。
    - MCP の `run_make_all_equivalent` が同時期に成功していても、「MCP がテストを skip した」とは限らない。実際には `run_cargo_test_selected(..., exact=true)` で `suite::landlock::test_root_read` と `suite::managed_proxy::managed_proxy_mode_routes_through_bridge_and_blocks_direct_egress` を個別実行して通している。
    - したがって、`make` と MCP の差はネットワーク制限や skip ではなく、テスト実行時の `codex-linux-sandbox` バイナリ解決経路や配置差を疑う。
  - 対処:
    - Docker 側では `python3` を追加して、`codex-rs/exec` / `codex-rs/linux-sandbox` の Python 依存テスト前提を満たす。
    - bwrap が見つからない / `bubblewrap` 系前提不足で落ちる場合は、Docker イメージに `bubblewrap` が入っているか確認する。
    - ただし `landlock` / `managed_proxy` の一斉失敗の本命は Docker イメージ不足ではない。`codex-rs/linux-sandbox/tests/suite/` 側で `codex-linux-sandbox` の解決方法を補強する。
    - 方針としては product code ではなく test code のみを最小変更し、`env!("CARGO_BIN_EXE_...")` 固定参照だけに依存しないようにする。
  - rebase 観点:
    - `codex-rs/linux-sandbox/tests/suite/*.rs` は upstream の通常テストなので、ここへの変更は将来コンフリクトしうる。
    - ただし `Cargo.toml` へ追加依存を入れるより、テスト側の局所修正だけで閉じる方が差分は小さく、rebase 影響も比較的限定的。

## 関連ファイル一覧

- `docker/Dockerfile`
- `scripts/docker_run.sh`
- `Makefile`
- `codex-rs/linux-sandbox/tests/suite/mod.rs`
- `codex-rs/linux-sandbox/tests/suite/landlock.rs`
- `codex-rs/linux-sandbox/tests/suite/managed_proxy.rs`
- `.env` は `bash` で読み込まれるため、シェル形式の `KEY=VALUE` を使う。
- `CODEX_DOCKER_TARGET_DIR` はホスト側のパスを指定する（相対パスはリポジトリ直下基準）。
- MCP compose では、コンテナ内の `CARGO_TARGET_DIR` はデフォルトで `/tmp/codex-target` を使う（必要なら `CODEX_DOCKER_TARGET_DIR_IN_CONTAINER` で変更）。
