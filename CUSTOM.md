# Codex カスタム方針

このリポジトリは `openai/codex` をフォークしており、その上で行う Codex カスタマイズ方針（恒久ルール）をここに記録する。

## 目的

- 開発効率を上げつつ、上流更新を取り込みやすい（rebase/merge しやすい）状態を保つ。

## このファイルに書くこと（スコープ）

- Codex の振る舞いを継続的に変えるための「方針・ルール・置き場」を記録する。
- 一時的な作業の手順・タスクリストはここに書かず、`_docs/worklists/` に記録する。
- カスタマイズを行ったら、必ず `_docs/custom_notes/{custom-name}/` に知見を十分詳しく記録する。
  - 最低限含める: 目的 / 変更内容（何がどう変わるか） / 対象範囲（非対象も） / 注意点（環境差・既知の制約） / 動作確認手順（手動・テスト・スナップショット） / つまずきと対処（警告や失敗の修正） / 関連ファイル一覧
- 作業開始時は、必ず最初に `CUSTOM.md` を参照する。

## 原則（rebase しやすさ優先）

- 変更は小さく、局所的に行う（不要なリネーム・並べ替え・整形は避ける）。
- 差分が膨らむ変更（大規模な再フォーマット等）は避ける。
- カスタムの追加は「追記」を基本にし、既存の規約・指示文の改変は最小化する。
- 明示的な指示がない限り、上流由来の領域（例: 既存の `docs/` や `README.md` 等）は編集しない。

## コマンド実行の方針（この環境の制約）

- ここでいう「この環境」は、Codex が動作している実行環境（エージェント側の環境）を指す。
- 動作確認（ビルド/テスト/フォーマット）は **エージェント側では実行しない**。実行は利用者の手元環境（この環境外）で行う。
- そのため、動作確認コマンドは `Makefile` にターゲットとして追加して記録する（手元で `make ...` を実行できる形にする）。

## 置き場（カスタムを入れる場所）

- カスタム方針（恒久ルール）: `CUSTOM.md`
- 参考資料（背景・検討メモ・参考リンク等）: `_docs/`
- カスタムの知見（背景・設計・注意点・検証手順）: `_docs/custom_notes/{custom-name}/`
- 一時的なタスクリスト: `_docs/worklists/`
- よく使うコマンド（手元での実行用）: `Makefile`
- `AGENTS.md`: `CUSTOM.md` を参照する旨のみ（追加ルールは書かない）

## `_docs/custom_notes` インデックス

このリポジトリのカスタム関連の知見（背景・設計・注意点・検証手順）の置き場。新規カスタムを追加したら、ここにも追記する。

参照の目安:

- 実装や挙動を変更する前に、同じ領域の既存カスタムがないか確認したいとき
- テストが落ちた / 差分が増えたときに、既知の制約・回避策・検証手順を探したいとき
- 上流更新で衝突したときに、差分の意図（なぜ必要か）を素早く把握したいとき

各ノート（どの機能に関係するか）:

- `_docs/custom_notes/README.md`: custom_notes 全体の概要（追加時の方針）
- `_docs/custom_notes/codex_home_cli_flag/README.md`: `--codex-home` / `CODEX_HOME` の上書き（ホーム切替・テスト用ホーム運用）
- `_docs/custom_notes/additional_prompt_dirs/README.md`: `CODEX_ADDITIONAL_PROMPT_DIRS`（カスタムプロンプト探索パス）
- `_docs/custom_notes/tui-enter-newline-ctrl-enter-send/README.md`: TUI（tui）入力キー（Enter=改行、Ctrl+Enter/Ctrl+J=送信）と関連テスト
- `_docs/custom_notes/tui2_input_submit_behavior_tests/README.md`: TUI2（tui2）入力キーと送信挙動、`/prompts:` の引数なし挙動、関連テスト
- `_docs/custom_notes/exec_command_default_login/README.md`: `!`/shell 実行の login 制御（`CODEX_USER_SHELL_LOGIN` 等）
- `_docs/custom_notes/command_exec_worker_user/README.md`: コマンド実行を worker ユーザー（assistant 等）に固定する方針（未実装）
- `_docs/custom_notes/linux_default_shell_prefers_bash_over_zsh/README.md`: Linux のデフォルトシェル検出・bash 優先・関連テスト
- `_docs/custom_notes/shell_snapshot_redacted_exports/README.md`: Shell snapshot の `exports` マスキング（秘匿情報混入回避）
- `_docs/custom_notes/test_output_redacts_host_env/README.md`: テスト出力・ログからホスト環境変数の漏えい回避
- `_docs/custom_notes/langfuse_logging/README.md`: Langfuse ロギング連携（修正: 長寿命セッションで trace が生成できない問題、機能追加: OTEL trace 名/LLM 入出力可視化）
- `_docs/custom_notes/unified_exec_end_event_deterministic/README.md`: UnifiedExec の end event（取りこぼし/決定性）と関連テスト
- `_docs/custom_notes/hooks/README.md`: 外部コマンド hooks 構想（未実装、notify 拡張案含む）

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

- （機能追加）TUI の入力: Enter で改行、Ctrl+Enter（または Ctrl+J）で送信。
- （機能追加）config.toml の読み込み制御: `--config <FILE>` でユーザー `config.toml` の読み込みパスを任意に指定でき、`--no-config` でユーザー＋プロジェクトの config を無視できる（システム config や `-c key=value` は引き続き適用される）。
- （機能追加）Codex home の切り替え: `--codex-home PATH` で `CODEX_HOME`（デフォルト `~/.codex`）を上書きできるようにする（詳細: `_docs/custom_notes/codex_home_cli_flag/README.md`）。
- （機能追加）カスタムプロンプト探索パスの追加: `CODEX_ADDITIONAL_PROMPT_DIRS`（コンマ区切り、相対パスはカレントディレクトリ基準）でプロンプト探索ディレクトリを追加できるようにする（詳細: `_docs/custom_notes/additional_prompt_dirs/README.md`）。
- （テスト）シェル初期化ファイルの制御: `CODEX_SHELL_STARTUP_FILES=clean`（または `codex --shell-startup-files=clean`）で、可能な範囲でユーザー dotfiles を読まずにシェルを起動できるようにする（現状は zsh を `ZDOTDIR` で隔離）（検証・再現性のための制御、詳細: `_docs/custom_notes/linux_default_shell_prefers_bash_over_zsh/README.md` / `_docs/custom_notes/exec_command_default_login/README.md`）。
- （テスト）`!` のユーザーコマンドの login 制御: `CODEX_USER_SHELL_LOGIN=0` で `-c`（非 login）、未指定なら `-lc`（login）で実行する（検証・再現性のための制御）。
- （上流不具合修正・追従）exec-server（elicitation）: execve-wrapper が `git` のような素のコマンド名を送っても `PATH` で実行ファイルを解決し、`EscalateRequest.file` を絶対パス化して扱う（elicitation の文言一致と `execv()` の確実な実行のため）。公式（openai/codex の main）側で同様の修正が入ったら差分を寄せて削除する。
  - （テスト観点）`codex-exec-server` の `suite::accept_elicitation::accept_elicitation_for_prompt_rule` が、elicitation 文言の不一致により auto-accept されず（結果として deny 扱いになり）失敗するため、この修正で通ることを確認する。
    - 検証例: `cd codex-rs && cargo test -p codex-exec-server --test all suite::accept_elicitation::accept_elicitation_for_prompt_rule`
- （テスト）Shell snapshot: `exports` セクションは許可リストに限定し、ホスト環境変数の大量出力（秘匿情報混入）を避ける（詳細: `_docs/custom_notes/shell_snapshot_redacted_exports/README.md`）。
- （テスト）テスト/ログの安全性: 失敗時の差分表示でホスト環境変数が全量出力されないようにする（例: `env` は値を丸ごと比較せず、キー集合＋必要最小限のキーのみ値比較にする）（詳細: `_docs/custom_notes/test_output_redacts_host_env/README.md`）。
- （テスト）動作確認: `make verify-*` 系ターゲットはデフォルトで `CODEX_HOME=<リポジトリ配下>/_cache/codex_home` を使って実行する。
- （テスト）動作確認ログ: `make test-*` / `make verify-*` 実行時のログを `_tmp/*_test_result.txt` に保存する。
- （開発運用）フォーマット（rustfmt）: 上流の `codex-rs/rustfmt.toml` は `imports_granularity = "Item"` を含むため、フォーマットは `make fmt`（=`cargo +nightly fmt`）で実行する（安定版 rustfmt だと警告が出る）。
- （開発運用）NOTICE: フォークで加えた変更の著作権表記として `Modifications Copyright (c) 2025 Katsunori Nakanishi` を `NOTICE` に追記する。
- （テスト）既知の不安定テスト回避: `make almost`（=`make fmt` + `make test-almost`）を用意し、環境依存で揺れやすいテストを `--skip` して基本的な検証を回せるようにする（`SKIP_ALMOST_TESTS` でスキップ対象を変更できる）。
  - デフォルトのスキップ対象（`Makefile` の `SKIP_ALMOST_TESTS`）:
    - `view_image_tool_attaches_local_image`: GUI 必須ではないが、`ViewImageToolCall` 等のイベント待ちが固定タイムアウト（5秒）に依存しており、実行環境の負荷・ファイルIO・スケジューリングの揺れで間欠的にタイムアウトしやすい。
    - `approval_matrix_covers_all_modes`: サンドボックス拒否時の OS/ロケール依存エラーメッセージ（例: `Permission denied` / `許可がありません`）に依存した期待が含まれ、言語設定やシェル差で間欠的に失敗しやすい。
  - `make almost` は `fmt` が失敗しても `test-almost` を続行し、どちらかが失敗したら最後に失敗として終了する。
    - `make almost` 実行ログは `_tmp/almost_test_result.txt` に集約して保存する（途中で止まってもログが残ることを優先）。
    - ログ集約のため、内部的に `LOG_FILE` と `LOG_APPEND=1` を使って、配下ターゲットの `tee` 先を統一する。
  - 同様に、集約ターゲット（例: `make all` / `make verify-all-custom` / `make verify-codex-home-cli-flag`）も、途中で失敗しても残りの検証を続行し、最後に失敗として終了する（途中経過のログを残すことを優先する）。
    - 集約ログの出力先:
      - `make all`: `_tmp/all_test_result.txt`
      - `make verify-all-custom`: `_tmp/verify_all_custom_test_result.txt`
      - `make verify-codex-home-cli-flag`: `_tmp/verify_codex_home_cli_flag_test_result.txt`
  - `make almost` は「開発中の高速な安全確認」用で、最終確認は `make all`（= フォーマット + 全テスト）を優先する。
