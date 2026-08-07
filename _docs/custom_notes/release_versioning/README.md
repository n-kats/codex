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
- `codex` と `codex-code-mode-host` を release ビルドし、同じ tarball に同梱する
- release ビルド時に `codex-rs/Cargo.toml` の `[workspace.package].version` を一時的に `full_version` に差し替え、`env!("CARGO_PKG_VERSION")` に反映させる（ビルド後に復元）
- `_release/*_RELEASE.md` の Markdown インラインコードを壊さずに生成し、GitHub Releases へのアップロード案内や本文のコピペ出力は含めない

## 対象範囲

リリーススクリプトが生成する Linux 配布物と、その配布物に添付するリリースノートが対象。GitHub Release の作成・アップロード自体は対象外。

## 注意点

- 安定タグが無い、または履歴が浅くてタグを取得できない場合は release は失敗する。必要なら事前に `make fetch` を実行する。
- `x.y.z-custom-...` のような suffix を付けると、TUI の更新チェックはそのままだと比較不能になるため、この fork では suffix を含む current version も比較できるようにしている（詳細: `_docs/custom_notes/update_check_custom_version_suffix/README.md`）。
- リリースノートはシェルのヒアドキュメントで変数を展開するため、Markdown のバッククォートはエスケープして記述する。エスケープしないと、コマンド置換として実行されて内容が空になる。

## 動作確認手順

- 利用者環境で `make release` を実行する。
- 生成された tarball に `codex` と `codex-code-mode-host` が含まれることを確認する。
- `_release/*_RELEASE.md` を確認し、ファイル名・コードブロック・`LICENSE`/`NOTICE` が表示され、GitHub Releases 用セクションが存在しないことを確認する。
- 標準出力に生成したリリースノートが表示されること、および GitHub Releases 用のアップロード案内が表示されないことを確認する。

## つまずきと対処

- Markdown のコード表記が空になる場合は、`scripts/release_linux.sh` のヒアドキュメント内でバッククォートが `\`` の形になっているか確認する。

## 関連ファイル

- 実装: `scripts/release_linux.sh`
- 仕様メモ: `_docs/custom_notes/release_versioning/README.md`
