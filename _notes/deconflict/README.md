# deconflict

`_notes/deconflict/` には、rebase 時のコンフリクト解消ログを置く。

## ルール

- `custom` を `fork-origin/main` に rebase するときは、解消前にこのディレクトリ内のログを最低 1 件は読む。
- コンフリクトを解消したら、その場でログを追記または新規作成する。
- 行番号は「解消時点の位置」でよい。後からずれても直し直さなくてよい。
- 本家を踏襲したのか、custom を維持したのか、手動で折衷したのかを必ず書く。

## ファイル名

- `YYYYMMDD-<topic>.md`
- 1 回の rebase で 1 ファイルにまとめても、衝突単位で分けてもよい。

## 最低限の記録項目

- `File`: 対象ファイル
- `Line`: 解消時の行番号
- `Resolution`: 上流優先 / custom 維持 / 手動マージ の別
- `Note`: 何をどう解消したか

## テンプレート

```md
# 20260303-rebase-custom.md

- File: `path/to/file.rs`
  - Line: 123
  - Resolution: upstream 優先
  - Note: 上流のシグネチャ変更を採用し、custom の追加引数は削除した

- File: `path/to/other.rs`
  - Line: 456
  - Resolution: 手動マージ
  - Note: 上流の分岐構造を維持しつつ、custom の環境変数分岐だけ残した
```
