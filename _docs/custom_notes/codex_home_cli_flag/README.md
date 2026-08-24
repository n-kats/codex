# `--codex-home` CLI flag

## 目的

`~/.codex` 配下に保存される Codex のデータ（設定、ログ、各種キャッシュなど）の保存先を、環境変数を使わずに引数で切り替えられるようにする。

## 変更内容

- `codex --codex-home PATH` を追加し、`CODEX_HOME=PATH` と同等の意味で扱う。
- プロセス起動直後、CLI の本処理に入る前に引数を軽量パースして `CODEX_HOME` を設定する。
- 相対パスはカレントディレクトリ基準で解決してから環境変数へ入れる。

## 対象範囲

- 対象: Codex CLI のホームディレクトリ解決（`CODEX_HOME` / `~/.codex`）に依存する全ての保存先（例: `config.toml`, `log/`, `models_cache.json` など）。
- 非対象: `CODEX_HOME` を使わない別経路のキャッシュ（もし将来追加される場合は別途検討）。

## 注意点

- `--codex-home` は「引数で `CODEX_HOME` をセットする」方式のため、子プロセスにも同じ `CODEX_HOME` が引き継がれる。
- `--codex-home` は `--` より前でのみ認識する。`--` 以降の値は bootstrap の対象外。

## 動作確認

- `make verify-codex-home-cli-flag`（`CODEX_HOME=<リポジトリ配下>/_cache/codex_home_debug` で実行される）
  - Lint: `cd codex-rs && just fix -p codex-cli`
  - Test: `cd codex-rs && cargo test -p codex-cli`

## 関連ファイル

- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/custom_tests.rs`
- `docs/config.md`
- `README.md`
