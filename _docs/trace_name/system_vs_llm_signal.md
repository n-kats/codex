# system ログと LLM ログが混ざって読みにくい問題（現状の問題意識）

## 背景

Langfuse に OTEL で送っているデータが増え、以下が同じ trace/画面上で混在し始めた:

- “システム側” のログ（処理経路・どの関数/コンポーネントを通ったか、周辺イベント）
- “LLM 側” のログ（prompt / response / function calling など、LLM の挙動そのもの）

現状、LLM の In/Out を `llm_generation` に寄せたことで “見たいもの” は増えたが、
一方でシステム系の情報と混ざることで「読みにくさ」が課題になりうる。

## 問題意識

- LLM 挙動（In/Out）を追いたいとき、システム系のログがノイズになりがち
- システム側の異常（retry・timeout・tool 実行）を追いたいとき、LLM の巨大 payload がノイズになりがち
- “trace を 2つに分ける” と見やすくなる可能性がある一方で、相関が切れて原因追跡が難しくなる懸念もある

## 方向性（まだ決めない）

### A) trace を分けずに “レーン分け” する（第一候補）

- LLM は `llm_generation`（Langfuse generation observation）に集約
- システムは以下のいずれかで “フィルタ可能” にする
  - Langfuse/OTEL の metadata/tag を付与（例: `codex.component=system` / `codex.component=llm`）
  - system 系は OTEL logs 側に寄せて、traces には最小限だけ残す

メリット:
- 相関（この turn の LLM 呼び出し→tool など）が trace ツリーで保てる
- trace を横断して追う必要がない

懸念:
- Langfuse UI のフィルタ性（tag/metadata がどこまで実用になるか）に依存

### B) trace を 2つに分ける（第二候補）

- システム用 trace と LLM 用 trace を分離する

メリット:
- 画面上のノイズを根本的に減らせる可能性

懸念:
- 相関が切れる（ツール呼び出し/エラーと LLM 呼び出しを行き来する必要がある）
- Langfuse 側の “trace 間リンク” が弱い場合、運用が辛い

## 次のアクション（証拠取り）

- まず現状の Langfuse 画面で「何がノイズか」を具体例（スクショ/メモ）で残す
- フィルタ/tag が実用になるかを先に試し、ダメなら trace 分割を検討する
