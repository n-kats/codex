# exec_mcp_make_bridge

## 目的

- `_mcp/exec_mcp/server.py` から、Docker をネストせずに `make build` / `make all` 相当のビルド・テストを実行できるようにする。
- `_local/mcp.compose.yml` に MCP 用のキャッシュ/ホーム設定を反映し、`CARGO_TARGET_DIR` を `/tmp/codex-target`（container 内）へ bind mount して実行する。

## 変更内容（何がどう変わるか）

- `exec_mcp` に次の MCP ツールを追加した。
  - `run_make_build_equivalent`: `cargo build -p codex-cli --bin codex`
  - `run_make_all_equivalent`: `cargo +nightly fmt` -> Linux 時は `codex-linux-sandbox` をビルド -> `cargo test --all-features`
  - `run_cargo_test_selected`: crate とテスト名を指定して個別テストだけ実行
- 固定の `run_cargo_check` / `run_cargo_build` / `run_cargo_test` / `run_cargo_fmt_check` / `run_cargo_clippy` は `codex-rs` を作業ディレクトリにし、コンテナに渡された `CODEX_HOME` / `CARGO_HOME` / `RUSTUP_HOME` / `CARGO_TARGET_DIR` を優先して実行する。
- `CARGO_TARGET_DIR` は `Makefile` と同じホスト側 `target` を使う（`scripts/docker_run.sh` の `CODEX_DOCKER_TARGET_DIR` と揃える）。
  - `.env` の消費は Docker 起動時（compose / docker run）に行い、コンテナ環境変数 `CARGO_TARGET_DIR` と bind mount をそこで揃える。
  - `exec_mcp` は `.env` を解釈せず、渡された `CARGO_TARGET_DIR`（無ければ `CODEX_DOCKER_TARGET_DIR`）をそのまま使う。
- 実行ログは `_tmp/*_test_result.txt` に保存する。
- `exec_mcp` 用 Dockerfile に Rustup と nightly rustfmt を追加し、`server.py` は compose で `/app/server.py` に bind mount して起動する。

## 対象範囲（非対象も）

- 対象: `_mcp/exec_mcp/server.py`、`_mcp/exec_mcp/docker/Dockerfile`、`_local/mcp.compose.yml`
- 非対象: 既存の `Makefile` ターゲットそのものの挙動変更

## 注意点（環境差・既知の制約）

- `run_make_all_equivalent` は `make all` と同じく、fmt が失敗しても後続のテストを続けて最後に失敗を返す。
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
- `server.py` を変えたのに tool 一覧が更新されない: `docker compose -f _local/mcp.compose.yml up -d --force-recreate` でコンテナを作り直す。
- `target` で権限エラーになる:
  - ホスト側 `CODEX_DOCKER_TARGET_DIR`（未指定なら `../_cache/docker/target`）がコンテナの実行ユーザーで書けることを確認する。
  - コンテナ側 `CODEX_DOCKER_TARGET_DIR_IN_CONTAINER`（未指定なら `/tmp/codex-target`）が `CARGO_TARGET_DIR` と一致していることを確認する。
  - `CARGO_HOME` / `RUSTUP_HOME` の権限エラーは従来どおり `../_cache/docker` 配下の owner/permission を compose の `1000:1000` で書ける状態にする。
- 個別テストで `codex-linux-sandbox` 不足になる: `run_cargo_test_selected(..., build_linux_sandbox=true)` を使う。

## 関連ファイル一覧

- `_mcp/exec_mcp/server.py`
- `_mcp/exec_mcp/docker/Dockerfile`
- `_local/mcp.compose.yml`
- `scripts/docker_run.sh`
- `Makefile`
