# exec_mcp_make_bridge

## 目的

- `_mcp/exec_mcp/server.py` から、Docker をネストせずに `make build` / `make all` 相当のビルド・テストを実行できるようにする。
- `_local/mcp.compose.yml` に MCP 用のキャッシュ/ホーム設定を反映し、`CARGO_TARGET_DIR` を `/tmp/codex-target`（container 内）へ bind mount して実行する。

## 変更内容（何がどう変わるか）

- `exec_mcp` に次の MCP ツールを追加した。
  - `check_env`: Rust/compose のビルド環境として解決されるパス情報を返す
  - `run_make_build_equivalent`: `cargo build -p codex-cli --bin codex`
  - `run_make_all_equivalent`: `cargo +nightly fmt` -> Linux 時は `codex-linux-sandbox` をビルド -> `cargo test --all-features`
  - `run_make_almost_equivalent`: `cargo +nightly fmt` -> Linux 時は `codex-linux-sandbox` をビルド -> `cargo test -- --skip ...`
  - `run_cargo_test_selected`: crate とテスト名を指定して個別テストだけ実行
- 固定の `run_cargo_check` / `run_cargo_build` / `run_cargo_test` / `run_cargo_fmt_check` / `run_cargo_clippy` は `codex-rs` を作業ディレクトリにし、コンテナに渡された `CODEX_HOME` / `CARGO_HOME` / `RUSTUP_HOME` / `CARGO_TARGET_DIR` を優先して実行する。
- `CARGO_TARGET_DIR` は `Makefile` と同じホスト側 `target` を使う（`scripts/docker_run.sh` の `CODEX_DOCKER_TARGET_DIR` と揃える）。
  - `.env` の消費は Docker 起動時（compose / docker run）に行い、コンテナ環境変数 `CARGO_TARGET_DIR` と bind mount をそこで揃える。
  - `exec_mcp` は `.env` を解釈せず、渡された `CARGO_TARGET_DIR`（無ければ `CODEX_DOCKER_TARGET_DIR`）をそのまま使う。
- `CODEX_DOCKER_CACHE_DIR` だけは repo 外の外部ディスクを許可し、その配下の `docker/cargo` / `docker/rustup` / `docker/home` / `docker/target` を実体として使う。
- `_local/prepare_mcp_docker_env.sh` は cache root を決めて `docker/*` を `mkdir -p` するだけにして、`_local/codex.sh` から呼ぶ。
- `_local/codex.sh` は compose 起動時だけ shell 環境の `CODEX_CACHE_DIR` を一時的に外して `.env` を優先させる。
- 実行ログは `_tmp/*_test_result.txt` に保存する。`exec_mcp` の tool ログも `_tmp/exec_mcp` に揃える。
- `exec_mcp` 用 Dockerfile に Rustup と nightly rustfmt を追加し、さらに Node.js を入れて artifact runtime テストが JS runtime を見つけられるようにした。
- `exec_mcp` 用 Dockerfile に `bubblewrap` を追加し、MCP 側でも bwrap 系テストや sandbox 前提を満たせるようにした。
- `server.py` は compose で `/app/server.py` に bind mount して起動する。

## 対象範囲（非対象も）

- 対象: `_mcp/exec_mcp/server.py`、`_mcp/exec_mcp/docker/Dockerfile`、`_local/mcp.compose.yml`
- `make clean` / `clean-dry-run` は MCP compose が使う `_tmp/codex_build/codex_cache` も対象にする。

## 注意点（環境差・既知の制約）

- `run_make_all_equivalent` は `make all` と同じく、fmt が失敗しても後続のテストを続けて最後に失敗を返す。
- `run_make_almost_equivalent` は `make almost` と同じく、fmt が失敗しても後続の test-almost を続けて最後に失敗を返す。
- `run_cargo_test_selected` で Linux sandbox が必要なテストは `build_linux_sandbox=true` を付ける。
- compose の cache/home/target の bind mount は `../_cache/docker/*` を前提にしているため、初回起動時にホスト側へディレクトリが作られる。
- `server.py` の変更はコンテナ再作成で反映でき、image の再 build は不要。Dockerfile や apt/rustup 依存を変えたときだけ build が必要。

## 動作確認手順（手動・テスト・スナップショット）

```sh
docker compose -f _local/mcp.compose.yml up -d --force-recreate
```

- MCP クライアントから `run_make_build_equivalent` を呼ぶ。
- MCP クライアントから `run_cargo_check` を呼び、`/workspace/codex-rs` 起点で実行されることを確認する。
- MCP クライアントから `run_cargo_test_selected(package="codex-tui", test_name="enter_inserts_newline_instead_of_submitting", target_kind="lib")` を呼ぶ。
- MCP クライアントから `run_make_all_equivalent` を呼ぶ。

## つまずきと対処（警告や失敗の修正）

- `cargo +nightly fmt` が見つからない: `exec_mcp` イメージを再 build して nightly toolchain を取り込む。
- `no compatible JavaScript runtime found for artifact runtime ...` が出る: `exec_mcp` イメージに Node.js が入っているか確認する。
- `bwrap: No permissions to create a new namespace ...` や `bwrap` not found が出る: `exec_mcp` イメージに `bubblewrap` が入っているか確認する。
- `server.py` を変えたのに tool 一覧が更新されない: `docker compose -f _local/mcp.compose.yml up -d --force-recreate` でコンテナを作り直す。
- `target` で権限エラーになる:
  - ホスト側 `CODEX_DOCKER_TARGET_DIR`（未指定なら `../_cache/docker/target`）がコンテナの実行ユーザーで書けることを確認する。
  - コンテナ側 `CODEX_DOCKER_TARGET_DIR_IN_CONTAINER`（未指定なら `/tmp/codex-target`）が `CARGO_TARGET_DIR` と一致していることを確認する。
  - `CARGO_HOME` / `RUSTUP_HOME` の権限エラーは従来どおり `../_cache/docker` 配下の owner/permission を compose の `1000:1000` で書ける状態にする。
- root 制約で弾かれる: `server.py` が `error` ログで `CODEX_DOCKER_CACHE_DIR` / `CODEX_HOME` / `CARGO_HOME` などの変数名、root、実際の path、raw 値を出すので、そのログを見て compose 側の値を揃える。
- `CODEX_DOCKER_CACHE_DIR` を外部ディスクに置く場合は warning が出るが、これは想定どおり。
- `.env` を勝たせたいのに shell 側の `CODEX_CACHE_DIR` が効いてしまう場合は、`_local/codex.sh` から起動する構成に寄せる。
- compose 起動前に `cargo` / `rustup` / `home` / `target` が存在するので、Docker が mount 元を root で新規作成する経路を避けられる。
- 個別テストで `codex-linux-sandbox` 不足になる: `run_cargo_test_selected(..., build_linux_sandbox=true)` を使う。

## 関連ファイル一覧

- `_mcp/exec_mcp/server.py`
- `_mcp/exec_mcp/docker/Dockerfile`
- `_local/mcp.compose.yml`
- `_local/prepare_mcp_docker_env.sh`
- `scripts/docker_run.sh`
- `Makefile`

The MCP compose target directory is `/workspace/_tmp/codex_build/codex_cache`. The
`EXEC_MCP_EXPECTED_CARGO_TARGET_DEV` value from `.env` is required. The host-side
preparation script and MCP server compare the resolved target filesystem device
before creating directories or starting Cargo.
