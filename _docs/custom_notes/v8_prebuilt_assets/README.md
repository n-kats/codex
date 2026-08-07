# V8 prebuilt assets

## 目的

Linux の通常ビルドで `V8_FROM_SOURCE=1` を使わず、Codex が公開する `rusty_v8` の sandbox artifact を再利用して V8 のビルド時間を短縮する。

## 変更内容

- `scripts/downloads_assets.sh` が `v8` の解決バージョンとホスト target に対応する archive、binding、checksum を Codex release から `_tmp/assets` に取得する。
- `Makefile` の `make download-assets` でダウンロードできる。
- archive と binding が `_tmp/assets` に揃っている場合、通常の Cargo build/test 用環境に `RUSTY_V8_ARCHIVE` と `RUSTY_V8_SRC_BINDING_PATH` を自動設定する。

## 対象範囲

- 対象: `ptrcomp_sandbox_release`、現在の host target、Cargo の `v8` 依存を使う通常ビルド。
- 非対象: `V8_FROM_SOURCE=1` を明示したビルド、Codex release artifact が公開されていない target、Bazel の V8 入力。

## 注意点

- archive と binding は同じ `v8` バージョン・target のペアでなければならない。
- `make download-assets` とビルドは別々に実行する。Makefile の artifact 判定は make 起動時に行われる。
- Docker 経由のビルドでは、ホスト側の絶対パスを Cargo に渡さず、コンテナ内の `/workspace/_tmp/assets` を参照する。
- `_tmp/assets` は生成物であり、Git 管理対象にしない。

## 動作確認手順

```sh
make download-assets
make almost
```

ダウンロード後、Makefile が次の2変数を Cargo に渡すことを確認する。

- `RUSTY_V8_ARCHIVE`
- `RUSTY_V8_SRC_BINDING_PATH`

## つまずきと対処

- `denoland/rusty_v8` の upstream URL が 404 になる場合は、Codex 側の `rusty-v8-v<version>` release artifact を使う。
- target が `x86_64-unknown-linux-gnu` 以外の場合は、公開済みの同じ target artifact が必要になる。

## 関連ファイル

- `scripts/downloads_assets.sh`
- `Makefile`
- `third_party/v8/README.md`
