# Update check: カスタム版バージョン文字列の比較

## 目的

カスタム配布で `CARGO_PKG_VERSION`（=`codex --version`）に suffix を付けると、TUI の「更新チェック」がバージョン比較できず、更新通知が出なくなる。

例:

- current: `1.2.3-custom-2026-01-25`
- latest: `1.2.4`

このとき `1.2.3-custom-...` を `1.2.3` として扱い、`latest > current` を判定できるようにする。

## 変更内容

`codex-rs/tui/src/updates.rs` のバージョン比較ロジックを、`latest` と `current` で別のパース戦略にした。

- `latest`:
  - `MAJOR.MINOR.PATCH`（数字3要素）だけ比較対象にする
  - `-`（prerelease）が含まれる場合は **比較不能**（従来どおり）
  - `+`（build metadata）は無視
- `current`:
  - `-` または `+` 以降を捨てて `MAJOR.MINOR.PATCH` を抽出する
- `1.2.3-custom-...` / `1.2.3+custom...` のようなカスタム suffix を許容する

実装では、`parse_strict_version()` と `parse_current_version()` を分け、
`latest` 側は strict、`current` 側は suffix 許容で比較する。

## 対象範囲

- TUI（release ビルド限定）での更新チェック表示
  - `codex-rs/tui/src/updates.rs` は `#![cfg(not(debug_assertions))]` なので、debug ビルドでは動作しない

## 注意点

- `latest` が `1.2.3-rc.1` のような prerelease の場合は、比較不能として扱い更新通知は出ない（従来の設計を維持）。
- `current` が suffix を含むときのみ比較許容するので、上流の挙動（prerelease を更新通知しない）を変えない。

## テスト

- 実装の単体テスト:
  - `codex-rs/tui/src/updates.rs` 内の `custom_suffix_versions_are_comparable_against_plain_semver`
  - `codex-rs/tui/src/updates.rs` 内の `prerelease_version_is_not_considered_newer`
- 手元の検証（ターゲット）: `make test-tui`

## 関連ファイル

- 実装: `codex-rs/tui/src/updates.rs`
- UI: `codex-rs/tui/src/update_prompt.rs`
