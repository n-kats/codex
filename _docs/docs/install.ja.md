## インストール & ビルド（日本語メモ）

このドキュメントは `docs/install.md` の日本語メモです（原本は編集しない）。

### システム要件

| 要件              | 詳細                                                           |
| ----------------- | -------------------------------------------------------------- |
| 対応 OS           | macOS 12+、Ubuntu 20.04+/Debian 10+、または Windows 11（WSL2） |
| Git（任意・推奨） | 2.23+                                                          |
| RAM               | 最低 4GB（推奨 8GB）                                           |

### ソースからビルド（概要）

```bash
git clone https://github.com/openai/codex.git
cd codex/codex-rs

rustup component add rustfmt
rustup component add clippy
cargo install just

cargo build
cargo run --bin codex -- "explain this codebase to me"
```

