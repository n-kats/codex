# allowed_sudo_fallback

## 目的

`custom.exec.worker_user` の実行で許可する fallback を明確にする。

## 許可する fallback

- 許可する fallback は、worker user 実行を維持するための `sudo -n -u "#UID" -g "#GID" -- env -i ...` のみ。
- この fallback は、`setuid` / `setgid` capability がないホスト環境でも `custom.exec.worker_user` の権限分離を保つためのもの。
- fallback 後も実行ユーザーは worker user であり、invoker 権限には戻さない。

## 許可しない fallback

- worker 実行に失敗したコマンドを invoker 権限で自動再実行すること。
- 既存の spawn / stdio / process cleanup 実装を custom 側にコピーして別経路を作ること。
- `sudo` を使うために、既存の `spawn.rs` と別の起動経路を custom 側へ丸ごと持つこと。

## 実装方針

- custom 側は sudo fallback 用の `Command` 構築だけを持つ。
- stdio 設定、`pre_exec`、`kill_on_drop`、spawn は既存の共通経路を使う。
- 理由: 本家実装を維持し、rebase 時の衝突と挙動差を最小化するため。

## 関連ファイル

- `_docs/custom_notes/command_exec_worker_user/README.md`
- `codex-rs/core/src/spawn.rs`
- `codex-rs/core/src/custom/exec/run_as.rs`
