# linux_sandbox_bwrap_userns_disabled_skip

## 目的

CI/コンテナ環境で unprivileged user namespace（`CLONE_NEWUSER`）が禁止されている場合に、bubblewrap（bwrap）前提のテストが必ず落ちるため、`make almost`（= test-almost）で「前提が無いときはスキップ」できるようにする。

典型的な失敗:

- `bwrap: No permissions to create a new namespace ...`
- `kernel.unprivileged_userns_clone=1` の案内が出る

## 変更内容

- `should_skip_bwrap_tests()` の probe 実行が、存在しない `--require-bwrap` フラグを渡してしまい、bwrap 前提の可否判定が正しく動かないことがあった。
- probe から `--require-bwrap` を削除し、実際に bubblewrap デフォルト経路で 1 回実行して stderr を見て判定するようにした。

## 対象範囲

- `codex-rs/linux-sandbox/tests/suite/landlock.rs` の bwrap 系テストのみ（`should_skip_bwrap_tests()` を使うもの）。

## 注意点

- `/proc/sys/kernel/unprivileged_userns_clone` が `1` でも、コンテナランタイムや LSM により `CLONE_NEWUSER` が拒否されることがある。その場合は stderr の文言（`No permissions to create a new namespace` 等）でスキップ判定する。

## 動作確認手順

- `make almost CARGO_TEST_FLAGS=--no-fail-fast` を実行し、bwrap が使えない環境では該当テストが `skipping bwrap test:` を出してスキップされることを確認する。
- 逆に bwrap が使える環境ではスキップされず実行されることを確認する。

## 関連ファイル一覧

- `codex-rs/linux-sandbox/tests/suite/landlock.rs`

## 20260927: `/dev/fd` mask の bwrap setup 修正

今回の rebase 後に発生した `blocked_descriptor_path` の失敗は、user namespace が使えないためにテストを skip すべき問題ではなかった。MCP の exec-server 実行で、bwrap が `--dev /dev` によって作る `/dev/fd -> /proc/self/fd` の絶対 symlink を、sandbox の `/newroot` 構築中に `--tmpfs /dev/fd` で mask しようとして `No such file or directory` になっていた。

`codex-rs/linux-sandbox/src/bwrap.rs` では、procfs が必要な path mask の前に `--proc /proc` を配置し、`/dev/fd` とその配下の deny/mask 対象は実体の `/proc/self/fd` に写像する。これにより `mount_proc` の fallback を壊さず、descriptor path の隔離も維持できる。環境依存の skip や test list 追加は行わない。

確認:

- `codex-exec-server` の `shell_snapshot::shell_snapshot_concurrent_replays_keep_independent_readers::blocked_descriptor_path`: `1 passed`
- `codex-linux-sandbox` の bwrap 関連 filter: `44 passed`
- MCP `run_make_almost_equivalent`: fmt、linux-sandbox build、`test-almost` が全て exit code 0
