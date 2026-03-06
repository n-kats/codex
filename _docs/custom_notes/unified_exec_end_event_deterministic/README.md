# unified_exec: write_stdin 終了検知時に ExecCommandEnd を確実に emit する

## 目的

- `unified_exec` のテスト（およびイベント消費側）で、`ExecCommandBegin`/`ExecCommandEnd` のペアが確実に観測できるようにする。
- 具体的には、`TaskComplete` が先に届いて `ExecCommandEnd` が観測できない（0件になる）レースを潰す。

## 症状

次のテストが、環境やタイミングによって間欠的に失敗することがある。

- `codex-rs/core/tests/suite/unified_exec.rs` の `suite::unified_exec::unified_exec_emits_one_begin_and_one_end_event`
  - 失敗例: `expected end event for the write_stdin call`（`end_events.len()` が 0）

## 原因

- 長命コマンド（`exec_command`）の終了は、バックグラウンドの watcher タスク（`spawn_exit_watcher`）が PTY の終了を待って `ExecCommandEnd` を emit する設計だった。
- 一方でテストは「`TaskComplete` までに end が 1 件入っている」ことを前提にしている。
- `write_stdin` の呼び出しでプロセス終了が確定しても、watcher の emit は非同期なので、`TaskComplete` が先に到達してしまうと end が観測できず 0 件になりうる。

## 変更内容（何がどう変わるか）

- `write_stdin` が「セッションが終了した」ことを検知した場合、その場で `ExecCommandEnd` を emit する。
- watcher 側も引き続き動作するが、二重 emit を防ぐために `AtomicBool`（`end_emitted`）でガードする。
- end emit の前に、必要に応じて `output_drained` を短時間待つ（PTY 終了直後の trailing output を取り逃がしにくくする）。

## 対象範囲 / 非対象

- 対象: unified exec のセッション管理（`exec_command` + `write_stdin`）と、終了イベント emit のタイミング。
- 非対象: それ以外の exec 経路（`shell_command` / 非 unified exec の `exec_command` 等）。

## 注意点（環境差・既知の制約）

- end emit が `write_stdin` 側に寄るため、watcher だけに依存していたときよりイベント順序が安定する。
- 二重 emit 防止はプロセス ID 単位のフラグで行う（同一セッションで end を 2 回出さない）。

## 動作確認

- 失敗していたテストの再実行（例）:
  - `cd codex-rs && cargo test -p codex-core --test all suite::unified_exec::unified_exec_emits_one_begin_and_one_end_event`
- いつもの確認:
  - `make almost`

## つまずきと対処

- つまずき: watcher の emit は非同期なので、`TaskComplete` の方が先に観測されることがある（テストが `TaskComplete` でループを抜ける）。
- 対処: `write_stdin` の「Exited 検知」分岐で end を同期的に emit し、watcher とは `end_emitted` で協調する。

## 関連ファイル

- `codex-rs/core/src/unified_exec/mod.rs`
  - `SessionEntry` に `cwd/started_at/transcript/end_emitted` を保持
- `codex-rs/core/src/unified_exec/session_manager.rs`
  - `write_stdin` の Exited 分岐で `ExecCommandEnd` を emit
- `codex-rs/core/src/unified_exec/async_watcher.rs`
  - watcher 側も `end_emitted` を見て二重 emit を回避
- `codex-rs/core/tests/suite/unified_exec.rs`
  - 期待仕様（begin 1 / end 1 の観測タイミング）の根拠

