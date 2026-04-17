# Rebase Prep: Current Issues Before `fork-origin/main` Rebase

## 背景

`custom` ブランチは標準 TUI の `app-server` 化と remote surface の整備を進めているが、`fork-origin/main` との差分が大きくなっている。
rebase 前に、現状の不具合・未収束点を整理しておく。

## 主要な問題

### 1. config 読み込み/反映が不安定

- `--config` / `loader_overrides` の経路は修正しているが、
  本家の config フローへ差分が十分に乗っていない可能性がある。
- `custom.user_shell.no_inject` や `custom.exec.worker_user` が、
  実際の実行経路まで end-to-end で反映されていない疑いがある。
- 起動時/リロード時/onboarding 後の config 再構成で、本家の loader flow に合わせる必要がある。

### 2. user shell の `no_inject` が期待通りに効いていない

- `!pwd` を打っても後続 UI/履歴で `pwd` として扱われるように見える。
- `custom.user_shell.no_inject = true` の設定が、
  - 履歴保存
  - model コンテキスト注入
  - 表示上のコマンド再現
  に正しく伝播しているか要確認。

### 3. worker-user 実行が反映されていない疑い

- `id` の結果が `uid=1000(ubuntu)` のままで、設定した worker-user が使われていないように見える。
- `custom.exec.worker_user` が shell/exec の実行本体へ届いているか、
  本家の Run-As 実装に寄せて再確認が必要。

### 4. `app-server` 化の hybrid がまだ残っている

- `tui/src/app.rs` は `AppServerSession` 側の bridge を増やしているが、
  まだ local `ThreadManager` 前提の残りがある。
- `ThreadStarted` / `ThreadStatusChanged` / `SkillsChanged` / request bridge は進めているが、
  本家の event loop と完全一致ではない。
- `Not available in TUI yet for thread ...` のような fallback メッセージは、
  upstream 追従時に残すべきか整理が必要。

### 5. `/new` / `/resume` / `/fork` と approval/history の整合性

- 一部の live thread lifecycle を app-server に寄せ戻しているが、
  approval response の thread lookup や history replay の整合性がまだ崩れうる。
- `Failed to find thread ... for approval response` のような不整合は、
  rebase 後に再発しやすいので、衝突解消時に優先チェックする。

## 今後の方針

- `bwrap` と `sudo` は競合ではなく別軸として扱う。
- upstream にある `bwrap` の sandbox 流れは優先して取り込む。
- custom の `sudo` / `run_as` は、spawn / sandbox の seam にだけ差し込む。
- つまり「どちらかを消す」のではなく、「同じ抽象に乗せて backend を分ける」方針で解消する。
- UI や config の上位で `sudo` / `bwrap` の違いを広げず、`core/src/spawn.rs` と `core/src/tools/runtimes/shell/unix_escalation.rs` のような実行境界で合成する。
- rebase では upstream 優先で衝突を解き、custom 差分は実行境界へ薄く戻す。

## rebase 全体方針

- まず upstream の責務分離を採用し、custom は seam にだけ差し込む。
- config / login / UI は upstream の流れを壊さず、custom 設定は正規化層で吸収する。
- shell / exec / sandbox は、`run_as` と `bwrap` を別軸として backend 合成する。
- hybrid を温存しない。`false &&` のような一時しのぎではなく、upstream 形に戻すか、custom を seam に寄せて再実装する。
- rebase の判断に迷ったら、`fork-origin/main` 側の責務配置を優先し、custom は最小差分で追随する。

### 詳細な判断基準

- `bwrap` は sandbox の責務として扱う。
  - filesystem / network / namespace の制御は sandbox 層で完結させる。
  - UI / login / config の上位に sandbox backend の詳細を漏らさない。
- `sudo` / `run_as` は実行ユーザーの責務として扱う。
  - spawn / exec の境界で user switch を適用する。
  - sandbox と同一の概念として扱わず、あとから合成する。
- upstream にある流れを壊すような custom 分岐は避ける。
  - まず upstream の責務配置を受け入れる。
  - custom はその seam にだけ追加する。
- 「どちらかを消す」ではなく、backend を分けて合成する。
  - 例: `sudo` で実行ユーザーを切り替えた上で `bwrap` を適用する。
  - 例外がある場合も、config で明示し、暗黙の hybrid にしない。
- rebase 中に一旦 `false &&` や no-op で逃がした箇所は、最終的に消す。
  - その場しのぎの条件分岐は rebase 後の保守性を悪化させる。
  - 迷ったら upstream の実装を採用し、custom は test で補う。

## rebase 時に優先して見るファイル

- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/app.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/app/app_server_notifications.rs`
- `codex-rs/tui/src/app/app_server_requests.rs`
- `codex-rs/tui/src/custom_config_loader_tests.rs`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/login.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
- `codex-rs/core/src/tools/runtimes/shell/unix_escalation_tests.rs`

## 次のアクション候補

1. `fork-origin/main` を取り込み、config / shell / worker-user の差分を upstream 基準で見直す。
2. `custom.user_shell.no_inject` と `custom.exec.worker_user` の end-to-end 経路を、rebase 後に再検証する。
3. hybrid の残りを減らしてから、`cargo check` と selected test を回す。
4. 変更点を `_docs/custom_notes/tui_remote_alignment/README.md` とこの worklist に追記する。
