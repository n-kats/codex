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

date_jst_compact="$(TZ=Asia/Tokyo date +%Y%m%d)"
date_jst_hyphen="$(TZ=Asia/Tokyo date +%F)"

git_safe=(git -c "safe.directory=${root_dir}" -C "${root_dir}")

base_version_source="highest stable rust-vX.Y.Z tag"
is_shallow="$("${git_safe[@]}" rev-parse --is-shallow-repository 2>/dev/null || echo unknown)"
tags_raw="$("${git_safe[@]}" tag -l 'rust-v*' --sort=-v:refname 2>/dev/null || true)"
stable_tag="$(
  printf '%s\n' "${tags_raw}" \
    | { grep -E '^rust-v[0-9]+\.[0-9]+\.[0-9]+$' || true; } \
    | { grep -E -v '^rust-v0\.0\.' || true; } \
    | head -n 1
)"
base_version="${stable_tag#rust-v}"
if [[ -z "${stable_tag}" || -z "${base_version}" ]]; then
  echo "==> Release version (failed)" >&2
  echo "base_version_source=${base_version_source}" >&2
  echo "is_shallow_repository=${is_shallow}" >&2
  echo "ERROR: could not derive a stable base version (x.y.z) from tags." >&2
  echo "Fix: run 'make fetch' to update tags (and unshallow if needed), then ensure a stable rust-vX.Y.Z tag exists." >&2
  exit 2
fi

full_version="${base_version}-custom-${date_jst_hyphen}"
name="custom_codex_${full_version}"
stage_dir="$(mktemp -d)"
cargo_toml="${codex_rs_dir}/Cargo.toml"
cargo_toml_backup="${stage_dir}/Cargo.toml.bak"

restore_cargo_toml() {
  if [[ -f "${cargo_toml_backup}" ]]; then
    cp "${cargo_toml_backup}" "${cargo_toml}"
  fi
}

trap 'restore_cargo_toml; rm -rf "$stage_dir"' EXIT

echo "==> Release version"
echo "base_version_source=${base_version_source}"
echo "is_shallow_repository=${is_shallow}"
echo "base_tag=${stable_tag}"
echo "base_version=${base_version}"
echo "full_version=${full_version}"
echo "artifact_prefix=${name}"

echo "==> Building (release)"
cp "${cargo_toml}" "${cargo_toml_backup}"
tmp_cargo_toml="${stage_dir}/Cargo.toml"
awk -v new_version="${full_version}" '
  BEGIN { in_workspace_package=0; changed=0 }
  /^\[workspace\.package\]/ { in_workspace_package=1; print; next }
  /^\[/ { if (in_workspace_package) in_workspace_package=0; print; next }
  in_workspace_package && /^version[[:space:]]*=/ && changed == 0 {
    print "version = \"" new_version "\""
    changed=1
    next
  }
  { print }
  END { if (changed == 0) exit 3 }
' "${cargo_toml_backup}" >"${tmp_cargo_toml}"
cp "${tmp_cargo_toml}" "${cargo_toml}"

(cd "$codex_rs_dir" && cargo build -p codex-cli --bin codex --release)

target_dir="${CARGO_TARGET_DIR:-${codex_rs_dir}/target}"
bin_path="${target_dir}/release/codex"
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
git_rev="$("${git_safe[@]}" rev-parse --short HEAD 2>/dev/null || echo unknown)"

echo "==> Writing ${note_file}"
{
  printf '%s\n' \
    '# custom_codex リリース（Linux）' \
    '' \
    "- バージョン: ${full_version}" \
    "- ベース: ${base_version}" \
    "- ファイル: ${name}.tar.gz" \
    "- 日付（JST）: ${date_jst_compact}" \
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
