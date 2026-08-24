#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
skip_file="${repo_root}/skip_test_list.txt"
flaky_file="${repo_root}/flaky_test_list.txt"

cd "${repo_root}/codex-rs"

skip_tests=()
while IFS= read -r test_name; do
  [[ -z "${test_name}" || "${test_name}" == \#* ]] && continue
  skip_tests+=("--skip" "${test_name}")
done < "${skip_file}"

while IFS= read -r test_name; do
  [[ -z "${test_name}" || "${test_name}" == \#* ]] && continue
  skip_tests+=("--skip" "${test_name}")
done < "${flaky_file}"

cargo test ${CARGO_TEST_FLAGS:-} -- "${skip_tests[@]}"
