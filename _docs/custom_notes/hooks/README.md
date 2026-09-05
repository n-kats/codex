# hooks（外部コマンドフック）設計メモ（未実装）

このドキュメントは、Codex CLI に「外部コマンドフック（hooks）」を追加するための設計メモです。現時点では **未実装** で、方針検討のネタとして残します。

## 目的

- Codex のライフサイクルやユーザー操作に合わせて、外部コマンドを起動できるようにする。
- CI/自動化/通知/監査ログ/メトリクス連携などを、TUI に依存せずに実現できるようにする。
- 既存の `notify`（ターン完了通知）よりも汎用的なイベント・ペイロード・同期制御を提供する。

## 現状（notify の挙動）

- config の `notify` は `Option<Vec<String>>`（argv）で、**1つだけ** 指定できる。
- タイミングは **ターン完了（agent-turn-complete）** のみ。
- 実行は **fire-and-forget（待たない）**。
- JSON は「1引数」として末尾に追加される（固定のメッセージを送りたい場合はラッパースクリプトで吸収する想定）。
- JSON の内容は `thread-id`, `turn-id`, `cwd`, `input-messages`, `last-assistant-message` 等（詳細は `codex-rs/core/src/user_notification.rs` がソースオブトゥルース）。

## hooks を導入するメリット（わかったこと）

- **イベント粒度が増やせる**: 起動/終了、送信/受信、`/` コマンド前後など、ユーザーが欲しいタイミングを素直に表現できる。
- **複数フックが扱える**: 1イベントに対して複数コマンドを登録できる（通知 + ログ + 自動化など）。
- **同期/非同期の選択**: `/command:before` のように「失敗したら止めたい」ケースと、「裏で走ればよい」ケースを分けられる。
- **巨大 payload に耐えられる**: JSON を argv ではなく一時ファイルに逃がして、引数長制限・表示崩れを回避できる。
- **信頼境界を明確化できる**: project config 由来の hooks を既定で無効にする等、任意コマンド実行リスクを制御できる。

## 要件（ユーザー要望ベース）

### フックタイミング（例）

- `codex-start`: Codex 起動
- `session-first-request`: 新規セッションの最初のリクエスト
- `message-sent`: 新規メッセージ送信
- `response-received`: 回答受信
- `codex-exit`: Codex 終了
- `/` コマンド:
  - `slash-command:before`: 実行直前
  - `slash-command:after`: 実行完了後

### パラメータ渡し

- argv の各要素にテンプレートを使える（例: `["bash","my_hook.sh","{{ params.foo }}"]`）。
- `params` 自体が大きくなりうるので、`{{ params.tmp_file }}` のように「イベント JSON を書いた一時ファイル」を渡せる。

## 実装方針案（複数案）

### 案A: 新規 `hooks = [...]` を追加（テンプレは minijinja）

#### 設定イメージ（案）

```toml
hooks = [
  { on = "codex-start", command = ["bash", "/path/hook.sh", "{{ params.tmp_file }}"] },
  { on = "slash-command:before", command = ["bash", "/path/hook.sh", "{{ params.command.name }}"] , wait = true, timeout_ms = 5000, on_error = "fail" },
  { on = "slash-command:after",  command = ["bash", "/path/hook.sh", "{{ params.command.name }}", "{{ params.result.status }}"] },
]
```

#### 期待するセマンティクス（案）

- `on`: イベント名（string）
- `command`: argv（array<string>）。各要素に minijinja の `{{ ... }}` を許可。
- `wait`: `false` なら fire-and-forget、`true` なら完了まで待つ。
- `timeout_ms`: `wait=true` のときの上限。
- `on_error`: `"ignore" | "warn" | "fail"`（`wait=true` かつ `"fail"` の場合にイベント側を失敗として扱える）
- `payload`: `"minimal" | "full"`（デフォルトは `"minimal"` 推奨）

#### テンプレートエンジン

- 独自実装は避け、Rust では `minijinja` を採用する方針。
- 運用の安全性のため、まずは「変数参照（`{{ params.xxx }}`）中心」に絞り、`{% ... %}` 等の制御文は未サポートでもよい。

#### payload / params（案）

- 共通:
  - `params.on`, `params.timestamp`, `params.codex.{version,pid,cwd,project_root}` など
  - `params.tmp_file`: JSON 全量を書いた一時ファイルパス（常に用意する案が堅い）
- イベント固有（例）:
  - `message-sent`: `params.message.{role,text}` など
  - `response-received`: `params.response.{text,usage}` など
  - `slash-command:*`: `params.command.{raw,name,args}`、after は `params.result.{status,exit_code,duration_ms}` など

#### 長所/短所

- 長所: `hooks` として自然、複数イベント・複数コマンド、同期/非同期、テンプレで argv 組み立てが簡単。
- 短所: 新しい設定体系・ドキュメント・互換性配慮が必要（`notify` との関係整理）。

### 案B: `notify` を拡張してイベント配送に寄せる（テンプレ無し）

「テンプレ無しで済む」方向として、`notify` を “イベント配送バス” に育てる案。

#### 設定イメージ（案）

- 互換維持しつつ複数指定を可能にする（例: `notify = [ ["bash","a.sh"], ["bash","b.sh"] ]`）。
- 各エントリにフィルタを持たせる（例: `only = ["agent-turn-complete","codex-start"]`）。

#### 長所/短所

- 長所: Codex 側の実装が単純（JSONを渡すだけ）。テンプレ不要。ルータースクリプト不要で複数実行できる。
- 短所: argv をイベントから組み立てる柔軟性はスクリプト側に寄る。同期/ブロック用途をどうするかが難しい。

### 案C: “配送（notify）” と “同期フック（hooks）” を分離する

- `notify`: 非同期で外部にイベント配送（現在の思想を維持・拡張）
- `hooks`: 同期も扱う（`/command:before` のブロック用途など）

長所: 概念が混ざらず、UI/エラー伝播の設計が明確。
短所: 設定項目が増える。

## 同期/非同期について（要点）

`/command:before` などは「失敗したら止めたい」ニーズがあるため、hooks は **同期/非同期を設定で選べる** のが望ましい。

- 非同期（既定）: 失敗しても Codex 本体の動作を阻害しにくい。通知・ログ用途に向く。
- 同期: ガードレール・自動整形・自動コミットなどに向くが、ハングや遅延を招きやすいので `timeout_ms` が必須。

## セキュリティ/信頼境界（重要）

- project config（`.codex/`）で hooks を許すと「リポジトリを開いただけで任意コマンド実行」になり得る。
- 既定は「ユーザー層の config でのみ hooks/notify を有効」とし、project hooks は opt-in（例: `allow_project_hooks = true`）が安全。

## 動作確認（未実装のため計画のみ）

- `hooks`/`notify` の起動可否（spawn, argv, JSON生成）
- 同期フックの `timeout_ms` と `on_error` の挙動
- payload のサイズが大きいケースで `tmp_file` 経由が機能すること
- project config に hooks を書いても既定で実行されないこと（信頼境界）

## 関連ファイル一覧（現状把握）

- `codex-rs/core/src/user_notification.rs`（notify の JSON スキーマと spawn 実装）
- `codex-rs/core/src/codex.rs`（ターン完了時に notify を呼ぶ）
- `docs/config.md`（notify の説明）
