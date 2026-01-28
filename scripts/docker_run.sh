#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
image_name="${CODEX_DOCKER_IMAGE_NAME:-codex-dev}"
platform="${CODEX_DOCKER_PLATFORM:-}"

env_file="${root_dir}/.env"
if [[ -f "${env_file}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${env_file}"
  set +a
fi

cache_dir="${CODEX_DOCKER_CACHE_DIR:-${root_dir}/_cache/docker}"
mkdir -p "${cache_dir}/cargo" "${cache_dir}/rustup" "${cache_dir}/home"

codex_home="${CODEX_HOME:-${root_dir}/_cache/codex_home}"
if [[ -n "${CODEX_HOME:-}" && "${codex_home}" = /* && "${codex_home}" != "${root_dir}"* ]]; then
  echo "WARNING: CODEX_HOME must be under ${root_dir} for Docker; falling back to ${root_dir}/_cache/codex_home" >&2
  codex_home="${root_dir}/_cache/codex_home"
fi
mkdir -p "${codex_home}"

# Convert host CODEX_HOME into the container path under /workspace.
container_codex_home="/workspace${codex_home#${root_dir}}"

host_target_dir="${CODEX_DOCKER_TARGET_DIR:-${cache_dir}/target}"
if [[ "${host_target_dir}" != /* ]]; then
  host_target_dir="${root_dir}/${host_target_dir}"
fi
mkdir -p "${host_target_dir}"

container_target_dir="${CODEX_DOCKER_TARGET_DIR_IN_CONTAINER:-/var/cache/codex/target}"

docker_args=(
  --rm
  --user ubuntu
  -e "CODEX_HOME=${container_codex_home}"
  -e "CODEX_SHELL_STARTUP_FILES=${CODEX_SHELL_STARTUP_FILES:-clean}"
  -e "CARGO_TARGET_DIR=${container_target_dir}"
  -e "CARGO_HOME=/home/ubuntu/.cargo"
  -e "RUSTUP_HOME=/home/ubuntu/.rustup"
  -e "HOME=/home/ubuntu"
  -e "USER=ubuntu"
  -e "USERNAME=ubuntu"
  -e "LOGNAME=ubuntu"
  -v "${root_dir}:/workspace"
  -v "${cache_dir}/cargo:/home/ubuntu/.cargo"
  -v "${cache_dir}/rustup:/home/ubuntu/.rustup"
  -v "${cache_dir}/home:/home/ubuntu"
  -v "${host_target_dir}:${container_target_dir}"
  -w /workspace
)

# Interactive programs (like the TUI) need stdin to be open so the terminal can
# respond to queries (e.g., cursor position reports). Only allocate stdin when
# we're running attached to a tty.
if [[ -t 0 ]]; then
  docker_args+=(-i -t)
else
  docker_args+=(-t)
fi

if [[ -n "${platform}" ]]; then
  docker_args+=(--platform "${platform}")
fi

if [[ $# -eq 0 ]]; then
  echo "usage: scripts/docker_run.sh <command>" >&2
  exit 1
fi

command="$*"

exec docker run "${docker_args[@]}" \
  "${image_name}" \
  bash -lc "${command}"
