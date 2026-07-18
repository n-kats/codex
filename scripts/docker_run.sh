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

cache_dir="$(realpath "${CODEX_DOCKER_CACHE_DIR:-${root_dir}/_cache/docker}")"
mkdir -p "${cache_dir}/cargo" "${cache_dir}/rustup" "${cache_dir}/home"

codex_home="${CODEX_HOME:-${root_dir}/_cache/codex_home}"
mkdir -p "${codex_home}"

container_codex_home="/workspace${codex_home#${root_dir}}"

codex_memories_home="${CODEX_MEMORIES_HOME:-}"
container_codex_memories_home=""
if [[ -n "${codex_memories_home}" ]]; then
  if [[ "${codex_memories_home}" != /* ]]; then
    codex_memories_home="${root_dir}/${codex_memories_home}"
  fi
  mkdir -p "${codex_memories_home}"
  if [[ "${codex_memories_home}" == "${root_dir}"* ]]; then
    container_codex_memories_home="/workspace${codex_memories_home#${root_dir}}"
  else
    echo "WARNING: CODEX_MEMORIES_HOME is outside the repo root and will not be visible in the container: ${codex_memories_home}" >&2
    codex_memories_home=""
  fi
fi

host_target_dir="${CODEX_DOCKER_TARGET_DIR:-${cache_dir}/target}"
if [[ "${host_target_dir}" != /* ]]; then
  host_target_dir="${root_dir}/${host_target_dir}"
fi
mkdir -p "${host_target_dir}"

container_target_dir="${CODEX_DOCKER_TARGET_DIR_IN_CONTAINER:-/var/cache/codex/target}"

docker_platform_args=()
if [[ -n "${platform}" ]]; then
  docker_platform_args+=(--platform "${platform}")
fi

ensure_mount_writable() {
  local host_dir="$1"
  local label="$2"

  if docker run --rm "${docker_platform_args[@]}" --user ubuntu -v "${host_dir}:/mnt" "${image_name}" \
    bash -lc 'test -w /mnt'; then
    return
  fi

  echo "INFO: fixing Docker mount permissions for ${label}: ${host_dir}" >&2
  docker run --rm "${docker_platform_args[@]}" --user root -v "${host_dir}:/mnt" "${image_name}" \
    bash -lc 'uid="$(id -u ubuntu)"; gid="$(id -g ubuntu)"; mkdir -p /mnt && chown -R "$uid:$gid" /mnt && chmod -R u+rwX /mnt'
}

ensure_mount_writable "${cache_dir}/cargo" "cargo cache"
ensure_mount_writable "${cache_dir}/rustup" "rustup cache"
ensure_mount_writable "${cache_dir}/home" "home cache"
ensure_mount_writable "${host_target_dir}" "cargo target"

docker_args=(
  --rm
  --user ubuntu
  --security-opt seccomp=unconfined
  --security-opt apparmor=unconfined
  -e "CODEX_HOME=${container_codex_home}"
  -e "CODEX_SHELL_STARTUP_FILES=${CODEX_SHELL_STARTUP_FILES:-clean}"
  -e "TERM=${TERM:-xterm-256color}"
  -e "COLORTERM=${COLORTERM:-}"
  -e "RUST_BACKTRACE=${RUST_BACKTRACE:-1}"
  -e "RUST_LOG=${RUST_LOG:-}"
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

if [[ -n "${container_codex_memories_home}" ]]; then
  docker_args+=(-e "CODEX_MEMORIES_HOME=${container_codex_memories_home}")
fi

# Interactive programs (like the TUI) need stdin to be open so the terminal can
# respond to queries (e.g., cursor position reports). Only allocate stdin when
# we're running attached to a tty.
if [[ -t 0 ]]; then
  docker_args+=(-i -t)
else
  docker_args+=(-t)
fi

if [[ -n "${platform}" ]]; then
  docker_args+=("${docker_platform_args[@]}")
fi

if [[ $# -eq 0 ]]; then
  echo "usage: scripts/docker_run.sh <command>" >&2
  exit 1
fi

command="$*"

exec docker run "${docker_args[@]}" \
  "${image_name}" \
  bash -lc "${command}"
