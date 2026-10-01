# 20260429-rebase-custom

- File: `codex-rs/core/src/lib.rs`
  - Line: 60
  - Resolution: custom 維持
  - Note: `codex_core::config_loader` を互換 re-export として戻し、`LoaderOverrides` を参照する既存コードを壊さないようにした。

- File: `codex-rs/core/src/memories/mod.rs`
  - Line: 4
  - Resolution: 手動マージ
  - Note: 上流で欠けていた memories 実装を、`CODEX_MEMORIES_HOME` を読む最小互換シムに寄せて整理した。`clear_memory_roots_contents` はそのまま再公開した。

- File: `codex-rs/core/src/config/mod.rs`
  - Line: 2491
  - Resolution: upstream 優先
  - Note: `ShellEnvironmentPolicyInherit` は公開型 `codex_protocol::config_types` 側を使うように修正し、private 型参照をやめた。

- File: `codex-rs/core/src/session/session.rs`
  - Line: 2
  - Resolution: upstream 優先
  - Note: `ConstraintError` の参照元を `crate::config` に揃え、セッション側の型解決を上流形に戻した。

- File: `codex-rs/app-server/src/codex_message_processor.rs`
  - Line: 3231
  - Resolution: 手動マージ
  - Note: `thread_metadata_update_response` の受け取り型を `&ConnectionRequestId` に合わせ直し、エラーは `Result` で返す形に整理した。補助関数 `normalize_thread_metadata_git_field` は残した。

- File: `codex-rs/cli/src/main.rs`
  - Line: 204
  - Resolution: 手動マージ
  - Note: `Responses` サブコマンドの結線を戻し、`run_login_with_agent_identity` の呼び出しは現在の署名に合わせた。

- File: `codex-rs/cli/src/login.rs`
  - Line: 203
  - Resolution: upstream 優先
  - Note: `load_config_or_exit` へ `LoaderOverrides::default()` を渡す形に揃えた。

- File: `codex-rs/cli/src/marketplace_cmd.rs`
  - Line: 1
  - Resolution: upstream 優先
  - Note: marketplace の参照先を `codex_core_plugins` に合わせ、不要になった古い import を外した。

- File: `codex-rs/cli/src/responses_cmd.rs`
  - Line: 12
  - Resolution: custom 維持
  - Note: `Responses` CLI を再接続し、`AuthManager::shared_from_config(...).await` と `ResponseEvent::Completed` の新しい形に合わせた。

- File: `codex-rs/cloud-tasks/src/util.rs`
  - Line: 48
  - Resolution: 手動マージ
  - Note: `AuthManager::new(...).await` を使う形へ更新しつつ、ChatGPT JWT から account id を取り出す補助関数は互換用に残した。

- File: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - Line: 210
  - Resolution: upstream 優先
  - Note: merge で重複していた `history_search` の宣言と import を整理して、単一の実装に戻した。

- File: `codex-rs/tui/src/bottom_pane/footer.rs`
  - Line: 1054
  - Resolution: 手動マージ
  - Note: `ShortcutId::SendMessage` の overlay 表示を戻し、短縮キー一覧が欠けないようにした。
