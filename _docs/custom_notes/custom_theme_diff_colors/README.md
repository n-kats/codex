# custom_theme_diff_colors

## 目的

- Codex TUI の差分表示色（追加/削除の背景色）を `config.toml` から調整できるようにする。

## 変更内容（何がどう変わるか）

- `config.toml` に以下を追加。

```toml
[custom.theme.diff]
add_line_bg = "#102030"
del_line_bg = "#402010"
```

- さらに、色付けの有効/無効を制御できるフラグを追加。

```toml
[custom.theme.diff]
# 全体の有効化（default: true）
enabled = true

# 部位ごとの有効化（default: true）
line_bg = true
gutter = true
sign = true
content = true
```

- `core` で `custom.theme.diff` を読み取り、`Config` に RGB として保持。
- `tui` の diff renderer にグローバル上書きパレットを追加し、起動時に `Config` から反映。
- `enabled = false` で diff 色付け全体を無効化できる。
- `line_bg` / `gutter` / `sign` / `content` で部分的に無効化できる。

## 対象範囲（非対象も）

- 対象:
  - TUI の差分表示（追加/削除行の背景色）
- 非対象:
  - シンタックスハイライトテーマ (`tui.theme`)
  - 他 UI 要素の色

## 注意点（環境差・既知の制約）

- 色指定は `#RRGGBB`（`RRGGBB` も可）の 6 桁 hex のみ。
- 不正な値は config 読み込みエラーになる。
- 現在は diff 色付け（背景・ガター・記号・非syntax本文）の on/off を制御可能。
- `content = false` は diff 固有スタイルを外す。syntax highlight 自体は別機能。

## 動作確認手順（手動・テスト・スナップショット）

- 手元環境で実施:
  - `cd codex-rs && cargo test -p codex-core custom_theme_diff`
  - `cd codex-rs && cargo test -p codex-tui diff_palette_override`
- 手動確認:
  - 上記 `custom.theme.diff` を設定して TUI を起動し、差分表示の背景色が変わることを確認。
  - `enabled = false` で差分の色付けが無効になることを確認。
  - 各フラグを切り替えたときに対象部位のみ色が消えることを確認。

## つまずきと対処（警告や失敗の修正）

- TUI 側だけで処理すると設定バリデーションが遅れるため、`core` で hex を検証して `Config` に解決済み値を渡す構成にした。

## 関連ファイル一覧

- `codex-rs/core/src/config/mod.rs`
- `codex-rs/tui/src/diff_render.rs`
- `codex-rs/tui/src/lib.rs`
