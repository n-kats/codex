## はじめに（日本語メモ）

このドキュメントは `docs/getting-started.md` の日本語メモです（原本は編集しない）。

### CLI の使い方

| コマンド            | 用途                           | 例                              |
| ------------------ | ------------------------------ | ------------------------------- |
| `codex`            | 対話型 TUI                     | `codex`                         |
| `codex "..."`      | 対話型 TUI の初期プロンプト     | `codex "fix lint errors"`       |
| `codex exec "..."` | 非対話の「自動化モード」        | `codex exec "explain utils.ts"` |

主なフラグ: `--model/-m`, `--ask-for-approval/-a`.

### 対話セッションの再開

- 一覧: `codex resume`
- 直近: `codex resume --last`
- ID 指定: `codex resume <SESSION_ID>`

### プロンプト例

- `codex "Explain this codebase to me"`
- `codex "Write unit tests for utils/date.ts"`

### AGENTS.md（メモリ）

Codex は `AGENTS.md` を上から順にマージします:

1. `~/.codex/AGENTS.md`
2. リポジトリルートから CWD までの各ディレクトリ（`AGENTS.override.md` があれば優先）

