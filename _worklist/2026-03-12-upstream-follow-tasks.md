# 上流追従タスクリスト（`fork-origin/main` → `custom`）

作成日: 2026-03-12  
対象: `custom` ブランチ（フォーク）を `fork-origin/main` に追従させるための作業項目整理。

> このファイルは一時的なタスクリスト。恒久的な方針は `CUSTOM.md` に書く。  
> `git range-diff` の出力ログは `_tmp/range-diff/` に保存する（必要な場合）。

## 0. 前提・ゴール定義（今回の方針）

- [x] 方針: **カスタム機能が維持される範囲で、追従コストを極力低減する**（= 可能な限り上流のアーキテクチャへ寄せる）
- [x] 追従対象の上流 tip を固定する（`fork-origin/main`）
  - `cec211cabc158532459b0c522a0cf855a891bd40`（2026-03-12 時点）
  - 件名: `render local file links from target paths (#13857)`
- [x] 今回の追従スコープを固定する
  - 最優先: `codex-rs/exec/src/lib.rs` を上流の app-server client ルートへ寄せて差分縮小
  - 次点: `codex-rs/core/src/spawn.rs` を fork 要件（run_as 等）を保ったまま構造を上流寄りに整理
  - 対象外（今回）: app-server protocol 自体の新規 API 追加（必要が出たら別チケット化）
- [x] 追従完了の合格条件を固定する
  - [x] 影響crateのテストが通る（手元環境）
    - メモ: `make all` が PASS（2026-03-12）。ログ: `_tmp/all_test_result.txt`
  - [ ] 必要な生成物（schema 等）が更新されている
  - [ ] range-diff で「意図しない差分」が残っていない
  - [ ] fork 差分の理由が説明できる（ファイル単位）

## 1. “維持すべきカスタム” のチェックリスト（追従のガードレール）

上流へ寄せる作業をする前に、**壊してはいけない fork 要件**を明文化しておく。

- [ ] `--codex-home`（`CODEX_HOME` を起動直後に切替）: `_docs/custom_notes/codex_home_cli_flag/README.md`
  - 検証: `make verify-codex-home-cli-flag`
- [ ] コマンド実行を worker ユーザーに固定（`custom.exec.*`）: `_docs/custom_notes/command_exec_worker_user/README.md`
  - 検証: `make verify-command-exec-worker-user`
- [ ] `exec_command` の login / dotfiles 制御（`CODEX_SHELL_STARTUP_FILES`）: `_docs/custom_notes/exec_command_default_login/README.md`
  - 検証: `make verify-exec-command-default-login`
- [ ] `!` とモデル起動コマンドの env policy 分離: `_docs/custom_notes/user_shell_environment_policy_split/README.md`
  - 検証: `cargo test -p codex-core --lib config::custom_user_shell_environment_policy_overrides_user_shell_env`
- [ ] unified_exec の `ExecCommandEnd` 確実化: `_docs/custom_notes/unified_exec_end_event_deterministic/README.md`
  - 検証: `cargo test -p codex-core --test all suite::unified_exec::unified_exec_emits_one_begin_and_one_end_event`
- [ ] （必要なら）TUI の送信挙動（Enter vs Ctrl+Enter）: `_docs/custom_notes/tui-enter-newline-ctrl-enter-send/`
  - 検証: `cargo test -p codex-tui`（snapshot含む）

## 2. 差分の棚卸し（分類）

- [ ] 差分一覧を取る（大きい順に把握）
  - 例: `git diff --stat fork-origin/main..custom`
- [ ] 差分を3分類してメモする
  - **純追従**: 上流の変更をそのまま取り込める
  - **fork要件**: 上流に寄せると要件を満たせない（差分維持が必要）
  - **一時/作業用**: rebaseログや検証用など（必要なら整理/移動/削除）
- [ ] 次に効く順（差分量/追従コストの観点で優先度付け）
  - **最優先（差分が最大）**: `codex-rs/core/src/config/mod.rs`
    - ねらい: “fork要件” の設定項目を **局所化**して、上流の `ConfigToml`/ローダー/デフォルト導線へ寄せる
    - 具体案:
      - fork独自の設定キーは `config/custom.rs` のような **別モジュール**へ寄せ、`mod.rs` 側は上流の構造を保つ
      - 既存の追加項目が「上流の override 機構（CLI override / loader override）で表現できる」なら、その表現へ移す
    - 合格条件:
      - `git diff --numstat fork-origin/main -- codex-rs/core/src/config/mod.rs` が目に見えて縮む
      - 追加設定の説明が `_docs/custom_notes/` とコードの対応で追える
    - 実施メモ（2026-03-12）:
      - `codex-rs/core/src/config/custom.rs` を追加し、fork固有の `[custom]` 型/解決ロジックを分離
      - `codex-rs/core/src/config/mod.rs` のテスト定義を削除し、upstream と同じ `#[path = "config_tests.rs"] mod tests;` に寄せた
      - 差分確認: `git diff --numstat fork-origin/main -- codex-rs/core/src/config/mod.rs` → `+203/-52`（作業前は `+4588/-55`）
      - ポリシー: `deny_unknown_fields` により、未知のキーはロード時にエラー（Legacy 互換は入れない）
  - **次点（中〜大）**: `codex-rs/core/src/unified_exec/process_manager.rs`
    - ねらい: unified exec のコアロジックを upstream パターンへ寄せ、fork独自は拡張点で閉じる
    - 合格条件: range-diff で “毎回コンフリクトする塊” が減る（関数単位で一致が増える）
    - 実施メモ（2026-03-12）:
      - fork固有の `run_as` / sudo / argv0 / PTY 周りを `codex-rs/core/src/unified_exec/custom.rs` へ分離
      - `codex-rs/core/src/unified_exec/process_manager.rs` は upstream 骨格へ戻し、`env.run_as.is_some()` の場合のみカスタム経路へ分岐
      - 差分確認: `git diff --numstat fork-origin/main -- codex-rs/core/src/unified_exec/process_manager.rs` → `+7/-0`
  - **次点（差分は中だが衝突しやすい）**: `codex-rs/core/src/spawn.rs`
    - ねらい: `run_as`/sudo/PTY 等の fork要件を保ったまま、upstream の `spawn_child_async()` 骨格へ近づける
    - 制約: `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` / `CODEX_SANDBOX_ENV_VAR` に関わる行は不用意に変更しない（差分縮小のための移動でも注意）
    - 合格条件: 構造差（分岐の深さ・責務の混在）が減り、上流変更の取り込み時に迷う箇所が減る
    - 次に効く理由:
      - `spawn.rs` は上流側の変更頻度が高く、fork の `run_as` 要件が差分全体に波及しやすい（追従コストが積み上がりやすい）
      - unified_exec と同じく「upstream 骨格 + fork局所化」に寄せられる余地が大きい
    - 実施メモ（2026-03-12）:
      - `codex-rs/core/src/spawn.rs` を upstream に近い骨格へ戻し、fork固有の `run_as` / sudo / argv0 互換は `codex-rs/core/src/spawn/run_as.rs` に分離
      - `spawn_child_async()` は upstream 同形に維持し、`run_as.is_some()` の場合のみ `spawn_child_async_with_run_as()` を使う
      - 差分確認: `git diff --numstat fork-origin/main -- codex-rs/core/src/spawn.rs` → `+28/-1`（fork差分はサブモジュール側へ局所化）
- [ ] “差分が大きい” ファイルを優先して理由と方針を決める（例）
  - [ ] `codex-rs/exec/src/lib.rs`: 上流アーキテクチャへ寄せて差分縮小（最優先）
  - [ ] `codex-rs/core/src/spawn.rs`: fork要件（run_as 等）を残しつつ差分縮小
  - [ ] `codex-rs/core/src/codex.rs`: 追従での移行（telemetry/sandbox表現変更）を取り込む

## 3. 最重要: `codex-rs/exec` の実行モデル（追従コストの支配項）

上流は `codex-app-server-client`（in-process app-server）に寄せているが、fork は `codex-core` 直結（`ThreadManager` / `CodexThread`）の構造差がある。

- [x] 方針: **A: 上流に寄せる（追従コスト低減のため）**
- [ ] B（core直維持）は、Aがカスタム要件で成立しない場合のみのフォールバックとする

### 3A. `codex-rs/exec` を上流に寄せる（詳細タスク）

- [ ] 既存 fork 実装の責務を分解して、上流の app-server RPC にマッピング表を作る（このファイルに追記してよい）
  - `thread/start` / `thread/resume`
  - `turn/start` / `turn/interrupt`
  - `review/start`
  - approval / elicitation / tool request user input の流れ
- [ ] “上流の exec ルート” を手元で把握する（上流の `codex-rs/exec/src/lib.rs` を読む）
  - 目的: 置換対象のイベントループ/リクエスト送受信/エラー処理の骨格を理解する
- [x] “上流の exec ルート” を手元で把握する（上流の `codex-rs/exec/src/lib.rs`）
  - 実施: in-process app-server client（typed RPC + legacy bridge）のイベントループに寄せる方針を確定
- [x] `codex-app-server-client` に乗せ替える（追従コスト低減の本丸）
  - [x] `InProcessAppServerClient::start(...)` の起動・shutdownパスへ移行
  - [x] request 送信を `ClientRequest::*` に寄せる（`thread/start|resume`, `turn/start|interrupt`, `review/start`）
  - [x] event 受信を `InProcessServerEvent::*` に寄せる（server request/notification/legacy）
  - [x] 既存 fork の event processor（human/jsonl）を接続（stdout/JSONL 制約は維持）
  - 実装メモ:
    - `codex-rs/exec/src/lib.rs`: `run_exec_session()` を app-server ループへ置換
    - `codex-rs/exec/src/lib.rs`: `run_main()` を upstream 互換シグネチャに戻し、`run_main_with_agents_md()` を追加
    - `codex-rs/exec/src/main.rs` / `codex-rs/cli/src/main.rs`: 新シグネチャに合わせて呼び出しを更新
- [ ] “カスタム要件” を落とさずに統合する（壊れやすい順）
  - [ ] `--codex-home`（起動直後反映）が維持される（起動順序が変わりやすい）
  - [ ] stdoutポリシー（stdout は最終メッセージのみ、その他は stderr）を維持
  - [ ] `--json`（JSONL）出力互換（イベント順・フィールド）を確認
  - [ ] `custom.exec.*`（worker user）/ `ShellEnvironmentPolicy` の適用が維持される
  - [ ] `exec_command` の login/dotfiles 制御が維持される
- [ ] 合格条件（差分縮小の見える化）
  - [ ] `git diff --stat fork-origin/main..custom -- codex-rs/exec/src/lib.rs` が大幅に縮む
  - [ ] fork固有ロジックが “局所” に閉じている（ファイル全体に散らばらない）

### 3B. フォールバック: `codex-rs/exec` を core直のまま維持（Aが破綻した場合のみ）

- [ ] “上流追従不可の理由” を文章化する（なぜ app-server ルートに寄せない/寄せられないか）
- [ ] 追従ルールを決める（上流の変更をどう取り込むか）
  - 例: `codex-exec` に入った上流修正のうち、UI/出力/telemetry だけは追従する、など
- [ ] 合格条件: range-diff の `!` が fork要件として説明可能であること（理由が追えること）

## 4. `spawn/exec/sandbox`（fork要件が出やすい領域）

- [ ] `codex-rs/core/src/spawn.rs`（とその周辺）で “上流意図” と “fork要件” が両立しているか点検
  - `sandbox_policy` 判定（例: ネットワーク可否）
  - `run_as`（uid/gid / user名解決 / sudo経由 / PTY）
  - env clear / envs の順序と副作用
- [ ] 「上流に寄せられる部分」と「fork固有の拡張点」を分離して、差分縮小の余地を洗う
- [ ] 追従作業の注意: `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` / `CODEX_SANDBOX_ENV_VAR` 関連のコードは不用意に変更しない（差分が出る場合は “上流追従の結果” か “fork要件” かを明記する）

### 現状メモ（2026-03-12）

- `codex-rs/core/src/spawn.rs` は upstream の `spawn_child_async()`（NetworkSandboxPolicy ベース）に対し、fork 側で `run_as` / `sudo` / `arg0` 互換などの要件を抱えており、**構造差が大きい**。
- `CODEX_SANDBOX_*` 環境変数まわりは “触らない” ルールがあるため、差分縮小のための大規模な並べ替え・抽出は別作業として切り出す（このファイルの当該定義・設定箇所に差分を出さない前提で進める）。
- 次の方針（候補）:
  - `run_as`/sudo 部分を別ファイルへ抽出し、`spawn_child_async()` 本体は upstream に近い骨格へ寄せる（ただし `CODEX_SANDBOX_*` 周辺は差分を出さない）。
  - 追従コストが支配的なのは `exec` 側だったため、今回の優先度は `exec` を先に解消（完了）し、`spawn` は別チケット化して段階的に寄せる。

## 5. schema / docs / 生成物（追従で壊れやすいところ）

- [ ] `ConfigToml` や設定型を変更した場合: `just write-config-schema` を実行して更新を含める
- [ ] app-server protocol 形状が変わる場合: `just write-app-server-schema`（必要なら `--experimental`）を実行
- [ ] docs 更新が必要なら `docs/` と `_docs/custom_notes/` のどちらに書くべきか判断
  - 恒久ノウハウ: `_docs/custom_notes/{custom-name}/`
  - 一時タスク: `_worklist/`

## 6. 検証ゲート（手元環境で実行）

> この環境では `make` 実行は避ける（ログ上書き等の事故防止）。ここに書くのは “手元で実行する項目”。

- [x] `make all` が PASS（2026-03-12）
  - ログ: `_tmp/all_test_result.txt`
- [ ] 影響crate単位で `cargo test -p <crate>`（小さい単位から）
  - `codex-rs/exec` を触った: `cargo test -p codex-exec`
  - `codex-rs/cli` を触った: `cargo test -p codex-cli`
  - `codex-rs/core` を触った: `cargo test -p codex-core`
  - `codex-rs/tui` を触って UI/表示が変わった: `cargo test -p codex-tui` + snapshot確認
- [ ] カスタム維持の verify ターゲットを実行
  - `make verify-codex-home-cli-flag`
  - `make verify-command-exec-worker-user`
  - `make verify-exec-command-default-login`
- [ ] range-diff の最終チェック（`tmp-rebase` vs `custom`）
  - コマンド例: `git range-diff <old_base>..tmp-rebase <new_base>..custom`
  - 出力ログが必要なら `_tmp/range-diff/` に保存
  - メモ: `_tmp/range-diff/20260312-rebase-range-diff.fork-origin-main-cec211cab.custom-0bee91fd3.txt` を生成

## 7. 仕上げ（説明責任）

- [x] 「上流追従できない差分」について、各ファイルごとに理由を1〜3行で説明できる状態にする
  - 特に `codex-rs/exec/src/lib.rs` と `codex-rs/core/src/spawn.rs`
- [ ] rebaseログ（コンフリクト解消）があれば `_notes/deconflict/` に記録を残す

### 説明ドキュメント（作成）

- [x] `_worklist/2026-03-12-upstream-follow-summary.md` に、意図・差分理由・検証ログへの導線をまとめる
