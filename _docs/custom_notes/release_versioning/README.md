# Release versioning（custom）

## 目的

`make release` で作る配布物に、次の形式でバージョンを付与する:

- `x.y.z-custom-yyyy-mm-dd`

ここで `x.y.z` は upstream（`fork-origin/main`）の `rust-vx.y.z` タグから導出する。

## 変更内容

`scripts/release_linux.sh` で次を行う:

- `base_version`（`x.y.z`）を `fork-origin/main` から推定
  - 可能なら `git describe --match 'rust-v*' fork-origin/main` を使用
  - 取れない場合はローカルの `rust-vx.y.z` タグから最大を使用
- `full_version = {base_version}-custom-{yyyy-mm-dd}` を生成
- tarball 名を `custom_codex_{full_version}.tar.gz` にする
- release ビルド時に `codex-rs/Cargo.toml` の `[workspace.package].version` を一時的に `full_version` に差し替え、`env!("CARGO_PKG_VERSION")` に反映させる（ビルド後に復元）

## 注意点

- `fork-origin` のタグをローカルに反映していない場合、`git fetch fork-origin --tags` が必要になる。
- `x.y.z-custom-...` のような suffix を付けると、TUI の更新チェックはそのままだと比較不能になるため、この fork では suffix を含む current version も比較できるようにしている（詳細: `_docs/custom_notes/update_check_custom_version_suffix/README.md`）。

## 関連ファイル

- 実装: `scripts/release_linux.sh`
