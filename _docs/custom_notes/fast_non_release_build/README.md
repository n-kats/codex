# fast_non_release_build

## 目的

- `make test-*` / `make build` など（`make release` 以外）の反復を速くし、開発中の待ち時間を減らす。

## 変更内容（何がどう変わるか）

- `Makefile` で `CARGO_PROFILE_{dev,test}_*` を `export` し、非 release を速度優先にした。
- デフォルト値: `debug=1`, `codegen-units=16`, `lto=off`, `incremental=false`, `opt-level=0`
- `CARGO_BUILD_JOBS` をDocker内のCargoへ渡せる。既定値は `4`、メモリに余裕がある場合は `make CARGO_BUILD_JOBS=8 ...` で上書きできる。

## 対象範囲（非対象も）

- 対象:
  - `Makefile` 経由で Docker 内実行される「非 release」ターゲット。
- 非対象:
  - `make release`（リリースビルドの品質/再現性に影響させないため）
  - `make fmt`（ビルドを行わない）

## 注意点（環境差・既知の制約）

- `CARGO_PROFILE_{dev,test}_*` の上書きは、Cargo の `Cargo.toml` 設定より優先される。
  - デバッグが必要な場合は `make CARGO_FAST_BUILD=0 ...` や `make CARGO_FAST_DEBUG=2 ...` で戻す。
- 値は「手元での体感」を優先しているため、プロジェクト/環境によっては最適値が異なる。

## 動作確認手順（手動・テスト・スナップショット）

- デフォルト: `make test-core`
- 無効化: `make CARGO_FAST_BUILD=0 test-core`
- 調整: `make CARGO_FAST_DEBUG=2 test-core`

## つまずきと対処（警告や失敗の修正）

- 症状: デバッグに必要な情報が足りない（バックトレースやステップ実行がしづらい）
  - 対処: `make CARGO_FAST_DEBUG=2 ...` または `make CARGO_FAST_BUILD=0 ...`

## 関連ファイル一覧

- `Makefile`
