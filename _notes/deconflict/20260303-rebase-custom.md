# 20260303-rebase-custom

- File: `codex-rs/app-server/src/codex_message_processor.rs`
  - Line: 1801
  - Resolution: 手動マージ（上流追従 + custom 維持）
  - Note: 上流の config 保存経路まわりの更新を採用しつつ、user config の実ファイルパスを引く `user_config_toml_path()` と `with_config_path(...)` を残した。`project_doc_paths: None` も追従した。

- File: `codex-rs/cli/src/mcp_cmd.rs`
  - Line: 210
  - Resolution: 手動マージ（上流追従 + custom 維持）
  - Note: 上流の MCP CLI 構造変更に合わせつつ、`config_toml_file` / `no_config` / `LoaderOverrides` を使う custom の config 読み分けを残した。書き込み系は user config layer が無いときに止める分岐で合わせた。

- File: `codex-rs/core/src/codex.rs`
  - Line: 3939
  - Resolution: 手動マージ（上流追従 + custom 維持）
  - Note: 上流の session 更新処理を維持しつつ、`project_doc_paths` 更新時に `/custom-agents` 相当のパス検証と `user_instructions` 再計算を入れた。`exec_run_as` の伝播と `custom_tests` の読み込みも残した。

- File: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
  - Line: 639
  - Resolution: custom 維持
  - Note: shell escalation 実行器に `run_as: Option<RunAsUser>` を持たせ、zsh fork と各 executor 呼び出しへ `run_as` を引き回す差分を維持した。

- File: `codex-rs/tui/src/diff_render.rs`
  - Line: 170
  - Resolution: custom 維持
  - Note: diff 表示テーマの custom override（`DiffPaletteOverride`、`set_diff_palette_override()`、各 style 関数での有効/無効分岐）を維持した。上流の描画ロジックは極力そのままにし、色決定の直前だけ差し込んだ。

- File: `codex-rs/mcp-server/src/lib.rs`
  - Line: n/a
  - Resolution: 上流優先
  - Note: rebase 中の衝突対象には含まれていたが、最終的に `fork-origin/main` に対する保持差分は残っていない。解消時の正確な行番号は今回の再構成では特定していない。

- File: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - Line: 2737
  - Resolution: 上流追従
  - Note: `find_builtin_command(...)` のシグネチャ変更に合わせ、複数の `bool` 引数ではなく `self.builtin_command_flags()` を渡す形に戻した。`/custom-agents` の plain Enter 例外分岐はそのまま維持した。

- File: `codex-rs/tui/src/diff_render/custom_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: `diff_render.rs` のシグネチャ更新に追従し、`ResolvedDiffBackgrounds` と `fallback_diff_backgrounds(...)` を使う呼び出しへ合わせた。これは rebase 後の API ずれ修正で、custom 専用テスト側で吸収した。

- File: `codex-rs/app-server/src/codex_message_processor.rs`
  - Line: 1443
  - Resolution: custom 維持
  - Note: 上流側には同名メソッド追加が無かったため、`get_user_saved_config` / `get_user_info` / `set_default_model` の custom 追加をそのまま残し、衝突マーカーのみ除去した。
