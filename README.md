## カスタム（このリポジトリ固有）

このリポジトリは `openai/codex` をフォークしてカスタマイズしています（詳細: `CUSTOM.md`）。

現在のカスタム一覧:

- （機能追加）TUI の入力: Enter で改行、Ctrl+Enter（または Ctrl+J）で送信
- （機能追加）AGENTS.md の明示指定: `codex --agents-md <FILE>`（複数指定可）でプロジェクトドキュメントを指定し、`AGENTS.md` の自動探索を上書きできる
- （機能追加）TUI の AGENTS.md 切替: `/custom-agents <path> [path...]`（または `clear`）でセッション中のプロジェクトドキュメント指定を切り替えできる
- （機能追加）TUI の更新チェック: `x.y.z-custom-...` のようなカスタム版バージョン文字列でも更新判定できるようにする（詳細: `_docs/custom_notes/update_check_custom_version_suffix/README.md`）
- （機能追加）config.toml の読み込み制御: `codex --config <FILE>` / `codex --no-config`
- （機能追加）Codex home の上書き: `codex --codex-home PATH`（`CODEX_HOME` と同等）
- （機能追加）Memories ルートの上書き: `codex --codex-memory PATH`（`CODEX_MEMORIES_HOME` と同等。既定は `$CODEX_HOME/memories`。詳細: `_docs/custom_notes/codex_memory_cli_flag/README.md`）
- （機能追加）カスタムプロンプト: `CODEX_ADDITIONAL_PROMPT_DIRS`（コンマ区切り、相対パスはカレントディレクトリ基準）で探索ディレクトリを追加
- （テスト）シェル初期化ファイル: `CODEX_SHELL_STARTUP_FILES=clean`（または `codex --shell-startup-files=clean`）でユーザー dotfiles を可能な範囲で無視して実行（現状は zsh を `ZDOTDIR` で隔離）
- （機能追加）`!`（UserShell）の注入/ローカル記録を無効化: `custom.user_shell.no_inject=true`（詳細: `_docs/custom_notes/user_shell_no_inject/README.md`）
- （機能追加）コマンド実行の権限分離: モデルが実行する `shell` / `shell_command` / `exec_command` を worker ユーザー（例: `assistant`）に固定できる（設定は `custom.exec.*`。安全のため `shell_environment_policy.inherit = "all"` との併用はエラー。詳細: `_docs/custom_notes/command_exec_worker_user/README.md`）
- （上流不具合修正・追従）exec-server（elicitation）: execve-wrapper が `git` のような素のコマンド名を送っても `PATH` で実行ファイルを解決し、`EscalateRequest.file` を絶対パス化して扱う（公式が直ったら差分を寄せて削除予定）
- （テスト）Shell snapshot: `exports` セクションは許可リストに限定し、ホスト環境変数の大量出力を避ける
- （テスト）テスト/ログの安全性: 失敗時の差分表示でホスト環境変数が全量出力されないようにする（`env` は値を丸ごと比較しない）
- （テスト）tool parallelism: 並列ツールテストの判定を「時間」から「tool出力」へ変更し、Docker 等での不安定さを排除
- （テスト）exec-server: `dotslash` を Docker イメージに同梱し、DotSlash 由来の bash を使えるようにする
- （テスト）動作確認: `make` の検証ターゲットはデフォルトで `CODEX_HOME=<リポジトリ配下>/_cache/codex_home_debug` を使用（`run-tui` は `CODEX_MEMORIES_HOME=<リポジトリ配下>/_cache/codex_memory_debug` も設定）
- （テスト）動作確認ログ: `make test-*` / `make verify-*` 実行時のログを `_tmp/*_test_result.txt` に保存（`tee`）
- （開発運用）フォーマット（rustfmt）: `make fmt`（=`cargo +nightly fmt`）で実行
- （開発運用）NOTICE: フォークで加えた変更の著作権表記を `NOTICE` に追記
- （テスト）既知の不安定テスト回避: `make almost`（=`make fmt` + `make test-almost`）を用意し、環境依存で揺れやすいテストを `--skip` して基本的な検証を回せるようにする
- （修正）Langfuse OTEL 連携: 長寿命セッションで trace が生成できない（trace row 不在/parent 404）問題の修正（詳細: `_docs/custom_notes/langfuse_logging/README.md`）
- （機能追加）Langfuse OTEL 連携: trace 名の付与、LLM の入出力（プロンプト/レスポンス）全量の可視化（詳細: `_docs/custom_notes/langfuse_logging/README.md`）

## セットアップ（ホストで worker ユーザー運用）

モデルが実行するコマンド（`shell` / `shell_command` / `exec_command`）を、専用 OS ユーザー（例: `assistant`）で実行したい場合の導入メモです（`custom.exec.worker_user` を使う前提）。

### 1) Releases からバイナリを入れる

- GitHub Releases から該当プラットフォームのアーカイブをダウンロードして展開し、`codex` を `PATH` の通った場所に配置します（例: `/usr/local/bin/codex`）。

### 2) worker ユーザーを作る

```bash
sudo useradd --create-home --shell /bin/bash assistant
```

### 3) `exec_command` 用に sudo を通す（必要な場合のみ）

`exec_command` は内部で `sudo -n -u ... -g ...` を使ってユーザー切替するため、パスワード無し sudo が必要です（`shell` / `shell_command` には不要）。

- 例（要調整）: `visudo` で sudoers に `youruser ALL=(assistant:assistant) NOPASSWD: ALL` を追加

### 4) `shell` / `shell_command` 用に `setcap` を付ける

`custom.exec.worker_user` を使う場合、Codex は child process 側で `setgroups/setgid/setuid` を行う必要があるため、ホストで非 root 運用するなら `codex` 実体に capability を付与します。

```bash
sudo setcap cap_setuid,cap_setgid=ep "$(command -v codex)"
```

注意:
- capability は「ファイル」に付くため、バイナリを入れ替える（アップグレード/再インストール）と消えます。更新後は `setcap` をやり直してください。
- `setcap` が使えない環境（例: rootless Docker の overlayfs など）では、`sudo -n -u "#UID" -g "#GID" -- env -i ...` へのフォールバックで動かせます（必要に応じて `/tmp/codex-argv0/<argv0>` の symlink 経由で argv0 を維持します）。

### 5) 設定例（config.toml）

`custom.exec.worker_user` と、`!` とツール実行で環境変数ポリシーを分離する例です。

```toml
[custom.exec]
worker_user = "assistant"

# 互換のため残しておく（未指定時の既定）
[shell_environment_policy]
inherit = "core"
ignore_default_excludes = false

# モデルが起動するツール（shell/shell_command/exec_command/...）側
[custom.assistant_shell_environment_policy]
inherit = "core"
ignore_default_excludes = false
set = { HOME = "/home/assistant" }

# ユーザー起点の `!` 側（invoker のまま）
[custom.user_shell_environment_policy]
inherit = "core"
ignore_default_excludes = false
set = { HOME = "/home/youruser" }
```

---
<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install --cask codex</code></p>
<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="./.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

## Quickstart

### Installing and running Codex CLI

Install globally with your preferred package manager:

```shell
# Install using npm
npm install -g @openai/codex
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then simply run `codex` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Team, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
