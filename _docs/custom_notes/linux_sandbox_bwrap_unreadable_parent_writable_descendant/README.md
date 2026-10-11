# linux-sandbox(bwrap): unreadable 親配下の writable carveout を維持する

## 目的

- `bubblewrap(bwrap)` で、`unreadable (None)` なディレクトリ配下に `writable (Write)` なサブパスがある構成（例: `blocked/allowed`）を正しく表現する。
- 具体的には、親ディレクトリは一覧できない（中身は見えない）まま、明示的に許可した子ディレクトリは書き込みできる状態にする。

## 変更内容

- `--tmpfs <blocked>` で unreadable 親をマスクする際、writable 子の bind mount ターゲットを先に作る（`--dir <blocked/allowed>` など）。
- その後 `--remount-ro <blocked>` を行い、最後に `--bind <allowed> <allowed>` で writable 子を再バインドする順序にした。
  - `--remount-ro` が子マウントに影響し得るため、writable 子の bind は remount の後に行う。

## 対象範囲

- `codex-rs/linux-sandbox/src/bwrap.rs` のファイルシステム引数生成（split policy での unreadable carveout）。

## 注意点（環境差・既知の制約）

- bwrap の挙動やカーネルの mount/remount の挙動により、順序依存の失敗が出やすい領域。
- テスト失敗時に exit code しか出ないと原因切り分けが難しいため、統合テスト側で stderr を表示する改善も同時に入れている。

## 動作確認手順

- 対象テスト（例）:
  - `cargo test -p codex-linux-sandbox --test all suite::landlock::sandbox_reenables_writable_subpaths_under_unreadable_parents`
  - `make test-almost`

## つまずきと対処

- `sandbox_reenables_writable_subpaths_under_unreadable_parents` が `exit_code = 1` で落ちる場合:
  - stderr を確認し、`EROFS`（read-only filesystem）/ `EPERM`（権限）/ `No such file or directory`（mount target が無い）などの種別で原因を切り分ける。

## 関連ファイル一覧

- `codex-rs/linux-sandbox/src/bwrap.rs`
- `codex-rs/linux-sandbox/tests/suite/landlock.rs`
