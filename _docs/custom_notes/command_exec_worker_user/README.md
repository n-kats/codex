# コマンド実行を worker ユーザー（assistant 等）に固定する方針

## 目的

- AI によるコマンド実行（shell / exec_command 等）を低権限ユーザーに固定し、秘匿情報やユーザー環境（ログイン状態・トークン等）へのアクセスを抑える。
- 「危ないコマンドの denylist」ではなく、OS の権限境界（uid/gid）を一次防衛線にする。

## 背景（問題）

- `.env` や各種 secrets が作業ディレクトリやホスト環境に存在する場合、`cat` / `env` / アプリのログ出力などを通じて意図せず漏えいする可能性がある。
- OpenAI アカウントログイン等により作成される非環境変数のトークン（`CODEX_HOME` / `HOME` 配下に保存される情報）が、コマンド実行経路から読めてしまうと危険度が上がる。
- Docker を使う/使わないに関わらず、ユーザー分離ができていないと「うっかり」で境界が崩れやすい。

## 変更内容（方針）

実装時の基本方針は次のとおり。

- Codex 本体（LLM 通信・ログイン状態保持）は従来どおり起動ユーザー（invoker）の権限で動かす。
- コマンド実行系ツール（例: `shell`, `shell_command`, `exec_command` 等）で spawn されるプロセスは、設定で指定した worker ユーザー（例: `assistant`）で実行する（worker_only）。
- これにより、worker から invoker の `HOME` / `CODEX_HOME`（ログイン状態・キャッシュ）へアクセスしない構造を作る。

このノートでは「worker_only」までを対象とし、権限不足時に invoker で再実行する（ask_to_escalate）等は将来の拡張点として扱う。

## 今回のスコープ外

- `ask_to_escalate` は、worker（低権限）での実行に固定せず、以下のようなケースで「起動ユーザー（invoker）の権限で実行しますか？」を対話的に確認する挙動を指す。
  - 事前判定で、権限境界を跨ぐことが明らか（例: `sudo` / `su` / `doas` を含む）
  - 事前判定で、システム領域への書き込み等が明らか（例: `/etc`, `/usr`, `/var` 配下の変更）
  - 事後判定で、worker 実行が `EACCES` / `EPERM`（Permission denied / Operation not permitted）等で失敗した
- 今回実装しない理由:
  - `ask_to_escalate` は「昇格を伴う別ツール（別経路）」として提供する想定であり、このカスタム（worker_only の実装）とは別スコープで進める。
  - 本ノートは「コマンド実行を常に worker に固定する」方針と設定スキーマの整理に限定する。
- その代わり、今回のスコープでは `worker_only`（コマンド実行は常に worker に固定）に限定し、権限が必要な操作は「人間が手動で実行する」前提で運用する。

## 設定（案）

上流との衝突を避けるため、設定は `custom.exec.*` 名前空間に追加する。

- `[custom.exec]`
  - `worker_user = "assistant"`（任意）
  - `worker_uid = 1001`（任意）
  - `worker_gid = 1001`（任意）

### 解決ルール（コンフリクトはエラー）

- 未指定の場合: 従来挙動（起動ユーザーのまま実行）
- `worker_uid` と `worker_gid` は片方だけの指定を許可しない（欠けていればエラー）
- `worker_user` と `worker_uid/gid` が両方指定されている場合:
  - `worker_user` を解決した uid/gid と `worker_uid/gid` が一致していれば OK
  - 一致していなければエラー（曖昧さは許容しない）

## 対象範囲 / 非対象

- 対象: モデルが起動するコマンド実行系ツール経路（shell / shell_command 等の「子プロセス spawn」）
- 非対象: ユーザーが明示的に実行する `!`（user_shell）などの “ユーザー起点” のコマンド実行
- 非対象: Codex 本体の API 呼び出し、ログイン状態の保持、モデル通信（invoker 側に残す）

## 注意点（Docker / ホスト）

- Docker の bind mount では、ホスト側ファイルの所有 UID/GID がコンテナ内でも見えるため、worker の uid/gid が合わないと `/workspace` に書けないことがある。
  - 対策は運用で決める（ホスト側の所有/グループ調整、共有グループ、もしくは作業ツリーをコンテナ内に閉じる等）。
- `custom.exec.*` が「起動ユーザー（invoker）と同じ uid」に解決されている場合、権限分離は実質的に無効になる。
  - この場合は起動時に warning を出す（`!` は常に invoker で実行されるため、モデル起動コマンドも同じユーザーだと境界がない）。
- `custom.exec.worker_user` を使う場合、実行時には worker ユーザーの supplementary groups（補助グループ）も child process に設定する。
  - そのため「worker を共有グループに追加して `chmod 710` で“通過だけ”許可する」といった運用が成立する。
  - 一方、`custom.exec.worker_uid/gid` だけでユーザー名が分からない場合は supplementary groups を解決できない（= 設定しない）ため、必要なら primary GID を共有グループに合わせる（`worker_gid`）か、パス側の権限を運用で調整する。
- ホスト（非 root）で `custom.exec.worker_user` を使う場合、Codex はまず child process 側で `setgroups/setgid/setuid` を試し、これが許可されない環境では `sudo -n -u "#UID" -g "#GID" -- env -i ...` にフォールバックする。
  - `codex` 実体バイナリに file capability を付与（例: `setcap cap_setuid,cap_setgid=ep $(which codex)`）
  - systemd の `AmbientCapabilities=` 等で起動時に capabilities を付与（ファイルに `setcap` したくない場合）
  - `setcap` が使えない環境では、`sudoers` の許可でパスワードなし `sudo` を使えると worker-user 実行を維持できる。
  - root で起動する（推奨しない）
- `.env` 等の secrets は作業ツリー（AI が触るディレクトリ）に置かないことが最も確実。
  - シンボリックリンクで secrets を指す運用は、`chmod -R` 等の誤操作や参照境界が複雑化しやすい点に注意する。
- `custom.exec.*` を有効にしても、コマンド実行時に渡される環境変数自体が多いと `env` / `printenv` 等で情報が出る。
  - そのため本 fork では、`custom.exec.*` が設定されている場合に `shell_environment_policy.inherit = "all"` をエラーにする（安全のため）。
  - 推奨: `shell_environment_policy.inherit = "core"`（または `"none"`）にして、必要なら `include_only` で許可リスト運用にする。

## 動作確認手順（実装後に追記）

- `make verify-command-exec-worker-user` で設定解決（`custom.exec.*`）のテストが通ること
- `make test-core` で既存のコマンド実行系テストが通ること
- `custom.exec.worker_uid/gid` を設定して、`shell` / `shell_command` / `exec_command`（unified exec）で spawn が worker UID/GID になること
  - 例: `id -u` / `id -g` を実行して期待値になること
- `exec_command` は worker user 指定時に child process の `uid/gid` を落として実行し、`setuid/setgid` が許可されない環境では `sudo` にフォールバックする。
  - 起動直後（turn 作成時）に `id -u` / `id -g` で worker の解決結果を確認し、満たせない場合は早めに Warning を出す（後から `exec_command` で落ちるのを避ける）
- Docker bind mount あり/なしで worker 実行が機能すること（必要なら UID/GID をホスト側に合わせる）

## 設定例（推奨）

```toml
[custom.exec]
worker_user = "assistant"

[shell_environment_policy]
inherit = "core"
ignore_default_excludes = false
experimental_use_profile = false
include_only = [
  "HOME", "LOGNAME", "USER", "USERNAME",
  "PATH", "SHELL",
  "TMPDIR", "TEMP", "TMP",
  "LANG", "LC_*",
  "TERM", "COLORTERM",
]
```

## つまずきと対処（メモ）

- worker ユーザー指定が「ツールごと」になっていると抜け道が生じやすいので、spawn 直前の共通箇所に集約する。
- worker の `HOME` / `CODEX_HOME` を invoker と混ぜると、意図せずトークン/キャッシュが共有される。
- `codex-rs/core/src/spawn.rs` と `codex-rs/utils/pty/src/pipe.rs` では、worker user 固有の処理を child process の `pre_exec` に寄せ、失敗時は `sudo` フォールバックへ切り替える形にしている。

## 現在の実装メモ

- `custom.exec.worker_user` / `worker_uid` / `worker_gid` は `core/src/config/mod.rs` で解決し、`Config::custom_exec_run_as()` として各 runtime に渡している。
- `core/src/spawn.rs` の `SpawnChildRequest` に `run_as` を追加し、`setgroups` → `setgid` → `setuid` を spawn 直前で適用している。
- `shell` / `shell_command` / `exec_command` / `unified_exec` の各経路で `run_as` を埋めるようにしている。
- `shell_environment_policy.inherit = "all"` と `custom.exec` の併用は、`custom.exec` が設定された状態での env 漏えいを防ぐために `InvalidInput` にしている。
- `custom.exec.worker_user` の supplementary groups は Unix で `getgrouplist` から解決し、`worker_user` が現在のログインユーザーでも実行ユーザーとしてのグループ境界を保つようにしている。
- 起動時には `custom.exec.*` が invoker と同じユーザーへ解決される場合に warning を出す。
- 回帰テストとして、`core/src/config/config_tests.rs` に warning 解決テストを追加し、`core/tests/suite/custom_exec_command_worker_user.rs` に `exec_command` / `shell` が worker uid/gid で動く E2E テストを追加した。
- さらに `core/tests/suite/user_shell_cmd.rs` で、`!` が worker user の設定に影響されず invoker 側のまま動くことを確認している。

## 関連ファイル（実装時に追記）

- `CUSTOM.md`
- `Makefile`
- `docs/config.md`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/spawn.rs`
- `codex-rs/core/src/sandboxing/mod.rs`
- `codex-rs/core/src/tools/runtimes/shell.rs`
- `codex-rs/core/src/tools/runtimes/unified_exec.rs`
- `codex-rs/core/src/tasks/user_shell.rs`
- `codex-rs/core/src/exec.rs`
- `codex-rs/core/src/landlock.rs`
- `codex-rs/core/src/config/config_tests.rs`
