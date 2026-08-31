## 現状メモ（2026-01-01）

目的:

- Langfuse の trace 一覧で trace name が `new_session` のままにならず、`codex_{session_id}` で表示されること。
- Langfuse の trace 画面で LLM/Tool の観測（In/Out）が読めること。
- Codex 側の実装は Langfuse 固有キーに依存せず、Collector 側で remap する（カスタムをコンパクトにする）。

### 現状の設計

- Codex は span attribute を `codex.*` で出す:
  - `codex.trace.name`（例: `codex_<conversation_id>`）
  - `codex.observation.type`
  - `codex.observation.model.name`
  - `codex.observation.input` / `codex.observation.output`
- Collector が `attributes` processor で `codex.*` → `langfuse.*` に remap して Langfuse OTEL ingest に渡す:
  - `codex.trace.name` → `langfuse.trace.name`
  - `session.id` → `langfuse.session.id`
  - `codex.observation.*` → `langfuse.observation.*`
- Langfuse では `langfuse.trace.name` が無い場合、span 名にフォールバックして `new_session` が表示される。

### 発生している問題

- Langfuse の trace 一覧で name が `new_session` のままになるケースが残っている。
  - これは多くの場合、Collector の remap が適用されず `langfuse.trace.name` が Langfuse 側に届いていないことが原因になり得る。

### 追加した対策（進行中）

- `_tmp/otel/launch.sh`
  - `attributes/langfuse_remap` を追加し、Langfuse に送る traces パイプラインで `codex.*` → `langfuse.*` を upsert。
  - 既定で `LANGFUSE_FILTER_MODE=minimal` とし、Langfuse に送る span を allowlist で限定。
    - `api_request` はノイズになりやすいので minimal では落とす。
  - `cat <<EOF` でバッククォートが評価される問題があったため、生成 YAML 内コメントのバッククォートを撤去。
- `_tmp/otel/otelcol.debug-only.yaml`
  - debug-only collector の stdout でも remap 後のキーが確認できるよう、`attributes/langfuse_remap` を追加。

### 次にやること（証拠取り）

- 実際に Langfuse に送っている Collector の config に `attributes/langfuse_remap` が入っていることを確認する。
- Collector debug 出力で以下が出ているか確認する:
  - `codex.trace.name`
  - `langfuse.trace.name`（remap 後）
- `langfuse.trace.name` が Collector では見えるのに Langfuse UI が `new_session` の場合は、Langfuse ingest 側の挙動（バージョン/バグ/設定）を疑う。

