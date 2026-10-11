# Trace name が `new_session` のままになるときの切り分け計画（Langfuse OTEL）

## 状況

Langfuse の trace 一覧で trace name が `new_session` のまま更新されないケースがある。

Langfuse 側は OTLP span attribute の `langfuse.trace.name` があればそれを trace.name として採用し、無ければ span 名にフォールバックする
（結果として `new_session` が表示される）。

このフォークでは Codex 側は `codex.trace.name` を出し、Collector が `langfuse.trace.name` に remap する。
このため、まず以下を証拠で確定する:

- Codex → Collector の段階で `codex.trace.name` が出ているか
- Collector → Langfuse に送る段階で `langfuse.trace.name` に remap されているか

## 証拠取り（Collector で受信 span を確認）

### 1) debug-only collector を “別ポート” で起動

`4318` は既に他の collector/Langfuse が使用していることがあるため、ホスト側は衝突回避で別ポートにする。

例:

- コンテナ: 4318（固定）
- ホスト: 14318（任意）

起動例（host networking を使わない場合）:

```bash
docker run --rm --name otel-debug \
  -p 14318:4318 \
  -v "$PWD/_tmp/otel/otelcol.debug-only.yaml:/etc/otelcol-contrib/config.yaml:ro" \
  otel/opentelemetry-collector-contrib:latest \
  --config /etc/otelcol-contrib/config.yaml
```

### 2) Codex の `trace_exporter.endpoint` を debug-only collector に向ける

例（config の一時差し替え）:

- `trace_exporter.endpoint = "http://127.0.0.1:14318/v1/traces"`

設定サンプル（秘匿なし）:

- `/_docs/custom_notes/langfuse_logging/codex_config_langfuse_local.toml.example`

### 3) “チャットの方” を流して collector stdout を確認

collector の debug exporter 出力から、少なくとも以下を確認する:

- span attributes に `codex.trace.name` が存在するか
- span attributes に `langfuse.trace.name` が存在するか（Collector remap 後）
- `session.id` が存在するか

（秘密値は貼らず、attribute key と値の一部だけ確認する）

## 分岐（次の打ち手）

### A) collector stdout に `langfuse.trace.name` が “無い”

Collector remap が効いていない（Collector config に `attributes/langfuse_remap` が入っていない）。
もしくは Codex 側で `codex.trace.name` が OTLP へ流れていない（tracing-opentelemetry/export のどこかで落ちている）。

次の調査:

- どの span に attribute を付けているか（session root / child span）
- exporter が span attributes を落としていないか（protocol/json/binary 差、collector processor）
- 最小再現: 1 span だけ作って attribute が出るか

### B) collector stdout に `langfuse.trace.name` が “ある” のに Langfuse の name が `new_session`

Langfuse ingest 側の問題（バージョン差/バグ/設定）を疑う。

次の調査:

- Langfuse OTEL ingest の最小再現（OTLP 1 span を送って name が反映されるか）
- Langfuse の worker/server 側ログで ingest エラーが出ていないか

## 注意

- `Authorization` など秘匿値は docs/ログに残さない。
- trace name の反映確認は “新規に作った trace” で行う（過去の trace が後から更新されるとは限らない）。
