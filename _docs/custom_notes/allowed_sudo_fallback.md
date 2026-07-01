# allowed_sudo_fallback

## 目的

`custom.exec.worker_user` の `sudo` 実行経路を明確にする。

## 許可する fallback

- worker user 実行は `sudo -n -u "#UID" -g "#GID" -- env -i ...` に一本化する。
- この経路は、`custom.exec.worker_user` の権限分離を保つためのもの。
- 実行後も worker user を維持し、invoker 権限には戻さない。

## 許可しない fallback

- worker 実行に失敗したコマンドを invoker 権限で自動再実行すること。
- 既存の spawn / stdio / process cleanup 実装を custom 側にコピーして別経路を作ること。
- `sudo` 以外の worker-user 実行経路を新たに増やすこと。

## 実装方針

- custom 側は sudo 用の `Command` 構築だけを持つ。
- stdio 設定、`pre_exec`、`kill_on_drop`、spawn は既存の共通経路を使う。
- 理由: 本家実装を維持し、rebase 時の衝突と挙動差を最小化するため。

## 関連ファイル

- `_docs/custom_notes/command_exec_worker_user/README.md`
- `codex-rs/core/src/spawn.rs`
- `codex-rs/core/src/custom/exec/run_as.rs`
