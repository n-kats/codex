# exec_mcp_background_stop

## 目的

- `_mcp/exec_mcp` の build/test 系 tool を、実行中でもサーバー全体の応答を止めにくい形にする。
- 実行中の build/test を、サーバー自体を停止せずに外から止められるようにする。

## 変更内容（何がどう変わるか）

- `_mcp/exec_mcp/server.py` の長時間コマンド実行を `asyncio.create_subprocess_exec()` に切り替えた。
  - これにより、長い `cargo test` / `cargo build` / `make all` 相当の処理を待っている間も、サーバーは他の MCP リクエストに応答しやすくなる。
- 実行中ジョブを追跡するレジストリを追加した。
  - `list_active_runs` で現在動いている job を確認できる。
  - `stop_active_runs` で、実行中の build/test job を外から停止できる。
- 停止時はプロセスグループごと止めるため、`cargo` の子プロセスも残りにくい。
- 長時間 tool のレスポンスに `run_id` と `cancelled_by_stop` を追加し、停止されたかどうかを判別しやすくした。

## 対象範囲（非対象も）

- 対象:
  - `_mcp/exec_mcp/server.py`
  - `_mcp/exec_mcp/tests/test_server.py`
- 非対象:
  - `codex-rs` 側の MCP 実装
  - リポジトリ本体の `Makefile`
  - 既存の `_local/mcp.compose.yml` の mount/port 設定

## 注意点（環境差・既知の制約）

- `stop_active_runs` は、MCP サーバーを止めずに子プロセスを止めるための操作。
- `stop_active_runs` は現在動いているジョブ全体を止めるので、必要なら `run_id` か `tool_name` で絞る。
- プロセスグループを止める実装は Unix 前提の挙動を含む。Windows での挙動は別途確認が必要。
- `run_id` は tool 呼び出し単位の一時 ID で、永続 ID ではない。

## 動作確認手順（手動・テスト・スナップショット）

- テスト:
  - `python3 -m unittest discover -s _mcp/exec_mcp/tests`
- 手動確認:
  - `list_active_runs` を呼ぶ
  - 長時間の `run_cargo_test` か `run_make_all_equivalent` を開始する
  - その最中に `stop_active_runs` を呼ぶ
  - 呼び出し中の tool が終了し、`list_active_runs` が空になることを確認する

## つまずきと対処（警告や失敗の修正）

- `stop_active_runs` を呼んでも止まらない:
  - `force=true` で再実行して、`SIGKILL` 相当で止める。
  - 子プロセスが別グループに逃げていないか `list_active_runs` の `pid` を確認する。
- `run_cargo_test` がサーバーを塞ぐ:
  - `server.py` が同期 `subprocess.run()` に戻っていないか確認する。
  - tool 実装が `async def` のままか確認する。
- テストが `python3` を見つけられない:
  - 実行環境の PATH と Python 3 の可用性を確認する。

## 関連ファイル一覧

- `_mcp/exec_mcp/server.py`
- `_mcp/exec_mcp/tests/test_server.py`
- `_docs/custom_notes/README.md`
