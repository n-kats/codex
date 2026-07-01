# Codex カスタム方針

このリポジトリは `openai/codex` をフォークしており、その上で行う Codex カスタマイズ方針（恒久ルール）をここに記録する。

## 目的

- 開発効率を上げつつ、上流更新を取り込みやすい（rebase/merge しやすい）状態を保つ。

## このファイルに書くこと（スコープ）

- Codex の振る舞いを継続的に変えるための「方針・ルール・置き場」を記録する。
- 利用者への回答は原則として日本語で行う（利用者が別言語を明示した場合を除く）。
- 一時的な作業の手順・タスクリストはここに書かず、`_worklist/` に記録する。
- カスタマイズを行ったら、必ず `_docs/custom_notes/{custom-name}/` に知見を十分詳しく記録する。
  - 最低限含める: 目的 / 変更内容（何がどう変わるか） / 対象範囲（非対象も） / 注意点（環境差・既知の制約） / 動作確認手順（手動・テスト・スナップショット） / つまずきと対処（警告や失敗の修正） / 関連ファイル一覧
- 作業開始時は、必ず最初に `CUSTOM.md` を参照する。

## 原則（rebase しやすさ優先）

- 変更は小さく、局所的に行う（不要なリネーム・並べ替え・整形は避ける）。
- 差分が膨らむ変更（大規模な再フォーマット等）は避ける。
- カスタムの追加は「追記」を基本にし、既存の規約・指示文の改変は最小化する。
- 明示的な指示がない限り、上流由来の領域（例: 既存の `docs/` や `README.md` 等）は編集しない。

## rebase 運用ルール（custom ブランチ）

- 対象: `fork-origin/main` に対する `custom` ブランチの rebase。
- 作業前に日付ブランチ（`YYYYMMDD`）を作成して、rebase 前の状態を保存する（過去再現用）。
- `custom` ブランチは rebase 前に squash して 1 つのコミットにまとめる。
- `tmp-rebase` ブランチを作成して比較用に残す（既に存在する場合は作業停止）。
- `fork-origin/main` に rebase し、コンフリクトを解消する。
- コンフリクト解消時は、`_notes/deconflict/` にある過去ログを最低 1 件は読み、解消方針の参考にする。
- コンフリクトを解消したら、`_notes/deconflict/` に「どのファイルのどの行で、どう解消したか」を追記または新規記録する。
- `git range-diff` で `tmp-rebase` と比較し、rebase 内容を確認する。
- rebase 後の修正はテストが通るまで `--amend` / squash しない（追加コミットで進める）。
- `fork-origin/main` に無いテストは `custom` 専用として分離し、`custom` を含むファイル名（例: `custom_tests.rs`）にのみ追加する（詳細: `_docs/custom_notes/rebase_rules/README.md`）。
- `git range-diff` の目視レビュー後、カスタム仕様レポート（項目ごとの実装箇所・テスト/検証）を作成して表示し、欠損がないことを確認する。
- 実行手順（コマンド）: `_docs/custom_notes/rebase_rules/README.md` を参照する。

## コマンド実行の方針（この環境の制約）

- ここでいう「この環境」は、Codex が動作している実行環境（エージェント側の環境）を指す。
- 動作確認（ビルド/テスト/フォーマット）は **エージェント側では実行しない**。実行は利用者の手元環境（この環境外）で行う。
- エージェント側の環境には `docker` が入っていない前提で扱い、**インストールもしない**（他の依存コマンドも同様）。
- そのため、動作確認コマンドは `Makefile` にターゲットとして追加して記録する（手元で `make ...` を実行できる形にする）。
- エージェント側の環境では **`make` を実行しない**（ログファイル上書きや Docker 依存により、調査用ログを破壊しやすいため）。
- ログを確認する場合は、`_tmp/*_test_result.txt` を読む（`sed` / `rg` / `stat` 等）だけにし、`make` の再実行でログを更新しない。
- `git range-diff` の出力ログを保存する場合は、`_tmp/range-diff/` に置く（`_worklist/` には置かない）。

## 置き場（カスタムを入れる場所）

- カスタム方針（恒久ルール）: `CUSTOM.md`
- 参考資料（背景・検討メモ・参考リンク等）: `_docs/`
- カスタムの知見（背景・設計・注意点・検証手順）: `_docs/custom_notes/{custom-name}/`
- 一時的なタスクリスト: `_worklist/`
- `git range-diff` の出力ログ: `_tmp/range-diff/`
- コンフリクト解消ログ: `_notes/deconflict/`
- よく使うコマンド（手元での実行用）: `Makefile`
- `AGENTS.md`: `CUSTOM.md` を参照する旨のみ（追加ルールは書かない）

## まず辿る導線（作業開始チェックリスト）

- 1. `CUSTOM.md`（このファイル）を最初に読む。
  - 特に「rebase 運用ルール」「コマンド実行の方針（この環境の制約）」を先に確認する。
- 2. `_docs/custom_notes/README.md` で custom_notes 全体の方針と入口を確認する。
- 3. 作業目的に応じて、次を読む。
- rebase 作業: `_docs/custom_notes/rebase_rules/README.md`
- rebase 後の差分確認・原因調査: `_docs/custom_notes/rebase_hints/README.md`
- コンフリクト解消の履歴参照: `_notes/deconflict/README.md`
  - custom テスト追加/命名/分離: `_docs/custom_notes/custom_tests/README.md`
  - 既存 custom の仕様確認: 該当の `_docs/custom_notes/{custom-name}/README.md`
- 4. 必要な実装・文書更新後、該当 custom ノート（`_docs/custom_notes/{custom-name}/README.md`）に今回の変更点と検証手順を追記する。

作業タイプ別の最短ルート:

- 上流取り込み（rebase）を始める: `CUSTOM.md` -> `rebase_rules` -> 必要に応じて `rebase_hints`
- テストが落ちて原因を当てる: `CUSTOM.md` の制約確認 -> `rebase_hints` -> 関連 custom ノート
- custom テストを追加/修正する: `custom_tests` -> 関連 custom ノート
- `make` / Docker 実行可否で迷う: `CUSTOM.md` の「コマンド実行の方針（この環境の制約）」を優先

## `_docs/custom_notes` インデックス

このリポジトリのカスタム関連の知見（背景・設計・注意点・検証手順）の置き場。新規カスタムを追加したら、ここにも追記する。

参照の目安:

- 実装や挙動を変更する前に、同じ領域の既存カスタムがないか確認したいとき
- テストが落ちた / 差分が増えたときに、既知の制約・回避策・検証手順を探したいとき
- 上流更新で衝突したときに、差分の意図（なぜ必要か）を素早く把握したいとき
- rebase の衝突解消後に「よくある直し方（症状→原因→対処）」で当たりを付けたいとき（`_docs/custom_notes/rebase_hints/README.md`）

各ノート（どの機能に関係するか）:

全体（運用・方針・テスト運用）:

- `_docs/custom_notes/README.md`: custom_notes の置き方・追加ルール（新規 custom 作成時の最初の入口）。
- `_docs/custom_notes/rebase_rules/README.md`: custom ブランチの rebase 運用ルール（squash、`tmp-rebase`、`range-diff`、custom テストの扱い）。
- `_docs/custom_notes/rebase_hints/README.md`: rebase 時の実務ヒント集（`range-diff` の見方、よくある衝突・症状→原因→対処、修正パターン）。
- `_docs/custom_notes/custom_tests/README.md`: custom 専用テスト運用（上流非依存に分離、`custom__機能名__テスト内容` 命名、`custom` を含むファイル名に限定、一覧/絞り込み手順）。
- `_docs/custom_notes/docker_test_env/README.md`: Docker ベースのビルド/テスト環境（Makefile の Docker 化、実行環境差の前提、ローカル運用の注意）。
- `_docs/custom_notes/tool_parallelism_test/README.md`: tool parallelism テストの安定化（時間依存を排除し、出力ベースで判定する設計とテスト）。
- `_docs/custom_notes/exec_server_tests_dotslash/README.md`: exec-server テストの前提整備（DotSlash 由来 bash を使うための同梱、Docker/CI 前提、関連テスト）。

カスタム機能:

- `_docs/custom_notes/codex_home_cli_flag/README.md`: `--codex-home` / `CODEX_HOME` の扱い（ホーム切替・テスト用ホームを安定運用するための方針と注意点）。
- `_docs/custom_notes/codex_memory_cli_flag/README.md`: `--codex-memory` / `CODEX_MEMORIES_HOME` の扱い（memories 成果物の保存先を分離し、sandbox 初期化エラーを避ける）。
- `_docs/custom_notes/additional_prompt_dirs/README.md`: `CODEX_ADDITIONAL_PROMPT_DIRS` の仕様（追加プロンプト探索パス、相対パス基準、分離文字、関連テスト）。
- `_docs/custom_notes/update_check_custom_version_suffix/README.md`: TUI 更新チェックのバージョン比較（`x.y.z-custom-...` を正しく比較するための仕様・実装・テスト）。
- `_docs/custom_notes/release_versioning/README.md`: `make release` の配布物バージョニング（`x.y.z-custom-yyyy-mm-dd` 形式の付与ルールとリリース手順）。
- `_docs/custom_notes/exec_command_default_login/README.md`: `!`/shell 実行の起動ファイル読み込み制御（`CODEX_SHELL_STARTUP_FILES` と再現性、関連テスト）。
- `_docs/custom_notes/command_exec_worker_user/README.md`: モデル起因のコマンド実行を worker ユーザーへ固定する方針（権限分離、supplementary groups、禁止組み合わせ、テスト）。
- `_docs/custom_notes/user_shell_environment_policy_split/README.md`: `!`（UserShell）とモデル起動コマンドの環境変数ポリシー分離（`custom.user_shell_environment_policy` 等の設定意図と影響範囲）。
- `_docs/custom_notes/user_shell_no_inject/README.md`: `!`（UserShell）の注入/ローカル記録を無効化する（`custom.user_shell.no_inject=true`）。
- `_docs/custom_notes/linux_default_shell_prefers_bash_over_zsh/README.md`: Linux のデフォルトシェル検出（bash 優先）と、zsh/dotfiles 差による揺れを抑えるための注意点・テスト。
- `_docs/custom_notes/shell_snapshot_redacted_exports/README.md`: Shell snapshot の秘匿対策（`exports` の出力を許可リスト化して漏えいを避ける設計とテスト）。
- `_docs/custom_notes/test_output_redacts_host_env/README.md`: テスト失敗ログの秘匿対策（ホスト環境変数を全量出力しない、差分表示の安全性、関連テスト）。
- `_docs/custom_notes/langfuse_logging/README.md`: Langfuse/OTEL ロギング連携（既知不具合の修正点、可視化の追加点、設定・テストの観点）。
- `_docs/custom_notes/agents_md_and_custom_agents_restore/README.md`: `--agents-md` と `/custom-agents` の復元（session 反映経路、`project_doc_paths` の復旧、関連テスト）。
- `_docs/custom_notes/custom_theme_diff_colors/README.md`: `custom.theme.diff` による TUI 差分色の上書き（追加/削除の背景色、無効化方針、関連テスト/スナップショット）。
- `_docs/custom_notes/unified_exec_end_event_deterministic/README.md`: UnifiedExec の end event 安定化（取りこぼし防止・決定性、関連テスト）。
履歴/参考（現状の実装に直接対応しない）:

- `_docs/custom_notes/hooks/README.md`: hooks 構想メモ（未実装。notify 拡張案など将来検討用の背景）。

## カスタム方針の所在（AGENTS.md への追記）

- 本ファイル `CUSTOM.md` がカスタム方針の一次情報源。
- `AGENTS.md` への追記は「`CUSTOM.md` を参照すること」のみに限定する（追加ルールや詳細は `CUSTOM.md` / `_docs/` に書く）。
  - rebase 時の衝突を減らすため、`AGENTS.md` の既存内容は変更せず、末尾への追記で対応する。

### `AGENTS.md` に追記する具体的内容

`AGENTS.md` には次の内容のみを追記する（既存内容は変更しない、追記位置は末尾）。

```md
## カスタマイズ

- 必ず `CUSTOM.md` を参照し、その方針に従うこと。
```

## カスタム一覧

- （機能追加）TUI の更新チェック: `x.y.z-custom-...` のようなカスタム版バージョン文字列でも更新判定できるようにする（詳細: `_docs/custom_notes/update_check_custom_version_suffix/README.md`）。
- （機能追加）config.toml の読み込み制御: `--config <FILE>` でユーザー `config.toml` の読み込みパスを任意に指定でき、`--no-config` でユーザー＋プロジェクトの config を無視できる（システム config や `-c key=value` は引き続き適用される）。
- （機能追加）Codex home の切り替え: `--codex-home PATH` で `CODEX_HOME`（デフォルト `~/.codex`）を上書きできるようにする（詳細: `_docs/custom_notes/codex_home_cli_flag/README.md`）。
- （機能追加）Memories ルートの切り替え: `--codex-memory PATH` で `CODEX_MEMORIES_HOME`（既定は `$CODEX_HOME/memories`）を上書きできる（詳細: `_docs/custom_notes/codex_memory_cli_flag/README.md`）。
- （機能追加）カスタムプロンプト探索パスの追加: `CODEX_ADDITIONAL_PROMPT_DIRS`（コンマ区切り、相対パスはカレントディレクトリ基準）でプロンプト探索ディレクトリを追加できるようにする（詳細: `_docs/custom_notes/additional_prompt_dirs/README.md`）。
- （テスト）シェル初期化ファイルの制御: `CODEX_SHELL_STARTUP_FILES=clean`（または `codex --shell-startup-files=clean`）で、可能な範囲でユーザー dotfiles を読まずにシェルを起動できるようにする（現状は zsh を `ZDOTDIR` で隔離）（検証・再現性のための制御、詳細: `_docs/custom_notes/linux_default_shell_prefers_bash_over_zsh/README.md` / `_docs/custom_notes/exec_command_default_login/README.md`）。
- （機能追加）`!`（UserShell）の注入/ローカル記録を無効化: `custom.user_shell.no_inject=true`（詳細: `_docs/custom_notes/user_shell_no_inject/README.md`）。
- （機能追加）コマンド実行の権限分離: モデルが実行する `shell` / `shell_command` / `exec_command` を worker ユーザー（例: `assistant`）に固定できる（設定は `custom.exec.*`。`custom.exec.worker_user` 指定時は `sudo -n -u "#UID" -g "#GID" -- env -i ...` で worker ユーザー実行する。安全のため `shell_environment_policy.inherit = "all"` との併用はエラー。`!` は invoker のまま。詳細: `_docs/custom_notes/command_exec_worker_user/README.md`）。
- （上流不具合修正・追従）exec-server（elicitation）: execve-wrapper が `git` のような素のコマンド名を送っても `PATH` で実行ファイルを解決し、`EscalateRequest.file` を絶対パス化して扱う（elicitation の文言一致と `execv()` の確実な実行のため）。公式（openai/codex の main）側で同様の修正が入ったら差分を寄せて削除する。
  - （テスト観点）`codex-exec-server` の `suite::accept_elicitation::accept_elicitation_for_prompt_rule` が、elicitation 文言の不一致により auto-accept されず（結果として deny 扱いになり）失敗するため、この修正で通ることを確認する。
    - 検証例: `cd codex-rs && cargo test -p codex-exec-server --test all suite::accept_elicitation::accept_elicitation_for_prompt_rule`
- （テスト）Shell snapshot: `exports` セクションは許可リストに限定し、ホスト環境変数の大量出力（秘匿情報混入）を避ける（詳細: `_docs/custom_notes/shell_snapshot_redacted_exports/README.md`）。
- （テスト）テスト/ログの安全性: 失敗時の差分表示でホスト環境変数が全量出力されないようにする（例: `env` は値を丸ごと比較せず、キー集合＋必要最小限のキーのみ値比較にする）（詳細: `_docs/custom_notes/test_output_redacts_host_env/README.md`）。
- （テスト）tool parallelism: 並列ツールテストの判定を「時間」から「tool出力」へ変更し、Docker 等での不安定さを排除する（詳細: `_docs/custom_notes/tool_parallelism_test/README.md`）。
- （テスト）exec-server: `dotslash` を Docker イメージに同梱し、exec-server テストで DotSlash 由来の bash を使えるようにする（詳細: `_docs/custom_notes/exec_server_tests_dotslash/README.md`）。
- （テスト）custom 専用テスト運用: 上流に無いテストは `custom` 専用として分離し、`custom__...` 命名で絞り込み実行できるようにする（詳細: `_docs/custom_notes/custom_tests/README.md`）。
- （テスト）動作確認: `make verify-*` 系ターゲットはデフォルトで `CODEX_HOME=<リポジトリ配下>/_cache/codex_home_debug` を使って実行する（`run-tui` は `CODEX_MEMORIES_HOME=<リポジトリ配下>/_cache/codex_memory_debug` も設定）。
- （テスト）動作確認ログ: `make test-*` / `make verify-*` 実行時のログを `_tmp/*_test_result.txt` に保存する。
- （開発運用）フォーマット（rustfmt）: 上流の `codex-rs/rustfmt.toml` は `imports_granularity = "Item"` を含むため、フォーマットは `make fmt`（=`cargo +nightly fmt`）で実行する（安定版 rustfmt だと警告が出る）。
- （開発運用）NOTICE: フォークで加えた変更の著作権表記として `Modifications Copyright (c) 2025 Katsunori Nakanishi` を `NOTICE` に追記する。
- （テスト）既知の不安定テスト回避: `make almost`（=`make fmt` + `make test-almost`）を用意し、環境依存で揺れやすいテストを `--skip` して基本的な検証を回せるようにする（`SKIP_ALMOST_TESTS` でスキップ対象を変更できる）。
  - デフォルトのスキップ対象（`Makefile` の `SKIP_ALMOST_TESTS`）:
    - `view_image_tool_attaches_local_image`: GUI 必須ではないが、`ViewImageToolCall` 等のイベント待ちが固定タイムアウト（5秒）に依存しており、実行環境の負荷・ファイルIO・スケジューリングの揺れで間欠的にタイムアウトしやすい。
    - `approval_matrix_covers_all_modes`: サンドボックス拒否時の OS/ロケール依存エラーメッセージ（例: `Permission denied` / `許可がありません`）に依存した期待が含まれ、言語設定やシェル差で間欠的に失敗しやすい。
    - `drop_kills_wrapper_process_group`: `codex-rmcp-client` の `process_group_cleanup` で、プロセス終了待ちが実行環境負荷やスケジューリング遅延で間欠的にタイムアウトしやすい。
    - `denying_network_policy_amendment_persists_policy_and_skips_future_network_prompt`: `bwrap`（bubblewrap）/ user namespace 前提の環境差があり、コンテナ実行環境によっては前提不成立で安定して失敗しうる。
    - `unified_exec_streams_after_lagged_output`: 出力のタイミング差（負荷・スケジューリング）に依存して間欠的に失敗しうる。
    - `remote_models_merge_adds_new_high_priority_first`: プロキシ環境変数など外部環境差で、ローカルモックへの到達可否が揺れて間欠的に失敗しうる。
    - `turn_start_jsonrpc_span_parents_core_turn_spans`: 出力通知/スケジューリングのタイミング差に依存して間欠的に失敗しうる。
  - `make almost` は `fmt` が失敗しても `test-almost` を続行し、どちらかが失敗したら最後に失敗として終了する。
    - `make almost` 実行ログは `_tmp/almost_test_result.txt` に集約して保存する（途中で止まってもログが残ることを優先）。
    - ログ集約のため、内部的に `LOG_FILE` と `LOG_APPEND=1` を使って、配下ターゲットの `tee` 先を統一する。
  - 同様に、集約ターゲット（例: `make all` / `make verify-all-custom` / `make verify-codex-home-cli-flag`）も、途中で失敗しても残りの検証を続行し、最後に失敗として終了する（途中経過のログを残すことを優先する）。
    - 集約ログの出力先:
      - `make all`: `_tmp/all_test_result.txt`
      - `make verify-all-custom`: `_tmp/verify_all_custom_test_result.txt`
      - `make verify-codex-home-cli-flag`: `_tmp/verify_codex_home_cli_flag_test_result.txt`
  - `make almost` は「開発中の高速な安全確認」用で、最終確認は `make all`（= フォーマット + 全テスト）を優先する。
