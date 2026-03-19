# make test-almost を途中で止めずに回す（`--no-fail-fast`）

## 目的

- `make almost` / `make test-almost` 実行時に、`cargo test` が 1 件目の失敗で停止してしまい「対象テストを最後まで走らせたい」ケースがある。
- `Makefile` 側で追加フラグを渡せるようにして、`--no-fail-fast` 等を簡単に指定できるようにする。

## 変更内容

- `Makefile` に `CARGO_TEST_FLAGS ?=` を追加し、`cargo test` 呼び出しに挿入できるようにした。
  - 例: `make test-almost CARGO_TEST_FLAGS=--no-fail-fast`

## 対象範囲

- `Makefile` の `test-core` / `test-all` / `test-almost` の `cargo test` 実行。

## 注意点

- `CARGO_TEST_FLAGS` は `cargo test` のフラグ領域（`--` より前）に入る。
  - そのため `-- --nocapture` のような “テストバイナリに渡す引数” をここに入れる用途には向かない。
- コマンド長制限は「テスト件数」ではなく、`--skip ...` を大量に列挙する等で発生し得る。
  - `--no-fail-fast` の追加だけで引数が膨らむことはほぼない。

## 動作確認手順

- `make test-almost CARGO_TEST_FLAGS=--no-fail-fast`
- `make almost CARGO_TEST_FLAGS=--no-fail-fast`

## つまずきと対処

- 「`make almost` が途中で止まる」場合:
  - `make` のターゲット連鎖ではなく、`cargo test` が fail-fast で止まっていることが多い。
  - `CARGO_TEST_FLAGS=--no-fail-fast` を付けて再実行する。

## 関連ファイル一覧

- `Makefile`
