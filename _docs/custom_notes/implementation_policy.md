# implementation_policy

## 目的

`fork-origin/main` の実装をできるだけ維持し、custom 再実装や rebase 時の衝突・挙動差・レビュー負荷を減らす。

## 基本方針

### 1. custom 実装の本体は `custom` と分かるファイルに置く

- 例: `src/custom/...`
- 例: `src/config/custom/...`
- 例: `custom_tests.rs`

理由:

- 上流ファイルへの差分を小さくする。
- custom の責務を見つけやすくする。
- rebase 時に「上流追従で見る場所」と「custom として維持する場所」を分ける。

### 2. 上流ファイルには接続点だけを置く

許容する接続点:

- `mod custom;`
- `pub(crate) custom: Custom...` のような集約フィールド
- custom helper の呼び出し
- 既存 request / params / context へ custom 用の最小フィールドを渡すこと

理由:

- 上流実装の流れを保つ。
- 衝突時に見る範囲を狭くする。
- custom 未設定時に本家挙動へ戻しやすくする。

### 3. 上流実装を custom 側にコピーしない

禁止例:

- 既存の spawn 処理を `custom_spawn.rs` に丸ごと複製する。
- 既存の stdio 設定を custom 側で再実装する。
- 既存の cleanup / lifecycle / retry / approval 処理を custom 側に複製する。

理由:

- 上流修正が custom コピーに反映されない。
- rebase 後に本家経路と custom 経路で挙動差が出る。
- 差分レビューで「何が custom の本質か」が見えなくなる。

### 4. custom helper は差分だけを担当する

よい例:

- worker user なら `setuid` / `setgid` と sudo fallback 用 `Command` 構築だけを custom 側に置く。
- env policy 分離なら、base policy を受け取って必要な override 済み policy を返すだけにする。

悪い例:

- custom helper が spawn 全体を実行する。
- custom helper が既存 runtime の approval / sandbox / stdio / cleanup を持つ。

理由:

- 本家の実行経路・エラー処理・I/O 処理を再利用する。
- custom は「差分の注入」に限定する。

### 5. fallback は明示的に許可されたものだけ入れる

fallback を入れる場合、浅い場所に専用ノートを作る。

必ず記録すること:

- fallback の目的
- fallback 後も守るべき安全境界
- 許可しない fallback
- 実装上、上流経路をどう維持するか

理由:

- 「便利だから fallback」ではなく、安全境界を保つための例外だと後から判断できるようにする。
- fallback が上流実装のコピーや責務拡大になっていないか確認できるようにする。

### 6. custom 設定は `custom.*` 名前空間に置く

例:

- `custom.user_shell.no_inject`

理由:

- 上流 config とキー衝突しにくくする。
- custom の削除・移植・比較を容易にする。

### 7. custom 状態は散らさず集約する

よい例:

- `permissions.custom`

悪い例:

- `permissions.custom_a`
- `permissions.custom_b`
- `permissions.custom_c`

理由:

- 上流構造体の形を極力維持する。
- custom の存在箇所を限定する。

### 8. 既存値を base として扱う

custom helper は、上流の既存設定・既存 policy を受け取り、必要な場合だけ上書きする。

理由:

- custom 未設定時に本家挙動へ自然に戻す。
- 上流の default / validation / lifecycle を維持する。

### 9. custom テストは分離する

例:

- `custom_tests.rs`
- `custom_*.rs`
- テスト名 `custom__feature__behavior`

理由:

- 上流テストとの差分を混ぜない。
- custom 回帰だけを探しやすくする。
- rebase 時に、上流テスト修正と custom テスト維持を分けて判断できる。

### 10. 上流ファイルを大きく触った場合は理由を記録する

記録場所:

- `_docs/custom_notes/...`

必ず書くこと:

- なぜ接続点だけでは足りないか
- どの上流処理を維持しているか
- rebase 時に確認すべき箇所

理由:

- 将来の作業者が「これは削れる差分か、必要な差分か」を判断できるようにする。

## 判断基準

次の質問に答えられない差分は、方針に合っていない可能性が高い。

- custom の本体は `custom` と分かるファイルにあるか。
- 上流ファイルの差分は接続点だけか。
- custom 側に上流実装のコピーがないか。
- custom 未設定時に本家挙動へ戻るか。
- fallback がある場合、許可理由と安全境界が浅い docs に記録されているか。
- rebase 時に確認すべきファイルが説明できるか。

## 関連ノート

- `_docs/custom_notes/custom_tests/README.md`
- `_docs/custom_notes/rebase_rules/README.md`
