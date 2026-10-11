# 20260913-rebase-custom.md

## 対象

- upstream base: `fork-origin/main` / `b04a2c264516ec2e6b3c91dd73ad18a21fd5a88f`
- custom side: `tmp-rebase` / `e89959c320d60c53bc36b87baf3b751bc2500940`
- rebase state: `refs/heads/custom` に `custom changes` を適用中
- 競合数: 19 ファイル

## 競合解消

- `app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - upstream の実験 API／生成内容を基準にし、現行 protocol の
    `ThreadSettingsUpdateParams.projectDocPaths` だけを再適用した。生成済み fixture の
    実験 schema テストで確認した。
- `code-mode-host/tests/stdio.rs` と `exec-server/tests/exec_process.rs`
  - upstream の現行 session API、rich shell profile 検証、sandbox 条件を維持した。
    custom の `Text file busy` retry、sandbox 中に host へ cache marker を書かない処理だけを
   接合した。
- `code-mode-runtime/src/service.rs` と
  `core/src/tools/code_mode/execute_handler.rs`
  - upstream の現行 delegate／step-context API を採用し、MCP 待機ツールを含む code-mode
    だけ `u64::MAX` の完了待機へ切り替える custom 要件を残した。テスト専用処理は
    `custom_tests.rs` に分離し、runtime の待機モード計算は本体と共有した。
- `core/src/agents_md_manager.rs`、`codex_thread.rs`、`session/thread_settings.rs`、
  `config/mod.rs`
  - upstream の provider／cache／設定変換を戻さず、`project_doc_paths`、PSP、custom theme
    の設定経路だけを追加した。cache は選択環境・trust level に加えて明示 doc path の変更
    でも再読み込みする。
- `core/src/shell_snapshot.rs` と `shell_snapshot_tests.rs`
  - upstream の broker／sandbox／credential metadata を維持し、snapshot を保存する境界で
    exports を許可リスト化した。旧 `strip_snapshot_preamble` と旧 capture API は復元して
    いない。custom redaction tests は専用ファイルへ分離した。
- `tui/src/app/event_dispatch.rs`、`app/tests/safety_buffering.rs`、
  `chatwidget/slash_dispatch.rs`、`slash_command.rs`、`lib.rs`、`update_versions.rs`
  - upstream の現在の startup、Windows 警告、theme／実験 API、slash command 構成を優先
    した。`/custom-agents`、`custom.user_shell.no_inject`、差分 theme、custom version suffix
    のみを接合し、廃止された Personality 選択 UI と `SandboxReadRoot` 分岐は戻していない。
  - upstream の通常 test ファイルに残っていた custom-only test は `*_custom_tests.rs` へ
    分離し、`custom__...` 命名へ統一した。
- `core/src/config/config_loader_tests.rs`
  - upstream の loader fixture を採用し、custom commit が重ねていた
    `ignore_project_config`／`project_root_markers` の不要な fixture 変更を削除した。対応する
    custom loader／CLI テストは既存の専用 custom test 側にあるため、重複を残していない。
- `chatwidget/settings_popups.rs`
  - upstream が Friendly／Pragmatic 選択を廃止しているため、custom 側の古い人格選択 popup
    は復元せず削除した。現行 custom 方針にもこの UI の要件はない。

## 競合外の過去維持の削除

- `core/src/tools/router.rs`／`registry.rs` の caller がない runtime-cancellation hook と
  unused `dispatch_any` wrapper を削除した。MCP 完了待機に使う hook と async telemetry は
  維持した。
- `core/src/agents_md_tests.rs` に残っていた旧 `AgentsMdManager` API の custom test を
  `agents_md_custom_tests.rs` へ移し、現行 `SessionInstructions` と refresh の戻り値に合わせた。
- `custom.user_shell.no_inject` 未設定時は warning を出さない現行 custom 仕様に合わせ、
  upstream の通常 integration test に混入していた旧 warning filter／定数を除去した。明示的な
  `false` の warning、`true` の注入・履歴抑止は custom 実装と専用テストで維持している。
- `app-server/tests/suite/v2/thread_start.rs` では本家の Cloud 設定エラー test を削除せず、
  custom の Cloud 無効デフォルトではコンパイルしない `#[cfg(feature = "cloud")]` に接合した。
  コメントアウト済み import は feature 付き import に戻し、不要になった skip list の当該行だけ
  削除した。これで本家の all-features 検証を維持しつつ、custom の通常経路を壊さない。
- 旧 `codex mcp-server` command、孤立 test、現行依存に対応しない deny 例外、根拠のない
  feature override は前回監査どおり復元していない。

## 検証

- 内容上の conflict marker: 19 競合パスを検索して 0 件。
- MCP `cargo fmt --all -- --check`: 成功（stable rustfmt の
  `imports_granularity = Item` 警告のみ）。
- MCP selected tests: app-server protocol の experimental schema、core custom tests 11件、
  config custom tests 2件、code-mode-runtime custom test 1件、TUI custom tests 8件、
  CLI custom tests 12件、exec custom tests 4件、HTTP custom CA test 1件、
  core warning integration test 1件、app-server safety notification test 1件、
  TUI startup／worktree、exec-server shell snapshot の `local_pipe`、code-mode-host の
  shared process test が成功。
- 初回 MCP 実行では `glib-sys`／GStreamer の system library が不足して停止したが、
  Dockerfile 2つの Ubuntu 26.04 化と MCP 再起動後は GStreamer 1.28 系を含むビルドを通過した。
- MCP `make almost` 相当は実行済み。shell snapshot 21件、worktree、daemon の managed-install
  関連3件など、今回の接合部テストは成功した。全体実行は並列負荷により本家由来の
  WebSocket 接続リセット、shell snapshot の旧期待値、daemon 判定のタイムアウトが順に発生したが、
  期待値を現行 custom 仕様へ合わせた後の個別再実行は成功している。`make all`／全 feature の
  完全テストは未実行。ローカルの `make`／Docker は実行していない。
- 手順書指定の range-diff は `_tmp/range-diff/20260913-rebase-custom.txt` に保存した。ただし
  read-only index のため解消済み worktree を commit として比較できず、upstream HEAD と元の
  custom commit の差分表示に留まる。最終 rebase 後に metadata が書き込み可能な環境で再実行する。

## 20260914 の追加確認

- `exec/tests/suite/worktree.rs` の enterprise/cloud bundle 検証は `feature = "cloud"` のときだけ
  実行するようにした。custom の通常ビルドでは cloud loader が無効なため、クラウド用の合成認証で
  外部 ChatGPT endpoint へ進まず、ローカル trust 検証を維持する。本家の cloud feature 経路は残している。
- 上記 worktree テストの単独実行は成功したが、`make almost` 相当では次に
  `codex-voice-host` の `real_decoder_renders_current_rtp_and_rejects_pre_epoch_arrivals` が
  `jitter buffer unavailable` で失敗した。原因は Ubuntu 26.04 の開発パッケージだけでは
  `rtpjitterbuffer` などの runtime plugin が入らないことなので、2つの Dockerfile に
  `gstreamer1.0-plugins-base` と `gstreamer1.0-plugins-good` を追加した。イメージ再構築後に
  voice-host テストと almost を再実行する必要があると判断した。
- worktree テストは custom の通常 feature と `codex-exec --all-features` の両方で単独成功した。
  all-features 実行時に見つかった、現行 `bootstrap_auth_config()` が既に内包している
  auth-route 解決の未使用計算も削除し、追加 warning が出ないことを確認した。
- Docker イメージ再構築・MCP 再起動後、voice-host の対象テストは `1 passed; 50 filtered out`。
  その時点の MCP `make almost` 相当も fmt／Linux sandbox build／test-almost の全 step が終了コード0
  だったが、後続の skip 見直し後に同じ相当実行を再度行った結果は別記のとおり失敗した。したがって、
  現時点で almost が成功したとは扱わない。

## 20260914 の skip 分類と再確認

- `flaky_test_list.txt` に一時追加されていた
  `suite::subagent_notifications::subagent_notification_is_included_without_wait` と
  `managed_install::path_tests::older_managed_binary_does_not_claim_updater_support` は、
  全体実行限定の flaky と確定する根拠を再確認する前の追加だったため登録しない状態に戻した。
  手順書の条件（全体実行での失敗、fully-qualified 単体実行の成功、タイミング等の原因確認）を
  満たしたテストだけを同リストに登録する。
- 最新の `almost` ログで失敗した
  `provider::tests::configured_provider_models_manager_uses_provider_bearer_token` は skip せず、
  MCP の除外なし `codex-model-provider --lib` 83件を実行して `83 passed; 0 failed` を確認した。
  したがって provider の実装・テスト・skip list は変更していない。provider 確認後に別途行った MCP
  `almost` 相当は provider を除外せず `fmt`・Linux sandbox build・test-almost の全工程が終了コード0
  だったが、skip 見直し後の再実行では後述の TUI テストで一度失敗した。
- 以前 skip されていた network approval の4テストは、skip を外した fully-qualified 実行で各 `1 passed`
  となり、直近の almost ログでも4件とも `ok` だった。この4件は skip list から削除した。
- `network_rejection_preserves_execution_and_review_outcomes` の3ケースは、除外なしの focused 実行で
  3件とも `background process remained alive after network rejection` の deadline failure になった。
  さらに同じ MCP 環境で、ネットワークを使わない `codex-core` の process-group cleanup 2テストと
  `codex-utils-pty` の PTY cleanup 1テストも、それぞれ descendant が残るため失敗した。このため、
  現時点の根拠は proxy bridge／NET_ADMIN 固有ではなく、環境の descendant/process-group cleanup である。
- MCP 環境 probe では `CapBnd` に NET_ADMIN のビットはある一方 `CapEff=0000000000000000` で、
  `bwrap --unshare-net ... ip link set lo up` は `RTNETLINK answers: Operation not permitted` になった。
  ただしこれは単純な probe の結果であり、上記の非ネットワーク cleanup failure も確認済みなので、これだけを
  proxy bridge の根拠にはしない。`flaky_test_list.txt` には入れず、skip list には current test の関数
  プレフィックスを1行だけ残している。
- skip 見直し後の最新 `almost` 相当は、current test を除外した状態で
  `app_server_session::rollout_history::tests::cached_legacy_resume_revalidates_history_across_migration_settings`
  が失敗し、`4641 passed; 1 failed; 3 ignored; 4 filtered out` だった。
- 上記は skip 対象ではなく、起動時のバックグラウンド migration と resume が maintenance lock を
  取得する順序に依存して、同じ安全な処理でも request 数が `+2` または `+3` になるテスト前提の問題だった。
  `rollout_history_tests.rs` の検証を、`Legacy/+2` と `Paginated/+3` の許容される直列化結果だけに限定し、
  予期しない request 数・履歴モードは引き続き失敗するように修正した。skip には追加していない。
- Dockerfile の Ubuntu／GStreamer 設定は voice-host の依存解決には効いたが、今回の process-group failure
  の説明にはならない。実行経路の `--init`／`init: true` は未設定で、その有効性を再構築・再起動後に検証して
  いないため、Docker 設定を直したとも process-group 用 skip が最終確定したともまだ判断しない。

## 20260914 skip 外の失敗の修正と再確認

- File: `codex-rs/tui/src/app_server_session/rollout_history_tests.rs`
  - Line: 337
  - Resolution: 手動修正（上流テストの意図を維持）
  - Note: startup migration が resume より先に lock を取得した場合の `Paginated/+3` も、resume が
    先に取得した `Legacy/+2` と同じく lock で直列化された正当な結果として検証するようにした。
    skip や flaky list には追加していない。
- MCP の codex-tui 全テストは `4646 passed; 0 failed; 3 ignored`、`almost` 相当は fmt／Linux sandbox
  build／test-almost の全 step が終了コード0。`_tmp/almost_test_result.txt` の workspace test も
  `4642 passed; 0 failed; 3 ignored; 4 filtered out` である。

## 20260915 全体実行で追加された失敗の整理

- `suite::directory_trust::connected_trust_cancellation_and_acceptance_control_task_creation` と
  `suite::worktree_stack::picker_side_worktree_fork_and_cd_run_on_the_production_stack` は、全体実行時に
  共通の PTY startup／screen 待機（30秒）を超過した。各 fully-qualified 実行は成功したため、
  `flaky_test_list.txt` に2件だけ追加した。これは directory trust や worktree の機能を無効化する
  `skip_test_list.txt` への追加ではない。
- `fd_mount::tests::mismatched_mount_closes_inherited_descriptor` は、close 後の raw descriptor 番号を
  別の並列テストが再利用し得る assertion で失敗した。既存の duplicate descriptor テストと同じ
  全体並列の競合で、focused 実行は成功したため `flaky_test_list.txt` に exact name を追加した。
- `app::tests::daybreak_tests::cyber_refusal_reads_eligibility_without_changing_the_model` は、production
  の `read_notice` が持つ3秒 timeout とテスト側の待機期限が同じだった。production の挙動は変更せず、
  全体負荷の polling 遅延を許容するようテスト側の待機だけを10秒にした。TUI lib 全4,649件は成功し、
  flaky／skip list には追加していない。
- `provider::tests::configured_provider_models_manager_uses_provider_bearer_token` は、全 workspace 並列時
  に local WireMock のモデル取得結果が bundled catalog に戻った assertion 失敗だった。対象テスト、
  `codex-model-provider --lib` 全83件、および他 crate と同時実行した crate 全体は成功したため、
  手順書の条件に従い `flaky_test_list.txt` に exact name を理由付きで追加した。provider 実装は変更せず、
  `skip_test_list.txt` にも追加していない。

## 20260915 最終検証

- MCP の `codex-tui --lib` 全4,649件: 成功。
- MCP の `codex-model-provider --lib` 全83件: 成功。
- 上記 provider bearer テストの focused 再実行: `1 passed`。
- MCP `run_make_almost_equivalent`: `ok: true`, `exit_code: 0`。fmt、Linux sandbox build、test-almost を
  含めて成功。

内容上は競合マーカーを除去済み。ただし `.git` が read-only mount のため index を更新する
`git add`／`git rebase --continue` は実行できず、Git の unmerged stage は残っている。Git metadata
が書き込み可能な環境で、次の競合パスを `git add` して rebase を継続する必要がある。
