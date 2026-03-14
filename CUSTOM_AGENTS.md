# Codex カスタム方針 + AGENTS 統合（CUSTOM_AGENTS）

このファイルは、リポジトリルートの運用・開発ルールとカスタム方針を統合した「運用・開発ルール集」です。
以後、作業開始時に参照する一次情報源は原則として本ファイルとし、個別カスタムの背景・設計・検証手順は `_docs/custom_notes/` 配下を参照します。

---

## 目的

- 開発効率を上げつつ、上流更新を取り込みやすい（rebase/merge しやすい）状態を保つ。

## このファイルに書くこと（スコープ）

- Codex の振る舞いを継続的に変えるための「方針・ルール・置き場」を記録する。
- 利用者への回答は原則として日本語で行う（利用者が別言語を明示した場合を除く）。
- 一時的な作業の手順・タスクリストはここに書かず、`_worklist/` に記録する。
- カスタマイズを行ったら、必ず `_docs/custom_notes/{custom-name}/` に知見を十分詳しく記録する。
  - 最低限含める: 目的 / 変更内容（何がどう変わるか） / 対象範囲（非対象も） / 注意点（環境差・既知の制約） / 動作確認手順（手動・テスト・スナップショット） / つまずきと対処（警告や失敗の修正） / 関連ファイル一覧
- 作業開始時は、必ず最初に `CUSTOM_AGENTS.md` を参照する。

## 原則（rebase しやすさ優先）

- 変更は小さく、局所的に行う（不要なリネーム・並べ替え・整形は避ける）。
- 差分が膨らむ変更（大規模な再フォーマット等）は避ける。
- カスタムの追加は「追記」を基本にし、既存の規約・指示文の改変は最小化する。
- 明示的な指示がない限り、上流由来の領域（例: 既存の `docs/` や `README.md` 等）は編集しない。

---

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

---

## コマンド実行の方針（この環境の制約）

- ここでいう「この環境」は、Codex が動作している実行環境（エージェント側の環境）を指す。
- エージェントが調査や検証のためにコマンドを実行する場合（ビルド/テスト/フォーマットを含む）は、原則として **MCP ツール経由で実施する**（再現性とログ収集のため）。
- エージェント側の環境には `docker` / `cargo` / `just` などが入っていない前提で扱い、**インストールもしない**（他の依存コマンドも同様）。
- そのため、動作確認コマンドは `Makefile` にターゲットとして追加して記録する（手元で `make ...` を実行できる形にする）。
- エージェント側の環境では **`make` を実行しない**（ログファイル上書きや Docker 依存により、調査用ログを破壊しやすいため）。
- ログを確認する場合は、`_tmp/*_test_result.txt` を読む（`sed` / `rg` / `stat` 等）だけにし、`make` の再実行でログを更新しない。
- `git range-diff` の出力ログを保存する場合は、`_tmp/range-diff/` に置く（`_worklist/` には置かない）。

### テストは MCP ツール経由で実行する

- エージェントが実行するテスト/検証は、原則として **MCP ツール経由で実施する**（再現性とログ収集のため）。
  - MCP ツールを介さずに直接コマンドを実行して「動いた」だけでは、検証として扱わない（例外を設ける場合は、理由と差分を明記する）。
- 検証手順（`Makefile` ターゲットや `_docs/custom_notes/**/README.md` の手順）には、必ず「どの MCP ツール（例: `shell` / `view_image`）を使うか」を書く。
- MCP ツールの挙動/前提/制約は、この節に追記して記録する（分散させない）。

#### MCP ツール説明（検証で使うもの）

- `shell`:
  - 目的: ビルド/テスト/フォーマット等の非対話コマンドを実行する。
  - 前提: 実行環境の制約（サンドボックス/ネットワーク制限/ユーザー権限/環境変数ポリシー等）の影響を受ける。
  - 運用: 検証は可能な限り `Makefile` の `verify-*` / `test-*` ターゲットに寄せ、ログは `_tmp/*_test_result.txt` に残す。
- `view_image`:
  - 目的: ローカル画像を入力として渡す（画像入力が絡む E2E/統合テストで利用する）。
  - 前提: 実行環境の負荷やファイル I/O に影響されやすく、イベント待ちのタイムアウト等で間欠的に失敗しうる。

---

## 置き場（カスタムを入れる場所）

- カスタム方針（恒久ルール）: `CUSTOM_AGENTS.md`
- 参考資料（背景・検討メモ・参考リンク等）: `_docs/`
- カスタムの知見（背景・設計・注意点・検証手順）: `_docs/custom_notes/{custom-name}/`
- 一時的なタスクリスト: `_worklist/`
- `git range-diff` の出力ログ: `_tmp/range-diff/`
- コンフリクト解消ログ: `_notes/deconflict/`
- よく使うコマンド（手元での実行用）: `Makefile`

---

## まず辿る導線（作業開始チェックリスト）

- 1. `CUSTOM_AGENTS.md`（このファイル）を最初に読む。
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

- 上流取り込み（rebase）を始める: `CUSTOM_AGENTS.md` -> `rebase_rules` -> 必要に応じて `rebase_hints`
- テストが落ちて原因を当てる: `CUSTOM_AGENTS.md` の制約確認 -> `rebase_hints` -> 関連 custom ノート
- custom テストを追加/修正する: `custom_tests` -> 関連 custom ノート
- `make` / Docker 実行可否で迷う: `CUSTOM_AGENTS.md` の「コマンド実行の方針（この環境の制約）」を優先

---

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
- `_docs/custom_notes/tui-enter-newline-ctrl-enter-send/README.md`: TUI の入力仕様（Enter=改行、Ctrl+Enter/Ctrl+J=送信）と回帰テスト（スナップショット含む）の位置。
- `_docs/custom_notes/update_check_custom_version_suffix/README.md`: TUI 更新チェックのバージョン比較（`x.y.z-custom-...` を正しく比較するための仕様・実装・テスト）。
- `_docs/custom_notes/release_versioning/README.md`: `make release` の配布物バージョニング（`x.y.z-custom-yyyy-mm-dd` 形式の付与ルールとリリース手順）。
- `_docs/custom_notes/exec_command_default_login/README.md`: `!`/shell 実行の起動ファイル読み込み制御（`CODEX_SHELL_STARTUP_FILES` と再現性、関連テスト）。
- `_docs/custom_notes/command_exec_worker_user/README.md`: モデル起因のコマンド実行を worker ユーザーへ固定する方針（権限分離、supplementary groups、禁止組み合わせ、テスト）。
- `_docs/custom_notes/user_shell_environment_policy_split/README.md`: `!`（UserShell）とモデル起動コマンドの環境変数ポリシー分離（`custom.user_shell_environment_policy` 等の設定意図と影響範囲）。
- `_docs/custom_notes/linux_default_shell_prefers_bash_over_zsh/README.md`: Linux のデフォルトシェル検出（bash 優先）と、zsh/dotfiles 差による揺れを抑えるための注意点・テスト。
- `_docs/custom_notes/shell_snapshot_redacted_exports/README.md`: Shell snapshot の秘匿対策（`exports` の出力を許可リスト化して漏えいを避ける設計とテスト）。
- `_docs/custom_notes/test_output_redacts_host_env/README.md`: テスト失敗ログの秘匿対策（ホスト環境変数を全量出力しない、差分表示の安全性、関連テスト）。
- `_docs/custom_notes/langfuse_logging/README.md`: Langfuse/OTEL ロギング連携（既知不具合の修正点、可視化の追加点、設定・テストの観点）。
- `_docs/custom_notes/agents_md_and_custom_agents_restore/README.md`: `--agents-md` と `/custom-agents` の復元（session 反映経路、`project_doc_paths` の復旧、関連テスト）。
- `_docs/custom_notes/custom_theme_diff_colors/README.md`: `custom.theme.diff` による TUI 差分色の上書き（追加/削除の背景色、無効化方針、関連テスト/スナップショット）。
- `_docs/custom_notes/unified_exec_end_event_deterministic/README.md`: UnifiedExec の end event 安定化（取りこぼし防止・決定性、関連テスト）。

履歴/参考（現状の実装に直接対応しない）:

- `_docs/custom_notes/hooks/README.md`: hooks 構想メモ（未実装。notify 拡張案など将来検討用の背景）。
- `_docs/custom_notes/tui2_input_submit_behavior_tests/README.md`: 旧 TUI2 の履歴（入力キー・送信挙動・`/prompts:` 引数なし挙動の経緯。tui2 は削除済みなので背景参照用）。

---

## Rust / codex-rs（`codex-rs/` 配下）開発ルール

`codex-rs` フォルダ（Rust 実装）に適用するルール:

- crate 名は `codex-` プレフィックスを付ける（例: `core` の crate は `codex-core`）。
- `format!` を使うとき、`{}` に変数をインラインできるなら必ずそうする。
- ここに書かれた手順の実行に必要なコマンド（例: `just` / `rg` / `cargo-insta`）が無い場合でも、エージェント側の環境ではインストールしない（`docker` / `cargo` / `just` 等を含む）。
- `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` または `CODEX_SANDBOX_ENV_VAR` に関するコードは **絶対に追加・変更しない**。
  - `shell` ツール使用時は `CODEX_SANDBOX_NETWORK_DISABLED=1` が設定される。既存の `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` 利用コードはこの前提で書かれている。
  - Seatbelt（`/usr/bin/sandbox-exec`）でプロセスを起動すると子プロセスに `CODEX_SANDBOX=seatbelt` が設定される。Seatbelt を自前で起動する統合テストは Seatbelt 下で動かせないため、必要に応じて `CODEX_SANDBOX=seatbelt` を検出して早期終了する場合がある。
- Clippy の指摘に従う:
  - if 文は collapsible にする: https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
  - `format!` 引数は可能な限りインライン化する: https://rust-lang.github.io/rust-clippy/master/index.html#uninlined_format_args
  - 可能ならクロージャではなくメソッド参照を使う: https://rust-lang.github.io/rust-clippy/master/index.html#redundant_closure_for_method_calls
- 可能な限り `match` は網羅的にし、ワイルドカードアームは避ける。
- テストでは、フィールドごとの比較よりも「オブジェクト全体の等価比較」を優先する。
- API を追加・変更した場合、該当するなら `docs/` 配下の文書も更新する。
- `ConfigToml` またはネストされた config 型を変更した場合、`codex-rs/` で `just write-config-schema` を実行して `codex-rs/core/config.schema.json` を更新する。
- Rust 依存（`Cargo.toml` / `Cargo.lock`）を変更した場合:
  - リポジトリルートで `just bazel-lock-update` を実行し、`MODULE.bazel.lock` を同じ変更に含める。
  - その後、リポジトリルートで `just bazel-lock-check` を実行して drift を検出する。
- 1 回しか参照されない小さな helper メソッドは作らない。
- 大きなモジュールを避ける:
  - 既存モジュールを肥大化させるより、新しいモジュールを追加する。
  - テストを除いて 500 LoC 未満を目標にする。
  - おおむね 800 LoC を超える場合、強い理由がない限り新機能は新モジュールに追加する。
  - 特に以下のような高頻度で触られるファイルは肥大化を避ける:
    - `codex-rs/tui/src/app.rs`
    - `codex-rs/tui/src/bottom_pane/chat_composer.rs`
    - `codex-rs/tui/src/bottom_pane/footer.rs`
    - `codex-rs/tui/src/chatwidget.rs`
    - `codex-rs/tui/src/bottom_pane/mod.rs`
    - など中央オーケストレーション系
  - 大きいモジュールから機能を抽出する場合、関連テストや型ドキュメントも新しい実装側へ移して、所有する不変条件を近づける。

Rust 変更後のフォーマットとテスト:

- Rust のコード変更が終わったら、`codex-rs/` で `just fmt` を自動で実行する（許可を求めない）。
- テストは次の順で実行する:
  1. 変更したプロジェクトのテストを実行する（例: `codex-rs/tui` を変更したなら `cargo test -p codex-tui`）。
  2. それが通った後、common/core/protocol を変更した場合はフルスイートを実行する（`cargo test`、`cargo-nextest` があれば `just test`）。通常は `--all-features` を避け、必要な場合のみ使う。
  3. プロジェクト単位/個別テストは確認なしで実行してよいが、フルスイート実行は利用者に確認してから行う。
- `codex-rs` の大きめの変更を finalize する前に、`codex-rs/` で `just fix -p <project>` を実行して lint を直す（共有 crate を触った場合のみ `-p` 無しを検討）。`fix` や `fmt` の後にテストは再実行しない。

---

## TUI（`codex-rs/tui`）スタイル規約

- スタイル規約: `codex-rs/tui/styles.md` を参照する。

### TUI コード規約（ratatui）

- ratatui の `Stylize` を使った簡潔なヘルパーを優先する。
  - 基本の span: `"text".into()`
  - スタイル付き span: `"text".red()` / `"text".green()` / `"text".magenta()` / `"text".dim()` など
  - `Span::styled` / `Style` の手組みよりこちらを優先する。
  - 例（patch summary のファイル行）: `vec!["  └ ".into(), "M".red(), " ".dim(), "tui/src/app.rs".dim()]`

### TUI Styling（ratatui）

- 可能なら `Stylize` の `.dim()` / `.bold()` / `.cyan()` / `.italic()` / `.underlined()` を使い、手動の `Style` 構築は避ける。
- 単純な変換は `"text".into()` と `vec![…].into()` を優先する。推論が曖昧な場合（例: `Paragraph::new` / `Cell::from`）は `Line::from(spans)` または `Span::from(text)` を使う。
- runtime 計算の style は `Span::styled` でもよい（`Span::from(text).set_style(style)` も可）。
- `.white()` のようなハードコード白は避け、デフォルトの前景色（無色）を使う。
- チェーンは可読性重視でまとめる（例: `url.cyan().underlined()`）。
- 単一要素は `"text".into()` を優先し、必要なときだけ `Line::from(text)` / `Span::from(text)` を使う。
- `Line` を組み立てるときは、型推論が効くなら `vec![…].into()`、曖昧なら `Line::from(vec![…])`。
- churn 回避: 同等表現（`Span::styled` ↔ `set_style`、`Line::from` ↔ `.into()`）の置換は、可読性や機能上の理由がない限り行わない（ファイル内の既存流儀に合わせ、`.into()` のために型注釈を足さない）。
- 1 行に収まるなら収める。`rustfmt` 後にどちらかだけが折り返し回避できるなら、その形を採用する。

### テキスト折り返し

- 生文字列の折り返しは必ず `textwrap::wrap` を使う。
- ratatui の `Line` を折り返す場合は `codex-rs/tui/src/wrapping.rs` の helper（例: `word_wrap_lines` / `word_wrap_line`）を使う。
- 折り返しのインデントは、可能なら `RtOptions` の `initial_indent` / `subsequent_indent` を使う（自前ロジックは避ける）。
- 複数行に一括で prefix を付ける場合は `line_utils` の `prefix_lines` を使う。

---

## テスト（スナップショット等）

### スナップショットテスト（insta）

- このリポジトリはスナップショットテスト（`insta`）を多用する（特に `codex-rs/tui`）。
- **要件:** ユーザー可視の UI 変更（新 UI 追加を含む）は、対応する `insta` スナップショットカバレッジを必ず追加/更新する。

意図的に UI / テキスト出力を変更した場合:

- `cargo test -p codex-tui` を実行してスナップショット差分を生成する。
- `cargo insta pending-snapshots -p codex-tui` で pending を確認する。
- 生成された `*.snap.new` を直接読むか、`cargo insta show -p codex-tui path/to/file.snap.new` で個別表示する。
- `codex-tui` で新スナップショットをすべて受け入れる意図なら `cargo insta accept -p codex-tui`。
- ツールが無い場合は `cargo install cargo-insta`。

### テストのアサーション

- diff を見やすくするため、テストでは `pretty_assertions::assert_eq` を使う（未導入なら test module の先頭で import する）。
- フィールド単位より、可能な限り deep equals（オブジェクト全体の `assert_eq!()`）を優先する。
- テストでプロセス環境を直接 mutate しない。上位から環境由来フラグ/依存を渡す。

### ワークスペースバイナリ起動（Cargo vs Bazel）

- ファーストパーティバイナリをテストで起動する必要がある場合、`assert_cmd::Command::cargo_bin(...)` や `escargot` より `codex_utils_cargo_bin::cargo_bin("...")` を優先する。
  - Bazel では runfiles 配下に置かれることがあるため、`chdir` 後も安定する絶対パス解決が必要。
- fixture / テスト資源のパス解決で `env!("CARGO_MANIFEST_DIR")` を避け、`codex_utils_cargo_bin::find_resource!` を使う（Cargo と Bazel の両方で安定させる）。

### Integration tests（core）

- E2E の Codex テストは `core_test_support::responses` を優先して使う。
- `mount_sse*` は `ResponseMock` を返すので保持し、`/responses` の POST ボディを assert する。
- 1 回だけの POST なら `ResponseMock::single_request()`、複数なら `ResponseMock::requests()` を使う。
- `ResponsesRequest` の helper（`body_json` / `input` / `function_call_output` / `custom_tool_call_output` / `call_output` / `header` / `path` / `query_param`）で構造化 assert を行う。
- SSE payload は `ev_*` コンストラクタと `sse(...)` で組む。
- `wait_for_event_with_timeout` より `wait_for_event` を優先する。
- `mount_sse_once_match` / `mount_sse_sequence` より `mount_sse_once` を優先する。

典型パターン:

```rust
let mock = responses::mount_sse_once(&server, responses::sse(vec![
    responses::ev_response_created("resp-1"),
    responses::ev_function_call(call_id, "shell", &serde_json::to_string(&args)?),
    responses::ev_completed("resp-1"),
])).await;

codex.submit(Op::UserTurn { ... }).await?;

let request = mock.single_request();
```

---

## App-server API 開発ベストプラクティス（`codex-rs`）

適用範囲（特に重要）:

- `app-server-protocol/src/protocol/common.rs`
- `app-server-protocol/src/protocol/v2.rs`
- `app-server/README.md`

### コアルール

- 新規 API は app-server v2 に追加する（v1 に新しい API 面は追加しない）。
- payload の命名は一貫させる:
  - リクエスト: `*Params`
  - レスポンス: `*Response`
  - 通知: `*Notification`
- RPC メソッド名は `<resource>/<method>` とし、`<resource>` は単数にする（例: `thread/read` / `app/list`）。
- wire のフィールドは、タグ付き union や互換要件がない限り `#[serde(rename_all = "camelCase")]` を使う。
  - 例外: config 系 RPC payload は config.toml のキーに合わせて snake_case（`app-server-protocol/src/protocol/v2.rs` の config read/write/list を参照）。
- v2 の request/response/notification 型には `#[ts(export_to = "v2/")]` を付け、生成 TS を正しい namespace に出す。
- v2 の payload フィールドに `#[serde(skip_serializing_if = "Option::is_none")]` を使わない。
  - 例外: params が意図的に無い request は `params: #[ts(type = "undefined")] #[serde(skip_serializing_if = "Option::is_none")] Option<()>` を使ってよい。
- `#[serde(rename = "...")]` を使う場合、`#[ts(rename = "...")]` も合わせる。
- discriminated union は両方に明示タグを付ける: `#[serde(tag = "type", ...)]` / `#[ts(tag = "type", ...)]`。
- API 境界の ID は基本 `String` を使う（UUID 変換は内部で行う）。
- timestamp は Unix 秒（`i64`）で `*_at` 命名（例: `created_at` / `updated_at` / `resets_at`）。
- 実験的 API:
  - `#[experimental("method/or/field")]` を使う。
  - フィールド単位 gating が必要なら `ExperimentalApi` を derive する。
  - 一部フィールドのみが実験的なメソッドでは `common.rs` 側で `inspect_params: true` を使う。

### Client->server リクエスト payload（`*Params`）

- optional フィールドには必ず `#[ts(optional = nullable)]` を付ける（`*Params` 以外では使わない）。
- optional の collection（`Vec` / `HashMap` 等）は `Option<...>` + `#[ts(optional = nullable)]` にする（`#[serde(default)]` で optional collection を表現しない。v2 payload で `skip_serializing_if` は使わない）。
- 省略＝`false` にしたい bool は `Option<bool>` ではなく、`#[serde(default, skip_serializing_if = "std::ops::Not::not")] pub field: bool` を使う。
- 新しい list メソッドはデフォルトで cursor pagination を実装する:
  - request: `pub cursor: Option<String>` / `pub limit: Option<u32>`
  - response: `pub data: Vec<...>` / `pub next_cursor: Option<String>`

### 開発フロー

- API 挙動を変えたら docs/examples（最低でも `app-server/README.md`）を更新する。
- API shape が変わったら schema fixture を再生成する:
  - `just write-app-server-schema`
  - 実験的 API に影響するなら `just write-app-server-schema --experimental`
- `cargo test -p codex-app-server-protocol` で検証する。
- `common.rs` の request フィールドごとの experimental マーカーだけを assert するような boilerplate テストは避け、schema 生成/テストと挙動のカバレッジに寄せる。

---

## カスタム一覧

- （機能追加）TUI の入力: Enter で改行、Ctrl+Enter（または Ctrl+J）で送信。
- （機能追加）TUI の更新チェック: `x.y.z-custom-...` のようなカスタム版バージョン文字列でも更新判定できるようにする（詳細: `_docs/custom_notes/update_check_custom_version_suffix/README.md`）。
- （機能追加）config.toml の読み込み制御: `--config <FILE>` でユーザー `config.toml` の読み込みパスを任意に指定でき、`--no-config` でユーザー＋プロジェクトの config を無視できる（システム config や `-c key=value` は引き続き適用される）。
- （機能追加）Codex home の切り替え: `--codex-home PATH` で `CODEX_HOME`（デフォルト `~/.codex`）を上書きできるようにする（詳細: `_docs/custom_notes/codex_home_cli_flag/README.md`）。
- （機能追加）Memories ルートの切り替え: `--codex-memory PATH` で `CODEX_MEMORIES_HOME`（既定は `$CODEX_HOME/memories`）を上書きできる（詳細: `_docs/custom_notes/codex_memory_cli_flag/README.md`）。
- （機能追加）カスタムプロンプト探索パスの追加: `CODEX_ADDITIONAL_PROMPT_DIRS`（コンマ区切り、相対パスはカレントディレクトリ基準）でプロンプト探索ディレクトリを追加できるようにする（詳細: `_docs/custom_notes/additional_prompt_dirs/README.md`）。
- （テスト）シェル初期化ファイルの制御: `CODEX_SHELL_STARTUP_FILES=clean`（または `codex --shell-startup-files=clean`）で、可能な範囲でユーザー dotfiles を読まずにシェルを起動できるようにする（現状は zsh を `ZDOTDIR` で隔離）（検証・再現性のための制御、詳細: `_docs/custom_notes/linux_default_shell_prefers_bash_over_zsh/README.md` / `_docs/custom_notes/exec_command_default_login/README.md`）。
- （機能追加）コマンド実行の権限分離: モデルが実行する `shell` / `shell_command` / `exec_command` を worker ユーザー（例: `assistant`）に固定できる（設定は `custom.exec.*`。`custom.exec.worker_user` 指定時は supplementary groups も反映。安全のため `shell_environment_policy.inherit = "all"` との併用はエラー。`!` は invoker のまま。詳細: `_docs/custom_notes/command_exec_worker_user/README.md`）。
- （機能追加）`!`（UserShell）のモデル汚染/履歴保存を抑制: `custom.user_shell.no_inject = true` で、`!` コマンドの内容/出力をモデルコンテキストへ inject せず、ローカルのセッション履歴にも保存しない（詳細: `_docs/custom_notes/user_shell_no_inject/README.md`）。
- （上流不具合修正・追従）exec-server（elicitation）: execve-wrapper が `git` のような素のコマンド名を送っても `PATH` で実行ファイルを解決し、`EscalateRequest.file` を絶対パス化して扱う（elicitation の文言一致と `execv()` の確実な実行のため）。公式（openai/codex の main）側で同様の修正が入ったら差分を寄せて削除する。
  - （テスト観点）`codex-exec-server` の `suite::accept_elicitation::accept_elicitation_for_prompt_rule` が、elicitation 文言の不一致により auto-accept されず（結果として deny 扱いになり）失敗するため、この修正で通ることを確認する。
    - 検証例: `cd codex-rs && cargo test -p codex-exec-server --test all suite::accept_elicitation::accept_elicitation_for_prompt_rule`
- （テスト）Shell snapshot: `exports` セクションは許可リストに限定し、ホスト環境変数の大量出力（秘匿情報混入）を避ける（詳細: `_docs/custom_notes/shell_snapshot_redacted_exports/README.md`）。
- （テスト）テスト/ログの安全性: 失敗時の差分表示でホスト環境変数が全量出力されないようにする（例: `env` は値を丸ごと比較せず、キー集合＋必要最小限のキーのみ値比較にする）（詳細: `_docs/custom_notes/test_output_redacts_host_env/README.md`）。
- （テスト）tool parallelism: 並列ツールテストの判定を「時間」から「tool出力」へ変更し、Docker 等での不安定さを排除する（詳細: `_docs/custom_notes/tool_parallelism_test/README.md`）。
- （テスト）exec-server: `dotslash` を Docker イメージに同梱し、exec-server テストで DotSlash 由来の bash を使えるようにする（詳細: `_docs/custom_notes/exec_server_tests_dotslash/README.md`）。
- （テスト）custom 専用テスト運用: 上流に無いテストは `custom` 専用として分離し、`custom__...` 命名で絞り込み実行できるようにする（詳細: `_docs/custom_notes/custom_tests/README.md`）。
- （テスト）動作確認: `make verify-*` 系ターゲットはデフォルトで `CODEX_HOME=<リポジトリ配下>/_cache/codex_home_debug` を使って実行する。
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
