# 20260907 rebase follow-up

## 対象

- upstream base: `fork-origin/main` (`5ecb3afd1bf4`)
- custom side: `custom` / `tmp-rebase` (`2d3ac3e82684`)
- rebase state: `refs/heads/custom` の `2d3ac3e82684` 適用中

## 対応結果

- generated experimental schema は upstream の user-verification 変更と custom の `projectDocPaths` を統合した。
- CLI／exec／TUI は upstream の managed-worktree と remote 処理を維持し、custom の PSP、loader override、cloud 無効時の分岐を残した。
- upstream で削除された `codex-mcp-server` と `mcp_test_support` は復元せず、workspace／lockfile からも除去した。
- upstream の `SessionTarget.cwd` 追加に合わせて startup test fixture を更新した。
- 追加監査で、孤立していた `core-skills`／`mcp_edit` テスト、現行依存に対応しない reqwest deny 例外5件、機能に関係しない doc／空行差分を削除した。
- `ContentItemKinds` の古い `UnderDevelopment`／無効化 override も、現行 custom 方針に根拠がなく upstream が Stable／既定有効のため削除した。
- 削除済み `codex mcp-server` を指すコメント参照も一般化した。
- issue labeler の同コマンド向け説明も、現行の MCP server 用途に一般化した。
- Cloud Tasks の rebase 用ソース／コメント、exec-server の終了判定・retry、HTTP/TLS 防御処理など根拠のある差分は維持した。

## 検証結果

- cargo fmt check: 成功
- 変更接合部の selected tests: 成功
- `codex-features` の 41 tests: 成功
- 旧 MCP command／孤立テスト／旧 deny 例外／`ContentItemKinds` override の参照なしを確認
- `make almost` 相当: fmt／Linux sandbox build 成功、workspace test は `glib-2.0.pc` 不足で停止

作業ツリーの競合マーカーは除去済み。`.git` が read-only のため index を更新する `git add` と rebase continue は未実行で、ユーザー側で Git metadata が書き込み可能な環境に戻した後に完了できる。
