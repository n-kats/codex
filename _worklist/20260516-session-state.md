# 20260516 session state

## 現在地

- `custom` ブランチは `fork-origin/main` へ rebase 済み。
- 比較用退避は `tmp-rebase` として残してある。
- `almost` 相当の実行は 2 回試し、1 回目は `cli/Cargo.toml` の重複依存、2 回目は config / handler の残件で止まった。
- 途中で得た考察は `_worklist/20260516-rebase-custom-report.md` に追記済み。

## 変わらない方針

- 上流の新しい責務配置を優先し、その形へ custom を寄せる。
- custom 機能は消さない。実装の置き場所や型は上流に合わせ、動作は custom 専用テストで守る。
- custom 専用テストは `custom__...` 命名と custom ファイル分離を維持する。
- 既存の上流テストファイルへ新規 custom テストを混ぜない。

## 詳細な判断基準

- `LoaderOverrides.user_config_path`
  - `PathBuf` のまま残すのではなく、`AbsolutePathBuf` に寄せる。
  - 理由: 相対パスの解決が CLI と loader にまたがると、`--config` の意味が cwd に依存してぶれやすい。
  - 正しい持ち方: CLI で cwd 基準に絶対化し、loader には解決済みパスだけを渡す。
  - 守るべき custom 仕様: `--config` は引き続き任意のファイルを指せること、`--no-config` は user/project config を読まないこと。

- `ConfigLayerSource::User`
  - base layer と profile layer を区別した上流の形に合わせる。
  - base は `profile: None`、profile layer は `profile: Some(name)`。
  - `--config` は base user config の差し替えとして扱う。
  - 別 variant を足して回避するのは避ける。provenance が崩れて、後で `range-diff` とテストの両方が不安定になる。

- `ignore_user_config` と `disable_user_config`
  - `ignore_user_config` は上流が持っている意味を優先する。
  - `disable_user_config` は custom の `--no-config` のために残す。
  - 2 つを同義にすると、`--no-config` なのに metadata が残る、または逆に上流が期待する layer 情報が消える。

- `unavailable_tool`
  - ファイルを復活させるのは最終手段。
  - まず現行の handler / registry が「使えない tool」をどう表現しているかを見る。
  - 上流が整理した責務に custom を合わせる方が、次回以降の rebase コストが低い。

- custom テスト
  - 新規テストは `custom` を含むファイルに閉じる。
  - テスト名は `custom__...` に寄せる。
  - 目的は custom 機能の保守ではなく、上流追従後も custom の仕様を機械的に検出できるようにすること。

## 何を避けるか

- その場しのぎの型変換で `PathBuf` と `AbsolutePathBuf` を往復させること。
- loader 内で cwd 解決を増やして、CLI と config の責務を混ぜること。
- `profile` を埋めないまま `ConfigLayerSource::User` を作ること。
- 消えた module を「とりあえず復活」させること。
- custom のテストを既存の上流テストファイルへ混ぜること。

## 直す対象の考え方

- `LoaderOverrides.user_config_path`
  - `Option<AbsolutePathBuf>` を正とする。
  - CLI から入る `--config` は cwd 基準で絶対化してから loader へ渡す。
  - `PathBuf` のまま残すと、相対解決責務が分散して再現性が落ちる。

- `ConfigLayerSource::User`
  - base layer は `profile: None`。
  - profile layer は `profile: Some(profile_name)`。
  - `--config` は base user config の file 差し替えとして扱う。

- `ignore_user_config` と `disable_user_config`
  - 役割が違うので混ぜない。
  - `ignore_user_config` は上流の意味を壊さずに残す。
  - `disable_user_config` は custom の `--no-config` に対応するスイッチとして残す。

- `unavailable_tool`
  - ファイルを復活させる前提ではなく、現行 handler / registry の責務に合わせて扱う。
  - 上流の tool handler 構成に custom を接続する形を優先する。

## 直近の失敗ログ

- 1 回目の `make almost`:
  - `codex-rs/cli/Cargo.toml` に `codex-api` の重複キーがあった。

- 2 回目の `make almost`:
  - `codex-rs/config/src/state.rs`
    - `user_config_path` が `AbsolutePathBuf` と `PathBuf` の二重定義になっていた。
    - これは rebase 衝突の取り残しで、実装の方向性としては `AbsolutePathBuf` に揃えるべきだった。
  - `codex-rs/config/src/loader/mod.rs`
    - `ConfigLayerSource::User` の `profile` が不足していた。
    - これは上流の新しい provenance 形式に custom 側が追いついていない状態。
  - `codex-rs/core/src/tools/handlers/mod.rs`
    - `mod unavailable_tool;` が残っているが、対応ファイルが見当たらない。
    - これは実装の欠落ではなく、上流の tool handler 再編に custom がまだ合わせ切れていないことを示す。

## 次にやることの考え方

- `state.rs` と `loader/mod.rs` は、上流の config contract に custom を吸収する方向で直す。
- `core/src/tools/handlers/mod.rs` は、参照先が消えた理由を上流の再編から説明できるか確認する。
- その後に `make almost` を再実行し、残りの失敗を一件ずつ同じ基準で潰す。
- この流れであれば、`custom` の機能を残しつつ本家追従も保てる。

## 全文記録

以下は、再開時に判断をそのまま復元できるように残す完全版の要点記録。

### 見るべき軸

- 見るべき軸は「コンパイルを通す」ではなく、`custom` の意図を上流の新しい責務配置に乗せ直せているか。
- 今回ぶれやすいのは主に `config loading` と `tool handler` 周辺。

### 判断基準

- `custom` の独自仕様は維持する。
- ただし、上流が同じ概念をより正式な型や責務に昇格している場合は、custom 側の古い実装形を残さず、上流の形へ吸収するのが正解。
- つまり、次の 4 点が原則。
  - `custom` の機能要件は残す
  - 実装の型・経路・責務配置は上流を優先する
  - custom の古い補助フィールドや迂回経路は、上流の新 surface に置き換える
  - テストは custom 専用ファイル・`custom__...` 名で維持する

### config 周辺の結論

- `LoaderOverrides.user_config_path` は上流の `Option<AbsolutePathBuf>` を正にするのが妥当。
- 理由は、上流が `ConfigLayerSource::User { file, profile }` や `LoaderOverrides::user_config_path(&self, codex_home)` のように、config layer の identity を「解決済み absolute path + profile」として扱う方向に進んでいるから。
- ここで custom の `Option<PathBuf>` を残すと、相対パス解決責務が loader 内外に分散して、`--config` の再現性が落ちる。
- custom の正しい維持内容は次のとおり。
  - `--config FILE` は引き続き任意の `config.toml` を指定できる
  - 相対パスなら CLI の cwd 基準で絶対化する
  - loader へ渡す時点では `AbsolutePathBuf`
  - `--no-config` は user config と project config を読まない
  - auth や `CODEX_HOME` 自体は潰さない
- したがって、`PathBuf` フィールドを残すのではなく、CLI 側で `resolve_path_from_cwd` した後に `AbsolutePathBuf::from_absolute_path(...)` へ変換して入れるのが正しい。

### `ignore_user_config` と `disable_user_config`

- ここは特にぶれやすい。
- 上流の `ignore_user_config` は「ユーザー config の内容は読まないが、user layer の存在や folder-derived resources のための metadata は残す」意味に見える。
- 一方 custom の `disable_user_config` は `--no-config` のための「user config layer 自体を使わない」に近い。
- この 2 つを混ぜると、`--no-config` なのに `$CODEX_HOME/config.toml` 由来の hooks/rules/metadata が残る、または逆に上流が期待する metadata が消える、というバグになる。
- 正解は分けること。
  - `ignore_user_config`: 上流の意味を維持
  - `disable_user_config`: custom の `--no-config` 用に維持
  - `--config FILE`: `disable_user_config = false` で、user layer の file だけ差し替える
  - `--no-config`: `disable_user_config = true` かつ `disable_project_config = true`

### `ConfigLayerSource::User`

- base config layer は `ConfigLayerSource::User { file: user_file.clone(), profile: None }`。
- profile config layer は `ConfigLayerSource::User { file: profile_file.clone(), profile: Some(profile.to_string()) }`。
- custom の `--config FILE` は「base user config の file を差し替える」扱いなので、`profile: None` が自然。
- ここで custom 用の別 variant や `profile: Some("custom")` のような値を作るべきではない。config provenance が嘘になる。

### `unavailable_tool`

- これは単に `mod` を消すか復活するかではなく、「上流が tool registration の失敗や unavailable tool をどう表現するようになったか」を見る必要がある。
- 正しい判断の順序は次のとおり。
  1. 上流に `unavailable_tool` 相当の責務が別 module に移ったか確認する
  2. custom の MCP/tool mutation がまだその概念を必要としているか確認する
  3. 必要なら上流の新しい abstraction に custom 差分を接続する
  4. 不要なら `mod unavailable_tool;` と関連 import を削除する
- ファイルだけ復活させるのは避けたい。上流が tool executor / handler registry 周りを整理しているなら、古い module を戻すと次回 rebase でまた割れる。

### テストの維持

- `--codex-home`: CLI parse と bootstrap env
- `--codex-memory`: CLI parse と bootstrap env
- `custom.user_shell.no_inject`: `custom_user_shell_cmd.rs`
- `custom.exec.worker_user`: `custom_exec_command_worker_user.rs`
- `CODEX_SHELL_STARTUP_FILES`: CLI parse と `shell_startup_files/custom_tests.rs`
- `CODEX_ADDITIONAL_PROMPT_DIRS`: `tui/src/custom_prompts.rs` の custom tests
- `almost` が通ることは最終ゲートだが、custom 仕様の保証としては `custom__` テストが残っていて、上流テストファイルへ混ざっていないことも確認対象。

### 今回の修正方針の結論

- `LoaderOverrides.user_config_path` は上流の `Option<AbsolutePathBuf>` に統一
- custom CLI の `--config` は cwd 基準で絶対化して `AbsolutePathBuf` として渡す
- `disable_user_config` / `disable_project_config` は custom の `--no-config` 意味として残す
- `ignore_user_config` は上流の意味を壊さない
- `ConfigLayerSource::User` は必ず `profile` を埋める
- `unavailable_tool` は復活前提にせず、上流の現行 tool handler 構造に custom を合わせる
- 修正後は `custom__...` テストを重点的に通し、その後 `almost` を通す

### 追記の確認点

- `user_config_profile` の扱いも再開時に確認すること
- `ConfigLayerSource::User` の変更は `state_tests.rs` など provenance / layer 選択テストにも波及すること
- `cli/Cargo.toml` の `codex-api` 重複は既に解消済みであること
- `unavailable_tool` は「どの module に移ったか」「本当に必要か」を上流構造から確認すること

## 参照ファイル

- `_worklist/20260516-rebase-custom-report.md`
- `_notes/deconflict/20260516-rebase-custom.md`
- `_tmp/range-diff/20260516-rebase-range-diff.txt`
