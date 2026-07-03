# 2026-06-19 custom 追加テスト棚卸し

`custom` ブランチで追加されたテストを、再実装する custom 機能との対応関係で整理する。

確認元:

- 比較元: `$(git merge-base custom fork-origin/main)` = `cdde711fac008cd4e1115603ead713cf23b1a580`
- 比較先: `custom` = `174e84007a1b6e4b40685703fd5924f4c3611f64`

抽出範囲:

- Rust 追加テスト関数: 165 件
- Python 追加テスト関数: 1 件
- Makefile 検証ターゲット
- snapshot 期待値の追加/更新
- テスト実行基盤

## 対応表

| custom 機能 | 関連テスト/検証 |
|---|---|
| `config_toml_read_control` | `--config-file` / `--no-config-file` CLI parse、loader override、alternate config path、project config disable |
| `codex_home_cli_flag` | `--codex-home` CLI parse、bootstrap env、Makefile debug `CODEX_HOME` |
| `codex_memory_cli_flag` | `--codex-memory` CLI parse、bootstrap env、Makefile debug `CODEX_MEMORIES_HOME` |
| `agents_md_and_custom_agents_restore` | `--agents-md` parse、thread metadata update、project doc path override、custom agents slash command、model-visible layout |
| `exec_command_default_login` / `shell_startup_files` | `--shell-startup-files` CLI parse、bootstrap env、clean/default parse、zsh `ZDOTDIR` isolation |
| `user_shell_no_inject` | `custom.user_shell.no_inject` config、startup warning、rollout 非記録 |
| `custom_theme_diff_colors` | TOML deserialize、hex color parse、不正 hex rejection、snapshot |
| `update_check_custom_version_suffix` | custom suffix 付き semver parse/compare |
| `tui_remote_alignment` | websocket remote addr normalize、remote auth token transport policy |
| `custom_tests` 運用 | `cargo test custom__`、`list-custom-tests`、custom test file separation |
| `docker_test_env` / `make_test_almost_no_fail_fast` / `skip_test_list.txt` | Docker 検証環境、`make almost`、`test-almost`、skip list、ログ集約 |
| `exec_mcp_*` | MCP 経由の build/test 相当、background stop、almost 相当 |
| 本家追従/再実装対象外候補 | app-server/thread/config/TUI など、custom 固有名ではない追加テスト |

## `config_toml_read_control`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/cli/src/custom_tests.rs` | `custom__config_toml_read_control__config_toml_file_flag_is_global` | `codex exec --config-file alt.toml` が global 引数として解釈される。 | `parse cli; assert shared.config_toml_file == "alt.toml"` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__config_toml_read_control__config_toml_file_conflicts_with_no_config` | `--config-file` と `--no-config-file` を同時指定できない。 | `parse error; assert ArgumentConflict` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__config_toml_read_control__build_loader_overrides_no_config_disables_user_and_project` | `--no-config-file` で user/project config を無効化する。 | `build overrides; assert no_user_config && no_project_config` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__config_toml_read_control__build_loader_overrides_config_sets_user_config_path` | `--config-file` を user config path として絶対パス化する。 | `build overrides; assert config_toml_file == absolute(path)` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__config_toml_read_control__helpに表示される` | help に `--config-file` / `--no-config-file` などが出る。 | `render help; assert options present` |
| `codex-rs/exec/src/custom_tests.rs` | `custom__config_toml_read_control__config_toml_file_flag_is_global` | `codex-exec resume --config-file alt.toml --last ...` が parse できる。 | `parse exec cli; assert config_toml_file` |
| `codex-rs/exec/src/custom_tests.rs` | `custom__config_toml_read_control__config_toml_file_conflicts_with_no_config` | `codex-exec` でも `--config-file` と `--no-config-file` は conflict。 | `parse error; assert ArgumentConflict` |
| `codex-rs/core/src/config/config_loader_tests.rs` | `user_config_path_override_loads_alternate_file` | user config path override で別 config を読む。 | `load(user_config_path=alt); assert alt config used` |
| `codex-rs/core/src/config/config_loader_tests.rs` | `disable_project_config_omits_project_layers` | project config disable で project layer を読まない。 | `load(no_project_config=true); assert project layers omitted` |
| `codex-rs/tui/src/app/config_persistence.rs` | `rebuild_config_for_resume_or_fallback_preserves_user_config_path` | resume/fallback 用 rebuild で user config path を保持する。 | `rebuild config; assert user_config_path preserved` |
| `codex-rs/tui/src/lib.rs` | `run_tui_config_loader_applies_loader_overrides_to_final_config` | TUI 起動時の config loader override が最終 config に反映される。 | `run loader with overrides; assert final config uses override` |

## `codex_home_cli_flag`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/cli/src/custom_tests.rs` | `custom__codex_home_cli_flag__flag_is_global` | `--codex-home` が global 引数として解釈される。 | `parse cli; assert codex_home == path` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__codex_home_cli_flag__bootstrap_sets_code_home_env` | bootstrap 引数から `CODEX_HOME` を設定する。 | `bootstrap_env; assert CODEX_HOME == path` |
| `Makefile` | `verify-codex-home-cli-flag` | `--codex-home` 関連の fmt/lint/test をまとめて実行する。 | `run fmt, lint/test arg0, lint/test cli` |

## `codex_memory_cli_flag`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/cli/src/custom_tests.rs` | `custom__codex_memory_cli_flag__flag_is_global` | `--codex-memory` が global 引数として解釈される。 | `parse cli; assert codex_memory == path` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__codex_memory_cli_flag__bootstrap_sets_code_memory_env` | bootstrap 引数から `CODEX_MEMORIES_HOME` を設定する。 | `bootstrap_env; assert CODEX_MEMORIES_HOME == path` |

## `agents_md_and_custom_agents_restore`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/cli/src/custom_tests.rs` | `custom__agents_md_restore__flag_is_global` | 対話モードで `--agents-md` を global 引数として parse する。 | `parse "codex --agents-md file"; assert agents_md` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__agents_md_restore__flag_is_global_for_exec` | exec モードで `--agents-md` を global 引数として parse する。 | `parse "codex exec --agents-md file"; assert agents_md` |
| `codex-rs/cli/src/main.rs` | `agents_md_is_preserved_for_interactive_resume` | interactive resume で `agents_md` を保持する。 | `start with agents_md; resume; assert preserved` |
| `codex-rs/app-server/tests/suite/v2/thread_metadata_update.rs` | `thread_metadata_update_replaces_agents_md_in_next_turn_request` | thread metadata update の `agents_md` が次 turn request に反映される。 | `update agents_md; next turn; assert request contains new docs` |
| `codex-rs/core/src/agents_md_tests.rs` | `explicit_project_doc_paths_override_auto_discovery` | 明示 project doc paths が `AGENTS.md` 自動探索を上書きする。 | `set explicit docs; assert auto discovery not used` |
| `codex-rs/core/src/config/config_tests.rs` | `load_config_loads_global_agents_instructions` | global `AGENTS.md` instructions を config load で読む。 | `write global AGENTS.md; load; assert included` |
| `codex-rs/core/src/config/config_tests.rs` | `load_config_prefers_global_agents_override_instructions` | global agents override が通常 global `AGENTS.md` より優先される。 | `set override and file; assert override chosen` |
| `codex-rs/core/src/session/tests.rs` | `override_turn_context_reinjects_custom_agents_into_next_turn_context` | custom agents override を次 turn context に再注入する。 | `build next context; assert custom docs included` |
| `codex-rs/core/src/session/tests.rs` | `session_update_settings_clears_reference_context_item_when_project_doc_paths_change` | project doc paths 変更時に古い reference context を消す。 | `update docs; assert old reference removed` |
| `codex-rs/core/src/session/tests.rs` | `session_update_settings_reinjects_project_doc_paths_into_initial_context` | project doc paths 更新後に initial context へ再注入する。 | `update docs; assert new docs injected` |
| `codex-rs/core/src/session/tests.rs` | `session_update_settings_rejects_invalid_project_doc_paths` | invalid project doc paths を拒否する。 | `update invalid path; assert error` |
| `codex-rs/core/tests/suite/model_visible_layout.rs` | `snapshot_model_visible_layout_custom_agents_override_replaces_agents_md` | custom agents override が通常 `AGENTS.md` を置換することを snapshot で確認する。 | `build model-visible snapshot; assert override text` |
| `codex-rs/tui/src/chatwidget/tests/slash_commands.rs` | `queued_slash_custom_agents_with_args_updates_project_doc_paths` | `/custom-agents ...` が project doc paths を更新する。 | `queue command; assert doc paths == args` |
| `codex-rs/tui/src/chatwidget/tests/slash_commands.rs` | `queued_slash_custom_agents_clear_restores_auto_discovery` | `/custom-agents clear` が override を解除する。 | `set override; clear; assert auto discovery restored` |
| `codex-rs/tui/src/chatwidget/tests/slash_commands.rs` | `queued_slash_custom_agents_emits_visible_log_with_resolved_path` | `/custom-agents` 実行時に解決済みパスを visible log に出す。 | `run command; assert visible log contains absolute path` |

## `exec_command_default_login` / `shell_startup_files`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/cli/src/custom_tests.rs` | `custom__shell_startup_files_cli_flag__flag_is_global` | `--shell-startup-files clean` が global 引数として parse される。 | `parse cli; assert shell_startup_files == clean` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__shell_startup_files_cli_flag__equals_form_is_global` | `--shell-startup-files=clean` も parse される。 | `parse cli; assert clean` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__shell_startup_files_cli_flag__bootstrap_sets_shell_startup_files_env` | bootstrap で `CODEX_SHELL_STARTUP_FILES` を設定する。 | `bootstrap env; assert env == clean` |
| `codex-rs/cli/src/custom_tests.rs` | `custom__shell_startup_files_cli_flag__bootstrap_sets_shell_startup_files_env_from_equals_form` | 等号形式でも bootstrap env を設定する。 | `bootstrap env; assert env == clean` |
| `codex-rs/core/src/shell_startup_files/custom_tests.rs` | `custom__シェル起動ファイル__未指定はdefaultとして解釈する` | 未指定/default/未知値を `Default` として扱う。 | `parse None/default/unknown; assert Default` |
| `codex-rs/core/src/shell_startup_files/custom_tests.rs` | `custom__シェル起動ファイル__clean指定を解釈する` | `clean` を大小文字・空白込みで `Clean` として扱う。 | `parse " CLEAN "; assert Clean` |
| `codex-rs/core/src/shell_startup_files/custom_tests.rs` | `custom__シェル起動ファイル__cleanはzshのみ隔離する` | clean mode は zsh にだけ空 `ZDOTDIR` を注入する。 | `apply clean bash/zsh; assert only zsh has ZDOTDIR` |
| `Makefile` | `verify-exec-command-default-login` | app-server exec、exec args、startup files、CLI flag をまとめて検証する。 | `cargo test selected exec/default-login tests` |
| `Makefile` | `verify-linux-default-shell` | shell 検出と shell snapshot lifecycle 系を検証する。 | `cargo test selected shell tests` |

## `user_shell_no_inject`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/core/src/config/config_tests.rs` | `custom_user_shell_no_inject_is_resolved` | `[custom.user_shell] no_inject = true` を解決する。 | `load config; assert user_shell_no_inject()` |
| `codex-rs/core/src/config/config_tests.rs` | `custom_user_shell_no_inject_false_adds_startup_warning` | `no_inject = false` 明示時に startup warning を出す。 | `load false; assert warning` |
| `codex-rs/core/src/config/config_tests.rs` | `custom_user_shell_no_inject_default_adds_startup_warning` | 未指定時にも startup warning を出す。 | `load default; assert warning` |
| `codex-rs/core/tests/suite/custom_user_shell_cmd.rs` | `custom__user_shell_no_inject__bang_result_not_recorded_locally` | `!` コマンドを rollout に `<user_shell_command>` として記録しない。 | `run !; read rollout; assert command absent` |

## `custom_theme_diff_colors`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/core/src/config/config_tests.rs` | `custom_theme_diff_deserializes_from_toml` | `custom.theme.diff` を TOML から deserialize する。 | `parse toml; assert diff fields populated` |
| `codex-rs/tui/src/diff_render.rs` | `custom_diff_theme_override_parses_hex_colors` | hex color を parse する。 | `parse colors; assert RGB values` |
| `codex-rs/tui/src/diff_render.rs` | `custom_diff_theme_override_rejects_invalid_hex` | 不正 hex color を拒否する。 | `parse invalid; assert error` |

## `update_check_custom_version_suffix`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/tui/src/updates.rs` | `custom_suffix_versions_are_comparable_against_plain_semver` | `1.2.3-custom` を plain semver と比較できる。 | `parse custom suffix; compare with latest` |
| `codex-rs/tui/src/updates.rs` | `extract_version_from_brew_api_json` | brew API JSON から version を抽出する。 | `json; assert version` |
| `codex-rs/tui/src/updates.rs` | `extracts_version_from_latest_tag` | latest tag から version を抽出する。 | `tag v1.2.3; assert 1.2.3` |
| `codex-rs/tui/src/updates.rs` | `latest_tag_without_prefix_is_invalid` | prefix なし latest tag を invalid とする。 | `tag 1.2.3; assert invalid` |
| `codex-rs/tui/src/updates.rs` | `plain_semver_comparisons_work` | plain semver 比較を確認する。 | `compare 1.2.3 and 1.2.4` |
| `codex-rs/tui/src/updates.rs` | `prerelease_version_is_not_considered_newer` | prerelease を newer と見なさない。 | `latest beta; assert not newer` |
| `codex-rs/tui/src/updates.rs` | `whitespace_is_ignored` | version 文字列の空白を無視する。 | `parse " 1.2.3 "; assert 1.2.3` |

## `tui_remote_alignment`

| パス | テスト | 内容 | 疑似コード |
|---|---|---|---|
| `codex-rs/tui/src/lib.rs` | `normalize_remote_addr_accepts_websocket_url` | `ws://127.0.0.1:4500` を受け入れる。 | `normalize ws loopback; assert URL` |
| `codex-rs/tui/src/lib.rs` | `normalize_remote_addr_accepts_secure_websocket_url` | `wss://example.com:443` を受け入れる。 | `normalize wss; assert URL` |
| `codex-rs/tui/src/lib.rs` | `normalize_remote_addr_rejects_websocket_url_without_explicit_port` | port なし websocket URL を拒否する。 | `normalize ws no port; assert error` |
| `codex-rs/tui/src/lib.rs` | `normalize_remote_addr_rejects_invalid_input` | websocket 以外の URL を拒否する。 | `normalize https; assert error` |
| `codex-rs/tui/src/lib.rs` | `normalize_remote_addr_rejects_host_port_shortcut` | `host:port` 短縮形を拒否する。 | `normalize host:port; assert error` |
| `codex-rs/tui/src/lib.rs` | `remote_auth_token_transport_accepts_loopback_ws` | auth token transport で loopback `ws://` を許可する。 | `validate ws loopback; assert ok` |
| `codex-rs/tui/src/lib.rs` | `remote_auth_token_transport_accepts_secure_wss` | auth token transport で `wss://` を許可する。 | `validate wss; assert ok` |
| `codex-rs/tui/src/lib.rs` | `remote_auth_token_transport_rejects_non_loopback_ws` | non-loopback `ws://` を拒否する。 | `validate ws example.com; assert error` |
| `codex-rs/app-server-client/src/lib.rs` | `remote_auth_token_transport_policy_allows_wss_and_loopback_ws` | client 側 policy でも `wss://` と loopback `ws://` を許可する。 | `assert allow wss/loopback; reject non-loopback ws` |

## `custom_tests` / テスト運用

| パス | テスト/ターゲット | 内容 | 疑似コード |
|---|---|---|---|
| `Makefile` | `test-custom` | `cargo test custom__` で custom 命名テストを一括実行する。 | `cargo test custom__` |
| `Makefile` | `list-custom-tests` | `custom__` 接頭辞の Rust テスト関数を一覧する。 | `rg "fn custom__" | sort` |
| `Makefile` | `verify-all-custom` | custom verification をまとめて続行実行する。 | `run custom verify targets` |

## `docker_test_env` / `make almost` / 実行基盤

| パス | 対象 | 内容 | 疑似コード |
|---|---|---|---|
| `Makefile` | `fmt` | Docker 内で `cargo +nightly fmt` を実行する。 | `docker_run cargo +nightly fmt` |
| `Makefile` | `test-core` | Docker 内で `cargo test -p codex-core` を実行する。 | `docker_run cargo test -p codex-core` |
| `Makefile` | `test-all` | Docker 内で `cargo test --all-features` を実行する。 | `docker_run cargo test --all-features` |
| `Makefile` | `all` | `fmt` と `test-all` を続行実行する。 | `run_targets_continue_logged(fmt, test-all)` |
| `Makefile` | `test-almost` | `SKIP_ALMOST_TESTS` を除外して `cargo test` を実行する。 | `cargo test -- --skip ...` |
| `Makefile` | `almost` | `fmt` と `test-almost` を続行実行する。 | `run_targets_continue_logged(fmt, test-almost)` |
| `scripts/docker_run.sh` | Docker runner | `CODEX_HOME` / `CODEX_MEMORIES_HOME` / `CARGO_TARGET_DIR` を mount/env で揃える。 | `docker run with mapped env/cache` |
| `scripts/run_logged.sh` | ログ wrapper | Docker 実行結果を `_tmp/*_test_result.txt` に保存する。 | `run command | tee log` |
| `docker/Dockerfile` | 検証 image | Rust/Node/just/dotslash/cargo-insta/assistant user を用意する。 | `build codex-dev image` |

## `exec_mcp_*`

| パス | テスト/対象 | 内容 | 疑似コード |
|---|---|---|---|
| `_mcp/exec_mcp/tests/test_server.py` | `ExecMcpServerTests.test_stop_active_runs_stops_long_running_process` | `stop_active_runs` が長時間プロセスを停止し active run を空にする。 | `spawn sleep; stop; assert cancelled_by_stop && active_runs == []` |
| `_mcp/exec_mcp/pyproject.toml` | package entrypoint | `exec-mcp = "server:main"` を定義する。 | `install package; run exec-mcp` |
| `_mcp/exec_mcp/docker/Dockerfile` | MCP image | MCP 経由 build/test 用環境を固定する。 | `build exec-mcp image` |

## Snapshot 期待値

snapshot はテスト関数ではないが、UI/モデル入力の期待値として custom 差分に含まれる。

| 種別 | 件数 | 対応機能 |
|---|---:|---|
| 追加 snapshot | 3 | `custom_theme_diff_colors`、hooks warning、zellij composer / marketplace popup など TUI 表示 |
| 更新 snapshot | 42 | TUI 表示、core suite context、approval/status/terminal title/unified exec 表示 |

追加 snapshot:

- `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__zellij_empty_composer.snap`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__popups_and_settings__marketplace_upgrade_loading_popup_snapshot.snap`
- `codex-rs/tui/src/snapshots/codex_tui__app__tests__hooks_needing_review_startup_warning.snap`

## 本家追従/再実装対象外候補

機械抽出では追加テストだが、custom 固有機能との対応が弱いもの。
再実装時は、本家現行で既に存在するか、custom として維持する必要があるかを別途判定する。

### app-server / thread / resume / dynamic tools

- `codex-rs/app-server/src/codex_message_processor.rs`
  - `aborting_pending_request_clears_pending_state`
  - `adding_connection_to_thread_updates_has_connections_watcher`
  - `closed_connection_cannot_be_reintroduced_by_auto_subscribe`
  - `collect_resume_override_mismatches_includes_service_tier`
  - `command_profile_preserves_configured_deny_read_restrictions`
  - `config_load_error_leaves_non_cloud_requirements_failures_unmarked`
  - `config_load_error_marks_cloud_requirements_failures_for_relogin`
  - `config_load_error_marks_non_auth_cloud_requirements_failures_without_relogin`
  - `derive_config_from_params_uses_session_thread_config_model_provider`
  - `extract_conversation_summary_prefers_plain_user_messages`
  - `merge_persisted_resume_metadata_prefers_persisted_model_and_reasoning_effort`
  - `merge_persisted_resume_metadata_preserves_explicit_overrides`
  - `merge_persisted_resume_metadata_skips_missing_values`
  - `merge_persisted_resume_metadata_skips_persisted_values_when_model_overridden`
  - `merge_persisted_resume_metadata_skips_persisted_values_when_provider_overridden`
  - `merge_persisted_resume_metadata_skips_persisted_values_when_reasoning_effort_overridden`
  - `normalize_thread_list_cwd_filter_preserves_absolute_paths`
  - `normalize_thread_list_cwd_filter_resolves_relative_paths_against_server_cwd`
  - `read_summary_from_rollout_preserves_agent_nickname`
  - `read_summary_from_rollout_preserves_forked_from_id`
  - `read_summary_from_rollout_returns_empty_preview_when_no_user_message`
  - `removing_auto_attached_connection_preserves_listener_for_other_connections`
  - `removing_thread_state_clears_listener_and_active_turn_history`
  - `requested_permissions_trust_project_uses_permission_profile_intent`
  - `summary_from_state_db_metadata_preserves_agent_nickname`
  - `summary_from_stored_thread_preserves_millisecond_precision`
  - `summary_from_thread_metadata_formats_protocol_timestamps_as_seconds`
  - `thread_turns_list_merges_in_progress_active_turn_before_agent_status_running`
  - `validate_dynamic_tools_accepts_nullable_field_schema`
  - `validate_dynamic_tools_accepts_same_name_in_different_namespaces`
  - `validate_dynamic_tools_accepts_sanitizable_input_schema`
  - `validate_dynamic_tools_rejects_duplicate_name_in_same_namespace`
  - `validate_dynamic_tools_rejects_empty_namespace`
  - `validate_dynamic_tools_rejects_reserved_namespace`
  - `validate_dynamic_tools_rejects_unsupported_input_schema`

### config / profiles / plugins / connectors

- `codex-rs/core/src/config/config_tests.rs`
  - `approvals_reviewer_can_be_set_in_profile_without_guardian_approval`
  - `browser_feature_requirements_are_valid`
  - `cli_override_takes_precedence_over_profile_sandbox_mode`
  - `config_toml_deserializes_model_availability_nux`
  - `config_toml_deserializes_permission_profiles`
  - `config_toml_deserializes_status_line_use_colors_disabled`
  - `config_toml_deserializes_terminal_resize_reflow_config`
  - `config_toml_status_line_use_colors_defaults_to_enabled`
  - `filter_mcp_servers_by_allowlist_blocks_all_when_empty`
  - `filter_plugin_mcp_servers_by_allowlist_blocks_unlisted_plugin`
  - `filter_plugin_mcp_servers_by_allowlist_enforces_plugin_and_identity_rules`
  - `load_config_applies_amazon_bedrock_aws_profile_override`
  - `permissions_profiles_network_enabled_allows_runtime_network_without_proxy`
  - `profile_approvals_reviewer_falls_back_when_disallowed_by_requirements`
  - `profile_multi_agent_v2_config_overrides_base`
  - `runtime_config_defaults_model_availability_nux`
  - `set_feature_enabled_persists_feature_disable_in_profile`
  - `set_feature_enabled_profile_disable_overrides_root_enable`
  - `set_feature_enabled_updates_profile`
  - `set_model_updates_profile`
  - `smart_approvals_alias_is_ignored_in_profiles`
  - `test_precedence_fixture_with_o3_profile`
  - `test_resolve_oss_provider_explicit_override`
  - `test_resolve_oss_provider_from_profile`
  - `test_toml_parsing`
  - `test_tui_vim_mode_default_defaults_to_false`
  - `test_tui_vim_mode_default_true`
  - `tools_web_search_true_deserializes_to_none`
- `codex-rs/core/src/session/tests.rs`
  - `configured_multi_agent_v2_usage_hint_texts_omit_effectively_disabled_feature`
  - `configured_multi_agent_v2_usage_hint_texts_use_effective_enabled_feature_state`
  - `filter_connectors_for_input_skips_disabled_connectors`
  - `filter_connectors_for_input_skips_duplicate_slug_mentions`
  - `filter_connectors_for_input_skips_plugin_mentions`
  - `filter_connectors_for_input_skips_when_skill_name_conflicts`
  - `get_service_tier_does_not_default_non_enterprise_or_disabled_fast_mode`
  - `idle_interrupt_does_not_wake_queued_next_turn_items`
  - `prepend_pending_input_keeps_older_tail_ahead_of_newer_input`
  - `queued_response_items_for_next_turn_move_into_next_active_turn`
  - `record_model_warning_appends_user_message`
- `codex-rs/core/tests/suite/plugins.rs`
  - `plugin_mcp_tools_are_listed`
- `codex-rs/tools/src/tool_config_tests.rs`
  - `code_mode_only_implies_code_mode`

### TUI / CLI general

- `codex-rs/cli/src/debug_sandbox.rs`
  - `explicit_permission_profile_overrides_active_profile_sandbox_mode`
- `codex-rs/cli/src/responses_cmd.rs`
  - `reasoning_deltas_use_responses_event_names`
  - `response_events_keep_replayable_response_envelopes`
  - `tool_call_input_delta_uses_responses_event_name`
- `codex-rs/app-server/src/message_processor_tracing_tests.rs`
  - `turn_start_jsonrpc_span_parents_core_turn_spans`
- `codex-rs/core/src/tools/registry_tests.rs`
  - `handler_falls_back_from_shell_command_to_exec_command`
  - `handler_falls_back_to_flat_alias_for_namespaced_tools`
- `codex-rs/tui/src/app/tests.rs`
  - `hooks_needing_review_startup_warning_snapshot`
  - `ignore_same_thread_resume_allows_reattaching_displayed_inactive_thread`
  - `ignore_same_thread_resume_reports_noop_for_current_thread`
  - `session_summary_includes_resume_hint_for_persisted_rollout`
  - `session_summary_skips_when_no_usage_or_resume_hint`
  - `session_summary_uses_id_even_when_thread_has_name`
  - `startup_paused_goal_prompt_gate_is_only_for_quiet_resume`
  - `startup_waiting_gate_holds_active_thread_events_until_primary_thread_configured`
  - `startup_waiting_gate_is_only_for_fresh_or_exit_session_selection`
  - `update_feature_flags_disabling_guardian_in_profile_allows_inherited_user_reviewer`
  - `update_feature_flags_disabling_guardian_in_profile_keeps_inherited_non_user_reviewer_enabled`
  - `update_memory_settings_updates_current_thread_memory_mode`
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - `custom_prompt_completion_inserts_named_arg_template`
  - `zellij_empty_composer_snapshot`
- `codex-rs/tui/src/bottom_pane/command_popup.rs`
  - `prompt_prefix_filters_custom_prompt`
- `codex-rs/tui/src/bottom_pane/list_selection_view.rs`
  - `enter_accepts_selected_item_and_runs_actions`
- `codex-rs/tui/src/chatwidget/tests/composer_submission.rs`
  - `custom_keymap_survives_session_reconfiguration`
  - `submission_includes_configured_permission_profile`
- `codex-rs/tui/src/chatwidget/tests/plan_mode.rs`
  - `collab_slash_command_opens_picker_and_updates_mode`
- `codex-rs/tui/src/chatwidget/tests/review_mode.rs`
  - `replaced_turn_clears_pending_steers_but_keeps_queued_drafts`
- `codex-rs/tui/src/lib.rs`
  - `read_session_cwd_returns_none_without_sqlite_or_rollout_path`
