# MCP実装の怪しい箇所メモ（2026-02-13）

このメモは、`mcpサーバー関係の実装バグ` 調査で見つかった「挙動不審ポイント」を、
再現条件・影響・根拠コード付きで整理したもの。

## 1. `config/mcpServer/reload` が即時反映ではない（旧設定が残って見える）

- 症状:
  - リロード直後に、変更前のMCP設定で動いているように見える。
- 根拠:
  - app-server側で「次のアクティブターンに適用」と明記されている。
  - `codex-rs/app-server/src/codex_message_processor.rs:3443`
  - 実装も pending に積むのみ。
  - `codex-rs/core/src/codex.rs:3096`
  - 実際の適用は次ターン開始時に `take()` して実行。
  - `codex-rs/core/src/codex.rs:2547`
- 再現イメージ:
  - `config.toml` の MCP URL を更新。
  - `config/mcpServer/reload` を実行。
  - 同一スレッドで次のターンを開始する前に観測すると、旧接続の結果が見える場合がある。
- 影響:
  - 「リロードしたのに反映されない」「別サーバーが反応した」と誤解しやすい。
- 優先度:
  - High（誤動作というよりUX/運用上の強い混乱要因）。

## 2. `codex_apps` ツール一覧キャッシュが URL 非依存（古いツールセット残留リスク）

- 症状:
  - URLや認証情報を切り替えた直後でも、`codex_apps` のツール一覧だけ古い内容が残る可能性。
- 根拠:
  - `codex_apps` だけ1時間キャッシュ。
  - `codex-rs/core/src/mcp_connection_manager.rs:89`
  - 参照は `server_name == CODEX_APPS_MCP_SERVER_NAME` 条件のみで、URL/headers/tokenをキーにしていない。
  - `codex-rs/core/src/mcp_connection_manager.rs:993`
  - `codex-rs/core/src/mcp_connection_manager.rs:1011`
  - キャッシュ保存内容も `tools` と `expires_at` のみ。
  - `codex-rs/core/src/mcp_connection_manager.rs:1027`
- 補足:
  - 通常の Streamable HTTP MCP はURLを使って接続するため「URLを完全に無視」しているわけではない。
  - `codex-rs/core/src/mcp_connection_manager.rs:968`
  - `codex-rs/core/src/mcp_connection_manager.rs:981`
- 影響:
  - 実接続先は更新されていても、表示・選択候補のツール情報が遅延更新され、取り違えに見える。
- 優先度:
  - High（設定更新直後の一貫性破壊）。

## 3. ツール名衝突時に「後続をスキップ」するため期待ツールが消える

- 症状:
  - サニタイズ後の同名衝突や重複時に、あるサーバー側ツールが一覧から落ちる。
- 根拠:
  - 競合時は `warn!("skipping duplicated tool ...")` の上で `continue`。
  - `codex-rs/core/src/mcp_connection_manager.rs:148`
  - これが「別サーバーが反応する」に見える可能性がある（実際には片方が表示/解決対象から消失）。
- 影響:
  - 期待サーバーのツールが選ばれず、別の同名系ツールに見える挙動が発生。
- 優先度:
  - Medium（条件付きだがデバッグ困難）。

## 4. MCPサーバー（`codex-mcp-server`）で未応答リクエストがある

- 症状:
  - `resources/*` や `prompts/*` など一部メソッドに応答が返らず、クライアントが待ち続ける。
- 根拠:
  - リクエストは分岐しているが、ハンドラがログのみで `send_response/send_error` しない。
  - `codex-rs/mcp-server/src/message_processor.rs:85`
  - `codex-rs/mcp-server/src/message_processor.rs:259`
- 影響:
  - MCPクライアント側タイムアウト、またはハング。
- 優先度:
  - High（プロトコル違反寄り）。

## 5. Exec承認の応答受信失敗時に deny せず停止し得る

- 症状:
  - `elicitation/create` の応答受信失敗時、Exec承認が確定せずターン進行が止まる可能性。
- 根拠:
  - 受信失敗分岐で `return` のみ。
  - `codex-rs/mcp-server/src/exec_approval.rs:118`
  - 一方、Patch承認側は失敗時に `Denied` を submit しているため非対称。
  - `codex-rs/mcp-server/src/patch_approval.rs:103`
- 影響:
  - ツール呼び出しの完了が返らず、セッションが詰まる。
- 優先度:
  - High。

## 6. 異常終了時に request_id -> thread_id マップが掃除されない経路

- 症状:
  - エラー終了経路でマップ削除されず、長期でゴミが残る。
- 根拠:
  - 正常終了（TurnComplete）では remove あり。
  - `codex-rs/mcp-server/src/codex_tool_runner.rs:289`
  - エラーイベント/ランタイムエラー分岐では remove なし。
  - `codex-rs/mcp-server/src/codex_tool_runner.rs:243`
  - `codex-rs/mcp-server/src/codex_tool_runner.rs:377`
- 影響:
  - 将来的な cancel 対応で誤スレッドへ作用するリスク、メモリリーク傾向。
- 優先度:
  - Medium。

## 改修候補（短期）

- `config/mcpServer/reload`:
  - API応答に「deferred適用」である旨を明記（通知/レスポンスフィールド追加）。
  - 即時適用オプションの導入検討。
- `codex_apps` キャッシュ:
  - キーに `url + auth-context` を含める、または refresh 時に明示 invalidation。
- `mcp-server`:
  - 未実装requestは `METHOD_NOT_FOUND` か `not implemented` を必ず返す。
  - exec approval の oneshot失敗時に `Denied` をsubmit。
  - すべての終了分岐で `running_requests_id_to_codex_uuid` を確実に掃除。
