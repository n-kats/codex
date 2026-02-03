#!/usr/bin/env bash
set -euo pipefail

name="${1:?missing name}"
shift
command="${1:?missing command}"

log_file="${LOG_FILE:?LOG_FILE is required}"
mode="${LOG_APPEND:-0}"

if [[ "${mode}" = "1" ]]; then
  { printf "%s\n" "==> ${name} ($(date -Is))"; } | tee -a "${log_file}" >/dev/null
  CODEX_DOCKER_IMAGE_NAME="${CODEX_DOCKER_IMAGE_NAME:-}" \
  CODEX_DOCKER_PLATFORM="${CODEX_DOCKER_PLATFORM:-}" \
  CODEX_DOCKER_CACHE_DIR="${CODEX_DOCKER_CACHE_DIR:-}" \
  bash "${DOCKER_RUN}" "${command}" 2>&1 | tee -a "${log_file}"
else
  { printf "%s\n" "==> ${name} ($(date -Is))"; } | tee "${log_file}" >/dev/null
  CODEX_DOCKER_IMAGE_NAME="${CODEX_DOCKER_IMAGE_NAME:-}" \
  CODEX_DOCKER_PLATFORM="${CODEX_DOCKER_PLATFORM:-}" \
  CODEX_DOCKER_CACHE_DIR="${CODEX_DOCKER_CACHE_DIR:-}" \
  bash "${DOCKER_RUN}" "${command}" 2>&1 | tee "${log_file}"
fi
