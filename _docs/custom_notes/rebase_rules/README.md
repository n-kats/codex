# rebase_rules

## 目的

- `custom` ブランチの rebase を安定して再現できるようにする。

## 変更内容（何がどう変わるか）

- `fork-origin/main` に対する `custom` ブランチの rebase は、事前に squash を必須とする。
- 日付ブランチ（`YYYYMMDD`）を作成し、rebase 前の状態を固定して残す。
- `tmp-rebase` ブランチを作成して比較用に保持し、`git range-diff` で差分を確認する。
- rebase 前に差分量を確認し、大幅な設計変更が必要かどうかを確認する。
- 差分確認の結果は、必要に応じて報告し、設計変更が要る場合は先に相談して方針を決める。
- `git range-diff` の出力ログを保存する場合は、`_tmp/range-diff/` に置く。
- コンフリクト解消時は、過去の解消ログを最低 1 件は読んでから着手する。
- コンフリクト解消内容は、`_notes/deconflict/` に継続して記録する。

## 対象範囲（非対象も）

- 対象: `fork-origin/main` に対する `custom` ブランチの rebase 作業。
- 非対象: 他ブランチ間の rebase / merge、上流以外のベースへの rebase。

## 注意点（環境差・既知の制約）

- `tmp-rebase` が残っている場合は、前回作業が未完了と判断し作業を停止する。
- `fork-origin/main` の更新が必要な場合は、必要に応じて fetch してから進める。
- 通常は `git -c safe.directory=...` は不要。Git が安全確認で止まる場合のみ、リポジトリの所有権/権限を整える。
- コンフリクト解消ログは、解消時点の行番号でよい。後続の編集で行番号がずれても、解消時の記録を優先する。
- **テストが通るまで `--amend` / squash をしない**（コミットを書き換えない）。
  - rebase 後の修正は追加コミットで積み、手元のテストが通ったあとに必要ならまとめる。
- **`fork-origin/main` に無い（上流に存在しない）テストは `custom` 専用として分離する。**
  - 新規テストは「`custom` をファイル名に含む Rust ファイル」にのみ追加する（例: `custom_tests.rs` / `*_custom_*.rs`）。
  - 既存の上流テストファイルに、新規の `#[test]` / `#[tokio::test]` を追加しない（rebase の衝突を増やし、上流追従が困難になるため）。
  - テスト名も `custom__...` のように `custom` を明示して、`cargo test custom__` のように絞り込み実行できる状態を保つ。
  - 既存テストの修正が必要な場合でも、まず「上流の意図が変わったのか」「自分たちのカスタム仕様で変えたのか」を区別し、最小限の差分に抑える。

## 手順（コマンド）

0) 重要: `range-diff` は必ず目視レビューする（この手順の最重要項目）

- `git range-diff` の出力は**必ず人間が目視で確認**し、意図しない差分（ロジック欠落/余計な追加/削除）がないことを確認する。
- 目視レビューが終わるまで、rebase を「完了」と扱わない（push/共有/次の作業に進まない）。
- 重点確認ポイント:
  - `!`（内容が変わったパッチ）: 変更理由が「上流追従」か「意図したカスタム」かを説明できること
  - `<`/`>`（消えた/増えたパッチ）: 意図した削除・追加であること
  - 大量差分のときは `rg -n "[!<>]"` で該当行に絞ってから順に確認する

1) 日付ブランチを作成（既存なら作成済みでOK）

```sh
git checkout custom
git branch YYYYMMDD
```

2) `tmp-rebase` が残っていないことを確認（残っていたら作業停止）

```sh
git show-ref --verify --quiet refs/heads/tmp-rebase && exit 1
```

3) `custom` を squash（分岐点は `fork-origin/main` と `custom` の共通祖先）

```sh
git reset --hard YYYYMMDD
BASE=$(git merge-base fork-origin/main custom)
git reset --soft "$BASE"
git commit -m "custom changes"
```

4) `tmp-rebase` を作成（rebase 前の比較用）

```sh
git branch tmp-rebase
```

5) rebase 実行前に、過去のコンフリクト解消ログを最低 1 件読む

```sh
ls _notes/deconflict
# 少なくとも 1 ファイルは開いて読む（例）
sed -n '1,120p' _notes/deconflict/README.md
```

6) rebase 前に、差分量と設計影響を確認して報告する

```sh
git diff --stat fork-origin/main...custom
git log --oneline --left-right --cherry-mark fork-origin/main...custom
git diff --name-only fork-origin/main...custom
```

- 変更量が大きい場合は、rebase に入る前に「コンフリクト解消だけで済むか」「設計変更を先に入れるべきか」を確認する。
- API 変更、設定追加、実行経路の追加/変更が見える場合は、先に状況を報告し、必要なら相談して設計方針を確定する。
- ここで「相談が必要」と判断した場合は、rebase を進める前に止める。

7) rebase 実行（コンフリクト解消→記録→対象ファイルだけ add→continue）

```sh
git rebase fork-origin/main
# コンフリクト解消後
# _notes/deconflict/YYYYMMDD-<topic>.md へ記録
# 解消したファイルだけを個別に add する
git add <conflicted-path-1> <conflicted-path-2>
git rebase --continue
```

8) `range-diff` で比較（ベースが違うので明示指定）

```sh
BASE_TMP=$(git merge-base fork-origin/main tmp-rebase)
BASE_CUSTOM=$(git merge-base fork-origin/main custom)
git range-diff "$BASE_TMP"..tmp-rebase "$BASE_CUSTOM"..custom | tee "_tmp/range-diff/$(date +%Y%m%d)-rebase-range-diff.txt"
```

9) `range-diff` の目視レビューを完了する（必須）

- `!` と `<`/`>` の箇所を中心に、差分の意図を説明できるまで確認する。
- 不明点が残る場合は、修正してから再度 `range-diff` を取り直す。
- 保存したログは `_tmp/range-diff/` 配下のファイルを参照する。

10) カスタム仕様レポートを作成して表示する（必須）

- `range-diff` の目視レビューが終わったら、このフォークで維持している「カスタム仕様」が欠損していないことを、
  **人間が読める形式のレポート**として整理してから完了扱いにする。
- レポートは「どこに実装されているか」「どのテスト/検証で担保されているか」を、項目ごとに明記する。
- 形式（テンプレ）:
  - `- <カスタム項目名>`
    - 実装: `<path:line>`
    - テスト: `<path:line>`（無ければ「なし」）
    - 検証: `Makefile` ターゲット or 実行コマンド（手元環境）
- 例（抜粋）:
  - `CODEX_ADDITIONAL_PROMPT_DIRS`
    - 実装: `codex-rs/core/src/custom_prompts.rs:8`
    - テスト: `codex-rs/core/src/custom_prompts.rs:276`
    - 検証: `make verify-additional-prompt-dirs-env`
- 作成場所:
  - `_worklist/YYYYMMDD-rebase-followup.md`（今回の作業メモ）に追記するか、
  - `_worklist/YYYYMMDD-rebase-custom-report.md` のようなファイルとして作成する。
- 表示:
  - 作成したレポートを手元で開いて（エディタ/`cat` 等）、**目視で確認できる状態**にしてから次へ進む。

## 動作確認手順（手動・テスト・スナップショット）

- このルールは運用手順のため、原則としてエージェント側で動作確認コマンドは実行しない。
- ただし、MCP（ツール）経由で「make all 相当」を実行できる環境が用意されている場合は、次のツール呼び出しで代替できる:
  - `codex_dev_env/run_make_all_equivalent`
  - 注意: 実行環境側の制約により tool call がタイムアウトする場合がある（例: 600s）。その場合は再実行するか、より小さいターゲット（fmt / crate 単位 test など）に分割する。

## つまずきと対処（警告や失敗の修正）

- `tmp-rebase` が存在する: 直前の作業状態を確認し、必要なら手動で整理してから再開する。
- `range-diff` の差分が大きい: squash 前後の内容が一致しているかを確認し、意図しない変更があれば修正する。

## コンフリクト解消ログの書き方

- 保存先: `_notes/deconflict/YYYYMMDD-<topic>.md`
- 1 つの衝突ごとに、最低でも次を残す:
  - 対象ファイル
  - 解消時点の行番号
  - 採用した側（上流優先 / custom 維持 / 手動マージ）
  - 何をどう解消したか
- 例:

```md
# 20260303-rebase-custom.md

- File: `codex-rs/tui/src/chatwidget.rs`
  - Line: 3415
  - Resolution: upstream 優先 + custom の `service_tier` 追従を再適用
  - Note: `InputResult::Submitted` の処理は上流に合わせ、`Op::UserTurn` の追加フィールドだけ残した
```

## 関連ファイル一覧

- `CUSTOM.md`
- `_notes/deconflict/README.md`

## 20260722 スクリプトのブランチ作成順序

- 対象: `scripts/rebase_custom_workflow.sh`
- 変更内容:
  - `custom` の rebase 前・squash 前の状態を `YYYYMMDD` ブランチに保存する。
  - `custom` を squash した後、その状態から `tmp-rebase` を作成する。
  - その後に `custom` を `fork-origin/main` へ rebase する。
- 注意点:
  - 同名の日付ブランチまたは `tmp-rebase` が存在する場合は、既存状態を上書きせず停止する。
- 検証手順:
  - 手元環境で `custom` を clean にしたうえでスクリプトを実行する。
  - `git log --graph --oneline --decorate --all` で、日付ブランチが squash 前、`tmp-rebase` が squash 後・rebase 前を指していることを確認する。
- 関連ファイル:
  - `scripts/rebase_custom_workflow.sh`
