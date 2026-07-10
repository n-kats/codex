#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root_dir="${REBASE_WORKFLOW_ROOT:-${EXEC_MCP_ROOT:-${script_dir%/*}}}"

if [[ "${root_dir}" != /* ]]; then
  root_dir="$(cd "${script_dir}/../${root_dir}" && pwd)"
else
  root_dir="$(cd "${root_dir}" && pwd)"
fi

repo_dir="${root_dir}"
remote_name="fork-origin"
base_branch="fork-origin/main"
work_branch="custom"
backup_branch="tmp-rebase"

git -C "${repo_dir}" rev-parse --is-inside-work-tree >/dev/null

current_branch="$(git -C "${repo_dir}" branch --show-current)"
if [[ "${current_branch}" != "${work_branch}" ]]; then
  echo "expected to run on ${work_branch}, got ${current_branch:-detached HEAD}" >&2
  exit 1
fi

if [[ -n "$(git -C "${repo_dir}" status --porcelain=v1)" ]]; then
  echo "working tree must be clean before rebase" >&2
  exit 1
fi

if git -C "${repo_dir}" show-ref --verify --quiet "refs/heads/${backup_branch}"; then
  echo "${backup_branch} already exists; stop to avoid overwriting the backup branch" >&2
  exit 1
fi

echo "fetching ${remote_name}..."
git -C "${repo_dir}" fetch --all
if git -C "${repo_dir}" rev-parse --is-shallow-repository | grep -q true; then
  git -C "${repo_dir}" fetch "${remote_name}" --prune --tags --force --unshallow
else
  git -C "${repo_dir}" fetch "${remote_name}" --prune --tags --force
fi

echo "creating ${backup_branch} from ${work_branch}..."
git -C "${repo_dir}" branch "${backup_branch}"

base_commit="$(git -C "${repo_dir}" merge-base "${base_branch}" "${work_branch}")"
commit_count="$(git -C "${repo_dir}" rev-list --count "${base_commit}..${work_branch}")"

if [[ "${commit_count}" != "0" ]]; then
  echo "squashing ${commit_count} commit(s) onto ${base_commit}..."
  git -C "${repo_dir}" reset --soft "${base_commit}"
  git -C "${repo_dir}" commit -m "custom changes"
else
  echo "no commits to squash; leaving ${work_branch} as-is before rebase"
fi

echo "rebasing ${work_branch} onto ${base_branch}..."
git -C "${repo_dir}" rebase "${base_branch}"

echo "rebase complete"
echo "work_branch=$(git -C "${repo_dir}" rev-parse --short HEAD)"
echo "backup_branch=$(git -C "${repo_dir}" rev-parse --short "${backup_branch}")"
