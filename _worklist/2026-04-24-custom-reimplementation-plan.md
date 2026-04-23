# 2026-04-24 custom 機能の再実装順

本家に戻したあと、custom 機能を一つずつ再実装するときの優先順。

## 方針

- 先に土台を戻す。
- 次に実行系と入力系を戻す。
- 最後にテスト・検証系を固める。
- 依存が強いものはまとめて進める。

## 順番

1. `config_toml_read_control`
   - `--config <FILE>` / `--no-config` を先に戻す。
   - 以後の設定系の基盤になるため最優先。

2. `codex_home_cli_flag` / `codex_memory_cli_flag`
   - 保存先の根本を固定する。
   - `config.toml`、ログ、memories の出力先が安定するようにする。

3. `additional_prompt_dirs` / `agents_md_and_custom_agents_restore`
   - プロンプト探索と project doc の復元を戻す。
   - 起動時の参照先を安定させる。

4. `exec_command_default_login` / `linux_default_shell_prefers_bash_over_zsh`
   - shell 起動の再現性を戻す。
   - dotfiles 差で揺れるテストを減らす。

5. `command_exec_worker_user` / `user_shell_environment_policy_split` / `user_shell_no_inject`
   - コマンド実行の権限分離と `!` の扱いをまとめて戻す。
   - 相互依存が強いので別々に引かない。

6. `tui-enter-newline-ctrl-enter-send`
   - ユーザー入力の基本挙動を戻す。
   - ただし前段の shell / exec 基盤が先に必要。

7. `update_check_custom_version_suffix` / `custom_theme_diff_colors` / `tui_auth_onboarding_alignment`
   - TUI の見た目・導線・更新通知を戻す。
   - snapshot 差分を読みやすくするため、基盤後に進める。

8. `tui_remote_alignment` / `tui_app_server_removal`
   - remote 系の整合を戻す。
   - 旧 UI の残骸を引きずりやすいので後半に回す。

9. `unified_exec_end_event_deterministic` / `exec_server_tests_dotslash`
   - 実行イベント順序とテスト環境依存を潰す。
   - 実装とテストの両方を安定させる。

10. `shell_snapshot_redacted_exports` / `test_output_redacts_host_env` / `tool_parallelism_test` / `test_almost_skip_list` / `custom_tests`
    - 回帰防止と検証基盤を最後に固める。
    - 途中で入れてもよいが、最終的にはここを整える。

11. `release_versioning` / `langfuse_logging` / `hooks`
    - 独立度が高いか、優先度が低いもの。
    - 他が戻った後でよい。

## 実務メモ

- 先に「設定・保存先・起動シェル」を戻す。
- 次に「実行権限・入力」を戻す。
- その後に「TUI・remote」を戻す。
- 最後に「テスト・運用補助」を固める。

