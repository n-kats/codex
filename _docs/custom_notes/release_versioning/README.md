# Release versioning（custom）

## 目的

`make release` で作る配布物に、次の形式でバージョンを付与する:

- `x.y.z-custom-yyyy-mm-dd`

ここで `x.y.z` は upstream 側の **安定タグ** `rust-vx.y.z` から導出する。

## 変更内容

`scripts/release_linux.sh` で次を行う:

- `base_version`（`x.y.z`）をローカルにある安定 `rust-vX.Y.Z` タグから導出する
  - 最新の安定タグを採用する（ブランチ探索はしない）
  - `rust-v0.0.*` のような snapshot 系タグや `-alpha/-beta` 等は採用しない
  - 安定タグが導出できない場合は **エラーで停止**（ローカルタグへのフォールバックはしない）
- `full_version = {base_version}-custom-{yyyy-mm-dd}` を生成
- tarball 名を `custom_codex_{full_version}.tar.gz` にする
- release ビルド時に `codex-rs/Cargo.toml` の `[workspace.package].version` を一時的に `full_version` に差し替え、`env!("CARGO_PKG_VERSION")` に反映させる（ビルド後に復元）

## 注意点

- 安定タグが無い、または履歴が浅くてタグを取得できない場合は release は失敗する。必要なら事前に `make fetch` を実行する。
- `x.y.z-custom-...` のような suffix を付けると、TUI の更新チェックはそのままだと比較不能になるため、この fork では suffix を含む current version も比較できるようにしている（詳細: `_docs/custom_notes/update_check_custom_version_suffix/README.md`）。

## 関連ファイル

- 実装: `scripts/release_linux.sh`
