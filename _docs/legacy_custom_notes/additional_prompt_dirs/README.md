# CODEX_ADDITIONAL_PROMPT_DIRS（追加のカスタムプロンプト探索パス）

> 旧 custom 機能の記録。現在の `custom2` では維持対象外。

## 目的

- `/prompts:<name>` で参照するカスタムプロンプトを、`$CODEX_HOME/prompts/` 以外のディレクトリにも置けるようにする。
- プロジェクトごとのプロンプト置き場（例: リポジトリ配下の `./prompts/`）と、個人のプロンプト置き場を併用できるようにする。

## 変更内容

- 環境変数 `CODEX_ADDITIONAL_PROMPT_DIRS` を追加した。
  - 値は **コンマ区切り**のパス列（例: `./prompts,../shared-prompts,/abs/prompts`）。
  - **相対パスはカレントディレクトリ**（Codex セッションの `cwd`）からの相対として解釈する。
- TUI 起動時に `$CODEX_HOME/prompts/` と追加ディレクトリを走査し、`/prompts:<name>` の候補として使うようにした。
- 同名プロンプトが複数ディレクトリに存在する場合は、後に読み込まれたものが優先される（後勝ち）。
- 追加ディレクトリが存在しない場合は無視し、プロンプト一覧の生成を継続する。

## 対象範囲

- TUI のスラッシュ入力 `/prompts:<name>` と、スラッシュポップアップに表示されるカスタムプロンプト一覧。

## 注意点

- 追加ディレクトリは「ディレクトリとして読み取り可能な場合のみ」使用され、存在しない/読めない場合は無視される（エラーにはならない）。
- プロンプトはセッション開始時に読み込まれるため、ファイルを追加・編集したら Codex を再起動する。
- テストでは環境変数を直接 mutate せず、追加ディレクトリを明示的に渡せる内部ヘルパーで検証している。

## 動作確認

`make` のターゲットはデフォルトで `CODEX_HOME=<リポジトリ配下>/_cache/codex_home_debug` を使う。

- `make verify-additional-prompt-dirs-env`
- `cargo test -p codex-tui custom_prompts`

手動での簡易確認（例）:

1. `./prompts/hello.md` を作る（`prompts/` は任意のディレクトリ名でOK）。
2. `CODEX_ADDITIONAL_PROMPT_DIRS=./prompts codex` を起動する。
3. TUI で `/prompts:hello` が候補に出る/実行できることを確認する。
