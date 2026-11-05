"""Run a host Cargo command with the local Bazel-built rusty_v8 pair."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from typing import Mapping


def resolve_host_v8_cargo_env(
    repo_root: Path,
    *,
    environ: Mapping[str, str] | None = None,
) -> dict[str, str]:
    env = dict(os.environ if environ is None else environ)
    repo_root = repo_root.resolve()
    env["CODEX_REPO_ROOT"] = str(repo_root)
    os.environ["CODEX_REPO_ROOT"] = str(repo_root)

    repo_path = str(repo_root)
    if repo_path not in sys.path:
        sys.path.insert(0, repo_path)

    from scripts.codex_package.targets import TARGET_SPECS
    from scripts.codex_package.v8 import resolve_workspace_v8_cargo_env

    rustc = subprocess.run(
        ["rustc", "-vV"],
        cwd=repo_root / "codex-rs",
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    host_target = None
    for line in rustc.splitlines():
        if line.startswith("host:"):
            host_target = line.partition(":")[2].strip()
            break
    spec = TARGET_SPECS.get(host_target or "")
    if spec is None:
        raise RuntimeError(
            f"No Bazel V8 test target for host target {host_target!r}."
        )

    return resolve_workspace_v8_cargo_env(
        spec,
        asset_dir=repo_root / "_tmp" / "assets",
    )


def main(argv: list[str]) -> None:
    if argv and argv[0] == "--":
        argv = argv[1:]
    if not argv:
        raise SystemExit("usage: v8_env.py -- <command> [args ...]")

    repo_root = Path(__file__).resolve().parents[2]
    env = os.environ.copy()
    env.pop("V8_FROM_SOURCE", None)
    env.update(resolve_host_v8_cargo_env(repo_root, environ=env))
    os.execvpe(argv[0], argv, env)


if __name__ == "__main__":
    main(sys.argv[1:])
