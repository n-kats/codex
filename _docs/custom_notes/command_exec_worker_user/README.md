# コマンド実行を worker ユーザー（assistant 等）に固定する方針

## 目的

- AI によるコマンド実行（shell / exec_command 等）を低権限ユーザーに固定し、秘匿情報やユーザー環境（ログイン状態・トークン等）へのアクセスを抑える。
- 「危ないコマンドの denylist」ではなく、OS の権限境界（uid/gid）を一次防衛線にする。

## 背景（問題）

- `.env` や各種 secrets が作業ディレクトリやホスト環境に存在する場合、`cat` / `env` / アプリのログ出力などを通じて意図せず漏えいする可能性がある。
- OpenAI アカウントログイン等により作成される非環境変数のトークン（`CODEX_HOME` / `HOME` 配下に保存される情報）が、コマンド実行経路から読めてしまうと危険度が上がる。
- Docker を使う/使わないに関わらず、ユーザー分離ができていないと「うっかり」で境界が崩れやすい。

## 変更内容（方針）

実装時の基本方針は次のとおり。

- Codex 本体（LLM 通信・ログイン状態保持）は従来どおり起動ユーザー（invoker）の権限で動かす。
- コマンド実行系ツール（例: `shell`, `shell_command`, `exec_command` 等）で spawn されるプロセスは、設定で指定した worker ユーザー（例: `assistant`）で実行する（worker_only）。
- これにより、worker から invoker の `HOME` / `CODEX_HOME`（ログイン状態・キャッシュ）へアクセスしない構造を作る。

このノートでは「worker_only」までを対象とし、権限不足時に invoker で再実行する（ask_to_escalate）等は将来の拡張点として扱う。

## 今回のスコープ外

- `ask_to_escalate` は、worker（低権限）での実行に固定せず、以下のようなケースで「起動ユーザー（invoker）の権限で実行しますか？」を対話的に確認する挙動を指す。
  - 事前判定で、権限境界を跨ぐことが明らか（例: `sudo` / `su` / `doas` を含む）
  - 事前判定で、システム領域への書き込み等が明らか（例: `/etc`, `/usr`, `/var` 配下の変更）
  - 事後判定で、worker 実行が `EACCES` / `EPERM`（Permission denied / Operation not permitted）等で失敗した
- 今回実装しない理由:
  - `ask_to_escalate` は「昇格を伴う別ツール（別経路）」として提供する想定であり、このカスタム（worker_only の実装）とは別スコープで進める。
  - 本ノートは「コマンド実行を常に worker に固定する」方針と設定スキーマの整理に限定する。
- その代わり、今回のスコープでは `worker_only`（コマンド実行は常に worker に固定）に限定し、権限が必要な操作は「人間が手動で実行する」前提で運用する。

## 設定（案）

上流との衝突を避けるため、設定は `custom.exec.*` 名前空間に追加する。

- `[custom.exec]`
  - `worker_user = "assistant"`（任意）
  - `worker_uid = 1001`（任意）
  - `worker_gid = 1001`（任意）

### 解決ルール（コンフリクトはエラー）

- 未指定の場合: 従来挙動（起動ユーザーのまま実行）
- `worker_uid` と `worker_gid` は片方だけの指定を許可しない（欠けていればエラー）
- `worker_user` と `worker_uid/gid` が両方指定されている場合:
  - `worker_user` を解決した uid/gid と `worker_uid/gid` が一致していれば OK
  - 一致していなければエラー（曖昧さは許容しない）

## 対象範囲 / 非対象

- 対象: コマンド実行を伴うツール経路（shell / unified exec / MCP shell 等の「子プロセス spawn」）
- 非対象: Codex 本体の API 呼び出し、ログイン状態の保持、モデル通信（invoker 側に残す）

## 注意点（Docker / ホスト）

- Docker の bind mount では、ホスト側ファイルの所有 UID/GID がコンテナ内でも見えるため、worker の uid/gid が合わないと `/workspace` に書けないことがある。
  - 対策は運用で決める（ホスト側の所有/グループ調整、共有グループ、もしくは作業ツリーをコンテナ内に閉じる等）。
- `.env` 等の secrets は作業ツリー（AI が触るディレクトリ）に置かないことが最も確実。
  - シンボリックリンクで secrets を指す運用は、`chmod -R` 等の誤操作や参照境界が複雑化しやすい点に注意する。

## 動作確認手順（実装後に追記）

- （実装後）`make test-core` で既存のコマンド実行系テストが通ること
- （実装後）Docker bind mount あり/なしで worker 実行が機能すること

## つまずきと対処（メモ）

- worker ユーザー指定が「ツールごと」になっていると抜け道が生じやすいので、spawn 直前の共通箇所に集約する。
- worker の `HOME` / `CODEX_HOME` を invoker と混ぜると、意図せずトークン/キャッシュが共有される。

## 関連ファイル（実装時に追記）

- `CUSTOM.md`
- （実装予定）`codex-rs/core/...` のコマンド spawn 経路
