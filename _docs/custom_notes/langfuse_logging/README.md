# Langfuse へログを記録する改造（検討メモ）

## 2025-12-31 実装したカスタム（このフォーク固有）

Langfuse の OTEL 取り込み（`langfuse.*` attribute mapping）で **「LLM をどう呼んだか」** を見える化するため、
Codex 側に以下を追加しました。

重要:

- Codex 側の span attribute は `codex.*` として出し、Collector 側で `langfuse.*` へ remap します（Langfuse 固有キーを Codex から排除するため）。

- **修正: TUI セッションでも trace が欠けにくいようにする**
  - `new_session` span を「短命 root span」に変更し、親コンテキストのみ保持して子 span をぶら下げる
  - 長寿命セッションで “parent 404 / trace row 不在（trace が生成できないように見える）” が起きやすい問題の修正

- **機能追加: trace 名を `codex_{session_id}` にする**
  - `new_session` span に `codex.trace.name` を付与（Collector が `langfuse.trace.name` に remap）
  - `/resume` で使う session id と揃える目的

- **機能追加: LLM の入出力（prompt/response）全量を Langfuse に載せる**
  - `llm_generation` span を作り、`codex.observation.input` / `codex.observation.output` に JSON 文字列で格納（Collector が `langfuse.observation.*` に remap）
  - 形式: `instructions` / `input`（`ResponseItem[]`） / `tools` / `parallel_tool_calls` / `output_schema`
  - `codex.observation.type=generation` / `codex.observation.model.name=<model>`

- **機能追加: ツール実行の入出力を Langfuse に載せる**
  - `tool_exec` span を作り、`codex.observation.input` / `codex.observation.output` に JSON 文字列で格納（Collector が `langfuse.observation.*` に remap）
  - `arguments` / `output` はサイズ肥大を避けるため上限で truncate する（truncate 有無と元の長さも記録する）

- **機能追加: ツール呼び出し（モデルの意図）を Langfuse に載せる**
  - `tool_call` span を作り、モデルが出した tool call を `codex.observation.input` に JSON 文字列で格納（Collector が `langfuse.observation.input` に remap）
  - `tool_exec` との差分:
    - `tool_call`: モデルが「この tool をこの引数で呼べ」と出した時点のログ（実行前）
    - `tool_exec`: 実際の実行結果（duration / success / output）まで含むログ（実行後）

- **機能追加: API 呼び出しごとに `api_request` span を作る**
  - retry/latency/status を trace ツリーで追う目的（events だけだと UI で辿りづらい）
  - Langfuse の表示はノイズになりやすいので、Collector 側のフィルタで普段は落とす運用が想定

重要な注意:

- **prompt は未マスクで入ります**（デバッグ用途）。秘匿情報を含む環境では有効化しないでください。
- この実装は `langfuse.*` の attribute mapping に依存しており、Codex 本家へ upstream しづらい（Langfuse 固有）です。

関連ファイル:

- `codex-rs/otel/src/otel_manager.rs`
- `codex-rs/core/src/client.rs`
- `codex-rs/core/src/tasks/regular.rs`
- `/_docs/custom_notes/langfuse_logging/codex_config_langfuse_local.toml.example`（設定サンプル・秘匿なし）
- `/_tmp/otel/launch.sh`（Collector 起動スクリプト。`codex.*` → `langfuse.*` remap とフィルタを実施）
- `/_docs/custom_notes/langfuse_logging/current_status.md`（進行中の状況メモ）

## 目的

- Codex の会話/ツール実行/モデル呼び出しのログを Langfuse に集約し、デバッグや品質分析をしやすくする。
- 可能なら「コード改造なし（設定だけ）」でまず試せる形に寄せる。

## 結論（現状の最短ルート）

Codex は既に OpenTelemetry(OTEL) の OTLP export を持っているため、Langfuse 側が OTLP ingest を受けられる構成（または OpenTelemetry Collector で中継/変換できる構成）なら、**Codex 側の改造なしで** Langfuse 連携が成立する可能性が高い。

参考:
- OTEL 設定/イベントカタログ: `docs/config.md`（`[otel]` セクション）
- 実装: `codex-rs/otel/` と `codex-rs/core/src/otel_init.rs`

## 現状の Codex 側の観測（どこまで出ているか）

- `codex-otel` が `tracing::event!` を発行し、それを OTLP logs として export できる。
  - 例: `codex.conversation_starts`, `codex.api_request`, `codex.sse_event`, `codex.user_prompt`, `codex.tool_decision`, `codex.tool_result`
  - 形は `docs/config.md` の “Event catalog” を参照。
- `otel.trace_exporter` を有効にすると、`tracing-opentelemetry` により span も OTLP traces へ export できる（`codex-rs/otel/src/otel_provider.rs`）。

## 方針案

### 案A: 「改造なし」OTLP →（Collector）→ Langfuse

1. Codex 側は既存の `otel.exporter` / `otel.trace_exporter` を有効化して OTLP 送信。
2. OpenTelemetry Collector を立てて、受けた OTLP を Langfuse 側に転送（Langfuse が OTLP ingest を持たない場合は Collector 側で変換/ブリッジする）。

メリット:
- Codex のソース改造なしで始められる可能性が高い。
- 送信先や認証・フィルタリングは Collector 側の責務にできる。

デメリット/注意:
- Langfuse の受け口（OTLP ingest の可否、必要なヘッダ/認証方式、マッピング仕様）に依存。
- `codex.tool_result.output` 等に秘匿情報が混入しうるので、Collector 側でのフィルタ/マスクが実務上ほぼ必須。

Codex 側の設定例（概念、実際は `CODEX_HOME/config.toml` に記述）:

```toml
[otel]
environment = "dev"
log_user_prompt = false

# logs（codex_otel の tracing::event!）
exporter = { otlp-http = { endpoint = "http://127.0.0.1:4318/v1/logs", protocol = "binary", headers = { } } }

# traces（span）
trace_exporter = { otlp-http = { endpoint = "http://127.0.0.1:4318/v1/traces", protocol = "binary", headers = { } } }
```

補足:
- headers にトークンを渡せる（`docs/config.md` の例参照）。秘密値は環境変数展開を使う運用が前提。

### 案A-2: 設定で “任意メタデータ” を付与できるようにする（改造あり・小）

目的:
- Langfuse（や OTEL 側）で「プロジェクト名」「環境」「チーム」などでフィルタ/集計できるようにする。
- 既存の `conversation.id` だけだと横断分析しづらい問題を、ユーザー設定でコントロール可能にする。

設計の方向性（案）:
- `config.toml` に任意の key-value を書けるようにし、OTEL の **Resource attributes** と各イベントのフィールドに付与する。
  - 例: `project.name`, `project.root`, `repo.url`, `deployment.id`, `owner.team` など
- “どのプロジェクトか” を自動推定するかどうかも設定で制御できるようにする（推奨は手動指定 + 最小限の自動補助）。

設定案（例）:

```toml
[otel]
environment = "dev"

[otel.attributes]
project.name = "my-monorepo"
project.root = "/Users/alice/src/my-monorepo"
owner.team = "infra"
```

実装フック候補:
- `codex-rs/otel/src/otel_provider.rs` の `make_resource(...)` に `Resource` の属性追加。
- `codex-rs/otel/src/otel_manager.rs` の `OtelEventMetadata` に保持して、各 `tracing::event!` にも同様の属性を付与。
  - どちらか片方だけでも良いが、転送先（Langfuse/Collector）の取り回し的に、Resource と event field の両方にあると便利な場面が多い。

注意:
- “cwd から自動で project.name を決める” は便利だが、意図せず機微情報（ユーザー名を含む絶対パス等）を外部へ出しやすい。
- まずは **ユーザーが明示指定** できるようにし、必要なら「`git` が使える時だけ repo 情報を推定」など段階的に拡張するのが安全。

### 案B: Langfuse API へ直接送信する（Codex 改造あり）

Langfuse の “trace / span / observation” 概念に合わせて、Codex のセッション/ターン/ツール呼び出し/モデル呼び出しを Langfuse のデータモデルへ写像し、HTTP API で送る。

実装の置き場所案:
- 新 crate `codex-langfuse` を追加し、`codex-core` からは trait 経由で呼ぶ（依存方向と feature で分離）。
- 既存の `codex-otel` に「Langfuse exporter」を追加する（ただし `codex-otel` は “OpenTelemetry exporter” としての責務が明確なので、責務が混ざりやすい）。

どこにフックするか（候補）:
- 既存イベント生成の中心: `codex-rs/otel/src/otel_manager.rs`
  - `conversation_starts`, `record_api_request`, `tool_decision`, `tool_result`, `user_prompt` 等を Langfuse にも二重送信する形がわかりやすい。
- 追加で「会話履歴」を送りたいなら、`rollout`（`codex-rs/core/src/rollout/recorder.rs`）を“完了後にまとめて送る”設計もあり（オンライン送信と比べて取りこぼしが少ない）。

メリット:
- Langfuse の UI/機能（trace ツリー、入出力、メタデータ）に合わせた表現ができる。

デメリット/注意:
- API 仕様追従・失敗時リトライ・バッファリング・秘密情報マスキング等の実装コストが高い。
- 送信失敗で CLI 体験を劣化させないため、非同期 + バックプレッシャ + 落ちても動く設計が必要。

### 案C: “notify” フックで外部スクリプトが Langfuse へ送る（最小改造または改造なし）

`docs/config.md` の `notify` を使い、ターン完了イベントを外部プログラムへ渡して、そこで Langfuse に送る。

メリット:
- Codex 本体に依存追加せずに PoC ができる。

デメリット:
- 取得できる情報が「ターン完了」中心で粒度が粗い（ツール実行/モデル API の詳細は取りづらい）。

## セキュリティ/プライバシー注意点（重要）

- `otel.log_user_prompt = false` がデフォルトで、ユーザー入力は赤字化されるが、**ツール出力やツール引数**はイベントに載る（`codex.tool_result.arguments/output`）。
- Langfuse に送る場合は、少なくとも以下のどれかが必要になりやすい:
  - Collector 側で “特定イベント/フィールドを drop または mask”
  - Codex 側で “イベント生成前に redact” する仕組み（allowlist/denylist）
  - 送信対象を “trace のメタデータのみ” に絞る（PoC 段階）
  - （案A-2 を採用する場合）`otel.attributes` に「パスやユーザー名が入らない」運用ルールを設ける

## 動作確認手順（手元環境）

- Langfuse 連携の前段として、まずローカルの OpenTelemetry Collector に届くことを確認する。
  - `CODEX_HOME/config.toml` に `[otel]` を設定
  - Collector を `4318/v1/logs` と `4318/v1/traces` で待受
  - Codex を起動して 1 ターン実行し、Collector 側で受信を確認
- その後、Collector の exporter を Langfuse 側に向けて転送し、Langfuse UI で trace/log が見えるか確認する。

## つまずきと対処（想定）

- Langfuse が OTLP を直接受けない:
  - Collector 側で Langfuse 向け exporter/bridge が必要（要調査）。
  - 代替として案B（HTTP API 直送）か案C（notify スクリプト）に切り替える。
- 情報過多/秘匿情報混入:
  - 最初は `otel.log_user_prompt=false` のまま、送信対象を絞る（Collector でフィルタ）。
  - ツール結果の `output` を落とす/短縮する（Collector または Codex 改造）。

## 関連ファイル一覧

- OTEL 設定とイベント一覧: `docs/config.md`
- OTEL 初期化: `codex-rs/core/src/otel_init.rs`
- exporter 実装: `codex-rs/otel/src/otel_provider.rs`
- イベント発行: `codex-rs/otel/src/otel_manager.rs`
