#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd "${script_dir}/.." && pwd)"
assets_dir="${V8_ASSETS_DIR:-${root_dir}/_tmp/assets}"
target="${V8_TARGET:-$(rustc -vV | awk '/^host: / { print $2; exit }')}"
version="${V8_VERSION:-$(awk '/^name = "v8"$/ { found=1 } found && /^version = / { gsub(/"/, "", $3); print $3; exit }' "${root_dir}/codex-rs/Cargo.lock")}"
profile="ptrcomp_sandbox_release"
release_url="https://github.com/openai/codex/releases/download/rusty-v8-v${version}"

if [[ -z "${target}" || -z "${version}" ]]; then
    echo "failed to resolve V8 target or version" >&2
    exit 1
fi

case "${target}" in
    *-pc-windows-msvc)
        archive_name="rusty_v8_${profile}_${target}.lib.gz"
        ;;
    *)
        archive_name="librusty_v8_${profile}_${target}.a.gz"
        ;;
esac
binding_name="src_binding_${profile}_${target}.rs"
checksums_name="rusty_v8_${profile}_${target}.sha256"

mkdir -p "${assets_dir}"

download() {
    local name="$1"
    local destination="${assets_dir}/${name}"
    if [[ -s "${destination}" ]]; then
        return
    fi
    echo "Downloading ${name}..."
    curl --fail --location --retry 3 --silent --show-error \
        "${release_url}/${name}" -o "${destination}.part"
    mv "${destination}.part" "${destination}"
}

download "${checksums_name}"
download "${archive_name}"
download "${binding_name}"

if command -v sha256sum >/dev/null 2>&1; then
    (cd "${assets_dir}" && tr -d '\r' < "${checksums_name}" | sha256sum -c -)
else
    (cd "${assets_dir}" && tr -d '\r' < "${checksums_name}" | shasum -a 256 -c -)
fi

echo "V8 assets are ready in ${assets_dir}"
echo "RUSTY_V8_ARCHIVE=${assets_dir}/${archive_name}"
echo "RUSTY_V8_SRC_BINDING_PATH=${assets_dir}/${binding_name}"
