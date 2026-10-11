# make fetch 後に custom と fork-origin/main の距離を表示する

## 目的

- `make fetch` の直後に、`custom` ブランチと `fork-origin/main` がどれだけ離れているかをすぐ確認できるようにする。
- rebase 前後の確認や、上流との差分把握を毎回手で打たなくて済むようにする。

## 変更内容

- `Makefile` の `fetch` ターゲット末尾に、`git rev-list --left-right --count custom...fork-origin/main` を使った表示を追加する。
- 表示は `custom` 側の ahead 件数と `fork-origin/main` 側の ahead 件数を分けて出す。
- `fork-origin` のタグ取得は `--force` を付け、既存ローカルタグとの衝突で `make fetch` が失敗しないようにする。

## 対象範囲

- `make fetch` 実行時のみ。
- Git リモートを更新した後に、ローカルの `custom` ブランチと `fork-origin/main` の差分件数を表示する。

## 注意点

- `custom` または `fork-origin/main` の ref が存在しない場合は表示しない。
- 表示は差分件数の確認だけで、fetch の挙動自体は変えない。

## 動作確認手順

- `git rev-list --left-right --count custom...fork-origin/main`
- `make fetch` を実行して、末尾に `custom vs fork-origin/main: ...` が出ることを確認する。
- タグ衝突が出ていた環境では、`git fetch fork-origin --prune --tags --force` で成功することも確認する。

## つまずきと対処

- `custom` と `fork-origin/main` のどちらかが存在しない場合:
  - 先に `git fetch fork-origin` を実行して ref を揃える。
- `git rev-list` の出力は左列が `custom`、右列が `fork-origin/main` の件数になる。

## 関連ファイル一覧

- `Makefile`
