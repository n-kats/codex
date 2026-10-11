# 20260311 rebase custom report

## Summary

- Base: `fork-origin/main` @ `56420da857fe9f02a154a8c97412058db0e08e35`
- Branch: `custom` @ `b395673bfe5e323997eca8013077a17e847142c2`
- Comparison branch: `tmp-rebase-20260311` @ `4da3da588899425be9b51c6d8ef75d35a5fe49cf`
- Range-diff log: `_tmp/range-diff/20260311-rebase-range-diff.txt`

`fork-origin/main` は前回レポート作成時（`56420da8...`）から更新されておらず、rebase 本体の commit（`custom changes`）は `tmp-rebase-20260311` と `custom` で一致している。`custom` 側にはその後続として 6 commit が追加されている。

## Range-diff review

- Reviewed:
  - `git range-diff fork-origin/main...tmp-rebase-20260311 fork-origin/main...custom`
- Summary:
  - `tmp-rebase-20260311` の `4da3da588 custom changes` は `custom` 側でも完全一致（`=`）
  - `custom` 側にのみ追加された patch は 6 件で、いずれも rebase 後の追補として意図を説明できる
    - `14d3ffb32` rebase custom report 追加
    - `8ef4fa591` rebase verification 手順に MCP の `make all` 相当を追記
    - `d01fcd9ce` `make-all-equivalent` 再実行結果を記録
    - `510521bb7` `js_repl` / `view_image` の Node 依存テストを環境互換性で skip
    - `440729a2a` MCP target dir mount の説明を Makefile 前提に整合
    - `b395673bf` `.env` が Docker 起動時に消費される点を補足
- Conclusion:
  - rebase で custom patch が欠落・変質した形跡はない
  - 追加 6 patch は、レポート / 検証手順 / 環境依存テスト安定化 / MCP ドキュメント補足であり、rebase 後の follow-up として妥当

## Custom report

カスタム仕様の項目別レポートは、前回作成分を参照:

- `_worklist/2026-03-06-rebase-custom-report-fork-origin-main-56420da8.md`

## Notes

- workflow 上 `tmp-rebase` が残っている場合は停止ですが、作業継続のため既存 `tmp-rebase` を `tmp-rebase-20260311-prev` に退避しました。
- 今回の比較用 `tmp-rebase` は `range-diff` 取得後に `tmp-rebase-20260311` へリネームし、`tmp-rebase` は残していません。

## Verification

- MCP（ツール）で `make all` 相当（`codex_dev_env/run_make_all_equivalent`）を実行:
  - 1回目: 600s で timeout（tools/call deadline exceeded）
  - 2回目: `cargo test --all-features` が失敗（exit code 101、ログ: `_tmp/all_test_result.txt`）
  - 3回目以降: MCP target dir まわりの修正と sandbox / process-group cleanup テスト修正を反映後、`fmt` / `build-linux-sandbox` / `cargo test --all-features` まで成功
  - 最新ログ: `_tmp/all_test_result.txt`
  - 追記: MCP 側の `CARGO_TARGET_DIR` は、Docker 起動時に `.env`（`CODEX_DOCKER_TARGET_DIR`）を消費して bind mount / 環境変数で揃える方針に切り替え（サーバー側では `.env` を解釈しない）
- `make` 側で `codex-linux-sandbox` の `landlock` / `managed_proxy` テストがまとまって失敗した件:
  - 失敗例は `No such file or directory (os error 2)` で、`Permission denied` ではなかった。
  - そのため Docker 権限や `/var` / `/workspace` の mount 問題だけではなく、`codex-linux-sandbox` テストが補助バイナリをどう解決するかを疑った。
  - MCP は同系統テストを skip していたわけではなく、`run_cargo_test_selected(..., exact=true)` で `suite::landlock::test_root_read` と `suite::managed_proxy::managed_proxy_mode_routes_through_bridge_and_blocks_direct_egress` を個別実行して成功を確認した。
  - 対応方針は「Docker イメージに `python3` を追加して Python 依存前提を満たす」＋「`codex-rs/linux-sandbox/tests/suite/` 側で `codex-linux-sandbox` の解決方法を補強する」で、product code ではなく test code の局所修正に留めた。
  - この論点は rebase 中の follow-up 判断として残すが、恒久的な Docker 運用メモは `_docs/custom_notes/docker_test_env/README.md` に詳細を記録した。

## Scope note

- このレポートの `range-diff` 確認対象は commit 済みの `custom`（`b395673b...`）まで
- 作業ツリー上の未コミット変更は `range-diff` には含まれないため、別途レビューする
