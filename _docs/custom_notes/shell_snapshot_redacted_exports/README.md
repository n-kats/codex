# Shell snapshot の exports を最小化（ホスト環境変数の出力を避ける）

## 背景（問題）

- `shell_snapshot` は `export -p` / `env` 相当を snapshot ファイルに書き出していたため、以下が起きうる。
  - テストログやスナップショットに **ホストの環境変数が大量に出力**される（秘匿情報の混入や差分のノイズになり得る）
  - 実行環境差（PATH や各種 XDG 変数など）で snapshot の内容が揺れ、再現性が落ちる

## 変更内容

- `shell_snapshot` が書き出す `exports` セクションを、**安全寄りの許可リスト**に限定した。
  - 例: `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `TMPDIR`, `XDG_*` など
- `OPENAI_API_KEY` のようなホストの秘匿値が snapshot に混入しにくくなる。
- `shell_environment_policy` で落とした変数が、snapshot の `source` で復活する経路も抑える。

## 影響範囲

- `shell_snapshot` の `exports` セクションのみ（functions / aliases / setopts は従来どおり）。
- Codex が実行安定化のために明示注入する runtime 変数（例: `NO_COLOR`, `PAGER`, `CODEX_THREAD_ID`）や、shell が生成する `PWD` / `SHLVL` は対象外。

## 動作確認（手元環境で実行）

- `make verify-linux-default-shell`
- `make test-all`
  - `_tmp/test_all_test_result.txt` に、ホストの環境変数が大量に出力されないことを確認する。
- 個別確認: `cargo test -p codex-core --lib shell_snapshot`

## 関連ファイル

- `codex-rs/core/src/shell_snapshot.rs`
