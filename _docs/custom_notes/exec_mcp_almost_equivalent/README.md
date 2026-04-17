# exec_mcp に make almost 相当を追加する

## 目的

- `_mcp/exec_mcp` から `make almost` 相当の検証を 1 つの MCP tool で起動できるようにする。
- `cargo test` で環境依存の runtime 前提にぶつかったときでも、`make almost` と同じ skip 方針を MCP 側で再現できるようにする。

## 変更内容（何がどう変わるか）

- `_mcp/exec_mcp/docker/Dockerfile` に Node.js を追加し、artifact runtime テストが `node` を見つけられるようにした。
- `_local/mcp.compose.yml` に `EXEC_MCP_LOG_DIR=/workspace/_tmp/exec_mcp` を明示し、MCP tool の結果ログを workspace 内 `_tmp` に揃えた。
- `_mcp/exec_mcp/server.py` に `run_make_almost_equivalent()` を追加した。
  - `cargo +nightly fmt`
  - Linux では `codex-linux-sandbox` の事前ビルド
  - `cargo test -- --skip ...` に `Makefile` の `SKIP_ALMOST_TESTS` と同じ除外リストを適用

## 対象範囲（非対象も）

- 対象:
  - `_mcp/exec_mcp/docker/Dockerfile`
  - `_mcp/exec_mcp/server.py`
- 非対象:
  - リポジトリ本体の `Makefile`
  - `codex-rs` 側のテスト本体
  - 上流の `make almost` の定義そのもの

## 注意点（環境差・既知の制約）

- この tool は `make almost` の完全な代替ではなく、`fmt + test-almost` の複合実行を MCP から呼べるようにしたもの。
- Docker イメージに Node を入れても、ホスト側で直接 `cargo test` を回す場合は別途 Node の可用性が必要。
- `cargo test -- --skip ...` の skip は部分一致なので、`Makefile` 側の除外名に合わせて管理する必要がある。

## 動作確認手順（手動・テスト・スナップショット）

- MCP サーバーの `run_make_almost_equivalent()` を実行する。
- 期待値:
  - `cargo +nightly fmt` が先に走る
  - `test-almost` 相当で既知の skip リストが適用される
  - artifact runtime テストが Node 不足で落ちない
- 必要なら `run_cargo_test()` との差分を確認する。

## つまずきと対処（警告や失敗の修正）

- `no compatible JavaScript runtime found for artifact runtime ...` が出る場合:
  - `_mcp/exec_mcp/docker/Dockerfile` に Node が入っているか確認する
  - `ArtifactsClient::execute_build` 系のテストが runtime 前提を満たせているか確認する
- `almost` 相当を追加したのに特定テストがまだ落ちる場合:
  - `SKIP_ALMOST_TESTS` に該当テスト名が入っているか確認する
  - `cargo test -- --list` で実名を確認する

## 関連ファイル一覧

- `_mcp/exec_mcp/docker/Dockerfile`
- `_local/mcp.compose.yml`
- `_mcp/exec_mcp/server.py`
- `Makefile`
- `_docs/custom_notes/test_almost_skip_list/README.md`
- `_docs/custom_notes/make_test_almost_no_fail_fast/README.md`
