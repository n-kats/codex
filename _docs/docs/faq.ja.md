## FAQ（日本語メモ）

このドキュメントは `docs/faq.md` の日本語メモです（原本は編集しない）。

### 2021 年の Codex モデルと関係がありますか？

2021 年に公開された Codex モデルは 2023 年 3 月に提供終了しており、この CLI ツールとは別物です。

### どのモデルがサポートされていますか？

推奨モデルや reasoning レベルは原本の FAQ を参照してください。

### `brew upgrade codex` がアップグレードされない

原本の手順に従って formula をアンインストールして cask を入れ直してください:

```bash
brew uninstall --formula codex
brew install --cask codex
```

