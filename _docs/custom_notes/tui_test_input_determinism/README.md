# TUI テストの入力待ちを発生させないための初期状態

## 目的

- TUI の統合テストが実端末のキー入力を待ち続け、テスト全体を長時間停止させることを防ぐ。
- テスト対象の機能と無関係なディレクトリ信頼確認を、テスト入力に依存させない。

## 変更内容

- `restored_server_permission_profile_survives_cd_without_turn_override` の初期状態で、テスト用の作業ディレクトリを Trusted に設定する既存ヘルパーを呼び出す。
- これにより、エージェント一覧からスレッドを選択する際に本番のディレクトリ信頼 onboarding が開かず、実端末の Enter 待ちにならない。
- 同じテストモジュールにあった3件の inline snapshot を、改行を含む複数行形式へ整え、`insta` の将来互換性警告を解消する。

## 対象範囲

- `codex-rs/tui/src/app/agents_overview_tests.rs` の対象テスト。

## 非対象

- ディレクトリ信頼の本番実装、onboarding の入力処理、skip/flaky テスト一覧。
- permission profile の実装および `/cd` の動作。

## 注意点（環境差・既知の制約）

- `make_test_app` のテスト用作業ディレクトリは既定で `/tmp/project` であり、初期状態では信頼済みとは限らない。
- `select_agents_overview_thread` は未信頼の作業ディレクトリに対して実際の `tui.event_stream()` を使う onboarding を起動するため、テストからキー入力を注入しない場合は端末入力待ちになり得る。
- このテストは信頼 onboarding を検証するものではないため、初期フォルダを信頼済みにするのが適切である。
- 全体実行ログに残る別テスト群の legacy inline snapshot 警告は、内容不一致や入力待ちではないため、この修正の対象外とした。

## 動作確認手順

- MCP の Cargo テスト実行機能で、次のテストを単体実行する。
  - `codex-tui` / `lib` / `app::agents_overview::tests::restored_server_permission_profile_survives_cd_without_turn_override`
- 手元環境では通常の手順に従い、`cd codex-rs && just test -p codex-tui` を実行する。
- 全体の `make almost` を再実行する場合も、ログに実端末の Trust onboarding が表示されず、対象テストが入力待ちにならないことを確認する。

## つまずきと対処

- `make almost` のログで `codex-tui --lib` が約41時間かかり、途中で `Trust this folder? Press enter to continue` が表示された。
- 対象テストは作業ディレクトリの信頼状態を設定せずにスレッド選択を行っていたため、実端末の Enter を待っていた。
- 初期状態を `trust_fixture_folders` で明示し、skip/flaky 登録ではなくテストの前提条件を修正した。
- inline snapshot は内容を変更せず、先頭・末尾の改行を含む `@r"..."` 形式へ移行した。

## 実施結果

- 2026-09-21 の MCP 経由の almost 相当実行は fmt、Linux sandbox build、test-almost の全工程が成功した。
- `codex-tui --lib` は 4641 passed / 0 failed / 3 ignored、所要 49.93 秒で、実端末入力待ちの 60 秒超過警告は再発しなかった。
- 修正後の最初の全体実行で発生した daemon の単独テスト失敗は、単体・クレート全体では再現せず、全体実行を再実行すると成功したため、skip/flaky リストには追加していない。

## 関連ファイル一覧

- `codex-rs/tui/src/app/agents_overview_tests.rs`
- `codex-rs/tui/src/app/agents_overview.rs`
- `codex-rs/tui/src/onboarding/directory_trust.rs`
- `codex-rs/tui/src/onboarding/onboarding_screen.rs`
