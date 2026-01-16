#!/usr/bin/env bash
set -euo pipefail

root_dir="${1:-}"
codex_rs_dir="${2:-}"
release_dir="${3:-}"

if [[ -z "$root_dir" || -z "$codex_rs_dir" || -z "$release_dir" ]]; then
  echo "usage: $0 <root_dir> <codex_rs_dir> <release_dir>" >&2
  exit 2
fi

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "ERROR: release_linux.sh is intended for Linux (uname -s=$(uname -s))" >&2
  exit 1
fi

date_jst="$(TZ=Asia/Tokyo date +%Y%m%d)"
name="custom_codex_${date_jst}"
stage_dir="$(mktemp -d)"
trap 'rm -rf "$stage_dir"' EXIT

echo "==> Building (release)"
(cd "$codex_rs_dir" && cargo build -p codex-cli --bin codex --release)

bin_path="${codex_rs_dir}/target/release/codex"
if [[ ! -x "$bin_path" ]]; then
  echo "ERROR: expected built binary at ${bin_path}" >&2
  exit 1
fi

cp "$bin_path" "$stage_dir/codex"

tar_entries=("codex")
if [[ -f "${root_dir}/LICENSE" ]]; then
  cp "${root_dir}/LICENSE" "$stage_dir/LICENSE"
  tar_entries+=("LICENSE")
fi
if [[ -f "${root_dir}/NOTICE" ]]; then
  cp "${root_dir}/NOTICE" "$stage_dir/NOTICE"
  tar_entries+=("NOTICE")
fi

strip_used="no"
if command -v strip >/dev/null 2>&1; then
  echo "==> Stripping binary"
  strip "$stage_dir/codex"
  strip_used="yes"
else
  echo "WARN: strip not found; skipping strip" >&2
fi

mkdir -p "$release_dir"
tarball="${release_dir}/${name}.tar.gz"
sha_file="${release_dir}/${name}.tar.gz.sha256"
note_file="${release_dir}/${name}_RELEASE.md"

echo "==> Packaging ${tarball}"
tar -C "$stage_dir" -czf "$tarball" "${tar_entries[@]}"

echo "==> Writing sha256"
(
  cd "$release_dir"
  sha256sum "${name}.tar.gz" >"${name}.tar.gz.sha256"
)

sha_line="$(cat "$sha_file")"
git_rev="$(cd "$root_dir" && git rev-parse --short HEAD 2>/dev/null || echo unknown)"

echo "==> Writing ${note_file}"
{
  printf '%s\n' \
    '# custom_codex リリース（Linux）' \
    '' \
    "- ファイル: ${name}.tar.gz" \
    "- 日付（JST）: ${date_jst}" \
    "- コミット: ${git_rev}" \
    "- strip 済み: ${strip_used}" \
    '' \
    '## 使い方' \
    '' \
    '1) ダウンロードした `tar.gz` と `sha256` を同じフォルダに置く' \
    '' \
    '2) ハッシュ検証:' \
    '' \
    '```bash' \
    "sha256sum -c ${name}.tar.gz.sha256" \
    '```' \
    '' \
    '期待される sha256:' \
    '' \
    '```' \
    "${sha_line}" \
    '```' \
    '' \
    '3) 展開して実行:' \
    '' \
    '```bash' \
    "tar -xzf ${name}.tar.gz" \
    './codex --help' \
    '```' \
    '' \
    '## 付属ファイルについて' \
    '' \
    '- この tarball には `LICENSE` と `NOTICE` を同梱しています。' \
    '- 署名ファイル（`.asc`）は **付属しません**（sha256 のみ）。' \
    '' \
    '## GitHub Releases に置くファイル' \
    '' \
    "- ${name}.tar.gz" \
    "- ${name}.tar.gz.sha256"
} >"$note_file"

echo "==> Done"
echo "Artifacts:"
ls -lah "$tarball" "$sha_file" "$note_file"

echo ""
echo "==> GitHub Releases 本文（コピペ用）"
cat "$note_file"
