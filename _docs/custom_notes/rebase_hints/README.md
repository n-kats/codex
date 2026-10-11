# rebase ヒント（上流更新取り込み）

このリポジトリは `openai/codex` をフォークしており、定期的に上流（例: `fork-origin/main`）を取り込みます。このノートは、rebase 時の衝突を減らし、衝突解消ミスを早期に検知するためのヒントをまとめたものです。

## 目的

- 上流更新を安全に取り込み、カスタムの意図を保つ。
- コンフリクト解消の「変な採用（片側丸ごと/ロジック欠落）」を `range-diff` 等で早期に見つける。

## 変更内容（このノートの対象）

- git rebase の作業手順と確認観点（チェックリスト）
- `git range-diff` を使った差分確認の型

## 対象範囲（非対象も）

- 対象: rebase/衝突解消/差分検査/force-push までの流れ
- 非対象: 特定カスタムの仕様（各カスタムは対応する `_docs/custom_notes/*` を参照）

## 注意点

- rebase 後の push は基本的に `git push --force-with-lease` になる（履歴を書き換えるため）。
- stash に機密を入れない（入ってしまった場合はコミットしない/共有しない）。
- `CUSTOM.md` の方針により、エージェント環境ではビルド/テスト/フォーマットは実行しない。手元環境で `make` ターゲットを使う。

## 手順（例）

### 1) 作業前の状態確認

- `git status -sb`
- 必要なら一時退避: `git stash push -u -m "wip before rebase"`

### 2) 上流更新 → rebase

- `git fetch fork-origin`
- `git diff --stat fork-origin/main...custom` などで差分量を見て、大きい場合は設計変更が必要か確認して報告する
- `git rebase fork-origin/main`

### 3) コンフリクト解消の基本

- 片側丸ごと採用（`--ours/--theirs`）を多用しない。意図がある箇所だけ局所的に解消する。
- 大きい衝突は「どの上流変更に追従する必要があるか」を先に把握してから直す（例: module rename / 型名変更 / 引数変更）。

### 4) `range-diff` で衝突解消の妥当性チェック

rebase 前後のパッチ列の対応を見て、「意図しない差分増加/ロジック欠落」がないか確認します。

- 共通の分岐点（旧ブランチと上流の merge-base）を取る:
  - `BASE=$(git merge-base origin/custom fork-origin/main)`
- `range-diff`（rebase 前: `origin/custom`、rebase 後: `custom` の例）:
  - `git range-diff $BASE..origin/custom fork-origin/main..custom`

見方の目安:

- `=`: 同一（問題になりにくい）
- `!`: 内容が変化（ここを重点確認）
- `<` / `>`: 消えた/増えた（意図した削除・追加か確認）

`!` になったコミットは、そのコミット単位で `git show` して変更点が「上流追従」か「意図したカスタム」になっているか確認します。

### 5) カスタムが残っているか（ざっくり確認）

まずは “目印” を grep して、カスタムが消えていないことを確認します（詳細は各 custom_notes を参照）。

- カスタムノート: `_docs/custom_notes/*`
- 例: TUI 送信キー表記: `rg -n \"ctrl \\+ enter to send \\(or ctrl \\+ j\\)\" codex-rs/tui`
- 例: Langfuse: `rg -n \"langfuse\\.observation\\.\" codex-rs/otel`

### 6) 手元環境での動作確認

- `make verify-all-custom`
- 必要に応じて `make all`（時間がかかる）

### 7) push

- `git push --force-with-lease origin custom`

## つまずきと対処

- `range-diff` が大きすぎて見づらい:
  - `git range-diff ... | rg -n \"[!<>]\"` で変化があるコミットだけに絞る。
- `--config` などの CLI 変更でテストが落ちる:
  - まず “どのバイナリの CLI か” を確認する。本家追従のため `--config` は `-c key=value` と同じ override として扱い、custom の config.toml ファイル指定は `--config-file` に分離する。

## よく直したパターン（症状 → 原因 → 対処）

このセクションは「rebase で衝突解消したあとに出やすい、典型的な直し方」を記録します。衝突解消が正しいか不安なときは、まずここを確認します。

### 1) `structs are not allowed in struct definitions`

- 症状: `error: structs are not allowed in struct definitions`（“nested struct” 扱い）
- 原因: コンフリクト解消で `}` を落として、`struct A { ... struct B { ... } }` のように見えてしまっている。
- 対処: まず構文を戻す（`struct`/`impl` の括弧を正しい位置で閉じる）。その後に `super::Type` 参照が復活しているか確認する。
- 確認: `cargo test -p codex-core --lib`（最低限コンパイルが通ること）

### 2) module move / re-export での名前解決エラー（例: `could not find ... in ...`）

- 症状: `failed to resolve: could not find '...' in '...'`（例: `codex_otel::otel_manager::...` が消える）
- 原因: 上流の module 構成変更（ファイル移動、`pub mod traces;` 配下へ移動、`pub use` の位置変更など）。
- 対処:
  - まず “実体” を `rg` で探し、正しいパスに合わせる。
  - `use` で明示 import して参照を短くする（型名が長いほど rebase で壊れやすい）。
- 確認: 変更箇所のクレートをピンポイントで `cargo test -p <crate>`。

### 3) CLI テストが `unexpected argument '--xxx' found` で落ちる

- 症状: `error: unexpected argument '--config' found` のように clap が弾く。
- 原因:
  - “global flag” を後置できることを期待しているが、CLI 定義で `global = true` になっていない。
  - 複数バイナリで同名フラグが既に別用途で使われている、または本家が同名フラグを追加した（例: 本家の `--config key=value` と custom の config.toml ファイル指定が衝突する、など）。
- 対処:
  - テストが期待する “どのバイナリ” の CLI かを特定する（例: `codex` / `codex-exec` / `codex-cli`）。
  - 同名衝突がある場合は、バイナリ固有のトップレベル CLI に追加して吸収する（`TopCli` で parse → inner に merge 等）。
- 確認: 落ちたテストを単体で再実行して通ることを確認する。

### 4) `pattern does not mention field ...`（struct/enum のフィールド追加）

- 症状: `pattern does not mention field 'final_output_json_schema'` のようなエラー。
- 原因: 上流で struct/enum variant にフィールドが追加され、テストの destructuring が追従できていない。
- 対処: テスト側の `match` を `Ok(Type { needed, .. })` にする（本当に必要なフィールドだけ抜き出す）。
- 確認: 該当テストの再実行。

### 5) `has no field named ...`（struct リテラルが古い）

- 症状: `struct X has no field named 'use_shift_enter_hint'` のようなエラー。
- 原因: 上流で struct のフィールドが削除/改名されているのに、テスト/呼び出し側が残っている。
- 対処: リテラル側を更新（削除/改名に追従）。`Default::default()` を使っているなら、意図せず違う既定値になっていないかも確認する。
- 確認: 該当クレートのテスト。

### 6) `dead_code` 警告（variant が “存在するが使われない”）

- 症状: `variant ... is never constructed` のような警告。
- 原因: enum に variant を追加したが、生成側（composer 等）が一度も返していない。
- 対処:
  - “ハンドリング側” が存在するなら、生成側も実装する（例: `/review <args>` で `CommandWithArgs` を返す）。
  - 逆に不要なら variant ごと削除する。
- 確認: `cargo test -p codex-tui` / `cargo test -p codex-tui2`。

### 7) TUI/TUI2 の status スナップショットが `Agents.md: <none>` → `../AGENTS.md` で落ちる

- 症状:
  - `status_snapshot_*` がスナップショット差分になり、`Agents.md:` 行だけが `<none>` から `../AGENTS.md` 等に変化する。
- 原因:
  - テストが `ConfigBuilder::default().build()` に依存していると、`cwd` が実ワークツリー内になる。
  - `discover_project_doc_paths()` が `cwd` から Git ルート（`.git`）まで探索し、実リポジトリの `AGENTS.md` を見つけるため、表示が環境依存になる。
- 対処（方針）:
  - 目的は「スナップショット更新」ではなく「テストをホスト/作業ツリー非依存にする」。
  - TUI/TUI2 のテスト用 `Config` 生成で `harness_overrides` を使い、`no_config: true` と `cwd: Some(temp_dir)` を指定する。
  - テスト内で `config.cwd` を固定値（例: `/workspace/tests`）にせず、`temp_home.path().join("tests")` のように temp 配下へ寄せる。
- 追加メモ:
  - `*.snap.new` は `insta` が失敗時に一時生成するファイルなので、対処後は削除して良い（必要ならテスト再実行で再生成される）。

### 8) TUI2 の paste-burst / Enter 挙動テストが改行抜けで落ちる（`hi\nthere` が `hithere` になる等）

- 症状:
  - `ascii_burst_treats_enter_as_newline` / `non_ascii_burst_*` などで、Enter が改行としてバッファに入らず、改行が欠落する。
- 原因:
  - `Enter`（修飾なし）を「textarea が空なら何もしない」で握りつぶす経路があると、paste-burst 中の `append_newline_if_active()` に到達できない。
- 対処:
  - paste-burst の “改行はバッファに入れる” 挙動が `Enter` 経路でも必ず通るようにする（burst context のときは `None/false` で return しない）。

### 9) TUI2 の Enter と slash 文脈が衝突する（未知の `/xxx ` でも Enter がコマンド扱いになる）

- 症状:
  - `enter_inserts_newline_for_unknown_slash_command_text` が落ちる（Enter が改行を入れない）。
- 原因:
  - `Enter`（修飾なし）で「先頭が `/` なら submit 経路」を強制すると、未知コマンド入力までコマンド扱いになって newline が入らない。
- 対処:
  - `Enter`（修飾なし）は dispatch 経路に入れない（改行のみ）。
  - slash command の dispatch は `Ctrl+Enter` / `Ctrl+J` に限定する。

### 10) TUI2 の footer スナップショットが “右カラム開始位置/行位置” のズレで落ちる

- 症状:
  - `footer_mode_shortcut_overlay` / `footer_shortcuts_shift_and_esc` のスナップショットで、右カラムの開始位置がズレる・最終行（`ctrl + t ...`）が別の位置になる。
- 原因:
  - `build_columns()` の列幅計算（padding/gap）や「奇数個の項目をどちらのカラムに置くか」を変更すると、全行の整列が変わりスナップショットが崩れる。
- 対処:
  - “最後の項目（例: ShowTranscript）を右カラムに置く” など、期待している配置を先に固定してから列計算を調整する。
  - 列間の gap/padding の変更は影響が大きいので、上流追従以外では基本触らない（触ったら意図を `_docs/custom_notes/` に残す）。

## 関連ファイル一覧

- `CUSTOM.md`
- `_docs/custom_notes/README.md`
- `Makefile`
