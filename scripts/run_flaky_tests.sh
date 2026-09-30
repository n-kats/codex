#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
flaky_test_file="${repo_root}/flaky_test_list.txt"

if [[ ! -f "${flaky_test_file}" ]]; then
  printf '%s\n' "missing flaky test list: ${flaky_test_file}" >&2
  exit 1
fi

status=0
while IFS= read -r flaky_test; do
  [[ -z "${flaky_test}" || "${flaky_test}" == \#* ]] && continue
  cargo test ${CARGO_TEST_FLAGS:-} "${flaky_test}" -- --exact --test-threads=1 || status=$?
done < "${flaky_test_file}"

exit "${status}"
