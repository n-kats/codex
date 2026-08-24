# fast_non_release_build

## 現在の状態

- 高速ビルド用の `CARGO_PROFILE_{dev,test}_*` 上書きは無効化している。
- `make` と MCP は Cargo の通常プロファイルを使う。
- `CARGO_BUILD_JOBS` だけは Docker 内の Cargo へ渡す。既定値は `4`、メモリに余裕がある場合は `make CARGO_BUILD_JOBS=8 ...` で上書きできる。

## 対象範囲

- `Makefile` 経由で Docker 内実行される非 release ターゲット。
- `make release` は従来どおり release プロファイルを使う。
- `make fmt` はビルドを行わない。

## 注意点

- 通常プロファイルに戻したため、速度優先設定を使っていた場合より初回ビルドが長くなることがある。

## 動作確認手順

- `make test-core`

## 関連ファイル一覧

- `Makefile`
