#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")/.."

status=0

cat_target() {
    local target="$1"

    printf '==> %s\n' "$target"
    if ! cat "$target"; then
        status=1
    fi
}

cat_target _custom_tests/allowed_file
cat_target _custom_tests/allowed_dir/allowed.txt
cat_target _custom_tests/not_allowed_file
cat_target _custom_tests/not_allowed_dir/not_allowed.txt

exit "$status"
