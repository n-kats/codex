from __future__ import annotations

import asyncio
import argparse
import grp
import logging
import os
import pwd
import platform
import signal
import re
import shutil
import subprocess
import sys
import threading
import time
import uuid
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

try:
    from fastmcp import FastMCP
except ModuleNotFoundError:  # pragma: no cover
    class FastMCP:  # type: ignore[no-redef]
        def __init__(self, name: str):
            self.name = name

        def tool(self, fn=None):
            if fn is None:
                return lambda f: f
            return fn

        def run(self, *args: Any, **kwargs: Any) -> None:
            raise RuntimeError("fastmcp がインストールされていません")


@dataclass(frozen=True)
class StepSpec:
    name: str
    argv: list[str]
    cwd: str


@dataclass
class ActiveRun:
    run_id: str
    tool_name: str
    argv: list[str]
    cwd: str
    started_at_monotonic: float
    started_at_utc: datetime
    process: asyncio.subprocess.Process
    cancel_requested: bool = False


mcp = FastMCP("exec_mcp")

MAX_OUTPUT_CHARS = 400_000
DEFAULT_ROOT = "/workspace"
DEFAULT_CODEX_RS_DIR = "codex-rs"
DEFAULT_CACHE_DIR = "_cache/docker"
DEFAULT_CODEX_HOME = "_cache/codex_home_debug"
DEFAULT_TARGET_DIR = "_tmp/exec_mcp_target"
DEFAULT_TMP_DIR = "_tmp"
_ACTIVE_RUNS: dict[str, ActiveRun] = {}
_ACTIVE_RUNS_LOCK = threading.Lock()
ALMOST_SKIP_TESTS_FILE = "skip_test_list.txt"
FLAKY_TESTS_FILE = "flaky_test_list.txt"


def _configure_logging() -> None:
    level_name = os.environ.get("EXEC_MCP_LOG_LEVEL", "INFO").upper()
    level = getattr(logging, level_name, logging.INFO)
    logging.basicConfig(
        level=level,
        stream=sys.stderr,
        format="%(asctime)sZ %(levelname)s %(name)s: %(message)s",
    )
    logging.getLogger("uvicorn.error").setLevel(level)
    logging.getLogger("uvicorn.access").setLevel(level)


def _root_dir() -> Path:
    return Path(os.environ.get("EXEC_MCP_ROOT", DEFAULT_ROOT)).resolve()


def _log_root_path_error(
    *, env_name: str, root_resolved: Path, resolved: Path, raw_value: str | None
) -> None:
    logging.getLogger("exec_mcp").error(
        "%s は root 配下のみ許可: root=%s path=%s raw=%r",
        env_name,
        root_resolved,
        resolved,
        raw_value,
    )


def _resolve_cwd(root_dir: Path, cwd: str | None) -> Path:
    if cwd is None or cwd == "":
        return root_dir

    cwd_path = Path(cwd)
    if not cwd_path.is_absolute():
        cwd_path = root_dir / cwd_path

    resolved = cwd_path.resolve()
    root_resolved = root_dir.resolve()
    try:
        resolved.relative_to(root_resolved)
    except ValueError as exc:
        _log_root_path_error(
            env_name="cwd", root_resolved=root_resolved, resolved=resolved, raw_value=cwd
        )
        raise ValueError(
            f"cwd は root 配下のみ許可: root={root_resolved} cwd={resolved} raw={cwd!r}"
        ) from exc
    return resolved


def _resolve_root_path(
    root_dir: Path,
    raw_value: str | None,
    default_rel: str,
    *,
    env_name: str,
    allow_outside_root: bool = False,
) -> Path:
    raw = (raw_value or "").strip()
    path = Path(raw) if raw else root_dir / default_rel
    if not path.is_absolute():
        path = root_dir / path
    resolved = path.resolve()
    root_resolved = root_dir.resolve()
    if allow_outside_root:
        if not resolved.is_relative_to(root_resolved):
            logging.getLogger("exec_mcp").warning(
                "%s は root 外を使用: root=%s path=%s raw=%r",
                env_name,
                root_resolved,
                resolved,
                raw_value,
            )
        return resolved
    try:
        resolved.relative_to(root_resolved)
    except ValueError as exc:
        _log_root_path_error(
            env_name=env_name,
            root_resolved=root_resolved,
            resolved=resolved,
            raw_value=raw_value,
        )
        raise ValueError(
            f"{env_name} は root 配下のみ許可: root={root_resolved} path={resolved} raw={raw_value!r}"
        ) from exc
    return resolved


def _validated_exec_path(
    root_dir: Path, raw_value: str | None, default_rel: str, *, env_name: str
) -> Path:
    raw = (raw_value or "").strip()
    if not raw:
        return _resolve_root_path(root_dir, None, default_rel, env_name=env_name)

    path = Path(raw)
    if not path.is_absolute():
        path = root_dir / path
    resolved = path.resolve()
    if not resolved.is_relative_to(root_dir.resolve()):
        _log_root_path_error(
            env_name=env_name,
            root_resolved=root_dir.resolve(),
            resolved=resolved,
            raw_value=raw_value,
        )
    return path


def _limit_text(text: str, max_chars: int) -> tuple[str, bool]:
    if max_chars <= 0:
        return "", True
    if len(text) <= max_chars:
        return text, False
    return text[:max_chars], True


def _almost_skip_tests_file() -> Path:
    return _root_dir() / ALMOST_SKIP_TESTS_FILE


def _load_test_names(path: Path) -> tuple[str, ...]:
    skip_tests = []
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if line:
            skip_tests.append(line)
    return tuple(skip_tests)


def _load_almost_skip_tests() -> tuple[str, ...]:
    return _load_test_names(_almost_skip_tests_file()) + _load_test_names(
        _root_dir() / FLAKY_TESTS_FILE
    )


def _path_snapshot(
    path: Path, *, list_entries: bool = False, max_entries: int = 8
) -> dict[str, Any]:
    snapshot: dict[str, Any] = {
        "path": str(path),
        "exists": path.exists(),
        "is_dir": path.is_dir(),
        "is_file": path.is_file(),
        "is_symlink": path.is_symlink(),
    }
    if not snapshot["exists"]:
        return snapshot

    try:
        stat_result = path.lstat()
        snapshot["mode"] = oct(stat_result.st_mode & 0o7777)
        snapshot["uid"] = stat_result.st_uid
        snapshot["gid"] = stat_result.st_gid
        snapshot["owner"] = pwd.getpwuid(stat_result.st_uid).pw_name
    except KeyError:
        snapshot["owner"] = None
    try:
        snapshot["group"] = grp.getgrgid(stat_result.st_gid).gr_name
    except KeyError:
        snapshot["group"] = None
    snapshot["size"] = stat_result.st_size
    snapshot["mtime"] = datetime.fromtimestamp(stat_result.st_mtime, tz=timezone.utc).isoformat()
    if path.is_symlink():
        try:
            snapshot["target"] = str(path.resolve(strict=False))
        except OSError as exc:
            snapshot["target_error"] = str(exc)
    if list_entries and path.is_dir():
        try:
            entries = sorted(child.name for child in path.iterdir())
        except OSError as exc:
            snapshot["entries_error"] = str(exc)
        else:
            snapshot["entries"] = entries[:max_entries]
            snapshot["entries_truncated"] = len(entries) > max_entries
            snapshot["entries_count"] = len(entries)
    return snapshot


def _dir_usage_snapshot(path: Path, *, max_entries: int = 8) -> dict[str, Any]:
    snapshot: dict[str, Any] = {"path": str(path), "exists": path.exists(), "is_dir": path.is_dir()}
    if not snapshot["exists"]:
        return snapshot

    try:
        stat_result = path.lstat()
    except OSError as exc:
        snapshot["error"] = str(exc)
        return snapshot

    snapshot["mode"] = oct(stat_result.st_mode & 0o7777)
    snapshot["uid"] = stat_result.st_uid
    snapshot["gid"] = stat_result.st_gid
    snapshot["size_bytes"] = stat_result.st_size
    if not path.is_dir():
        return snapshot

    try:
        du_proc = subprocess.run(
            ["du", "-h", "-d", "1", str(path)],
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except subprocess.TimeoutExpired as exc:
        snapshot["du_timeout"] = 5
        snapshot["du_partial_stdout"] = (exc.stdout or "").strip()[:2000]
        snapshot["du_partial_stderr"] = (exc.stderr or "").strip()[:2000]
        return snapshot
    except OSError as exc:
        snapshot["du_error"] = str(exc)
        return snapshot

    snapshot["du_returncode"] = du_proc.returncode
    if du_proc.stdout:
        try:
            sort_proc = subprocess.run(
                ["sort", "-h"],
                input=du_proc.stdout,
                check=False,
                capture_output=True,
                text=True,
                timeout=2,
            )
        except subprocess.TimeoutExpired as exc:
            snapshot["du_sort_timeout"] = 2
            snapshot["du_partial_sorted_stdout"] = (exc.stdout or "").strip()[:2000]
            snapshot["du_partial_sorted_stderr"] = (exc.stderr or "").strip()[:2000]
            return snapshot
        except OSError as exc:
            snapshot["du_sort_error"] = str(exc)
            return snapshot
        snapshot["sort_returncode"] = sort_proc.returncode
        du_lines = [line for line in sort_proc.stdout.splitlines() if line.strip()]
        snapshot["du_lines"] = du_lines[:max_entries]
        snapshot["du_truncated"] = len(du_lines) > max_entries
        snapshot["du_count"] = len(du_lines)
    if du_proc.stderr:
        snapshot["du_stderr"] = du_proc.stderr.strip()
    return snapshot


def _resolve_log_dir(root_dir: Path) -> Path:
    default_dir = root_dir / DEFAULT_TMP_DIR / "exec_mcp"
    env_dir = os.environ.get("EXEC_MCP_LOG_DIR", "").strip()
    if not env_dir:
        resolved = default_dir.resolve()
    else:
        candidate = Path(env_dir)
        if not candidate.is_absolute():
            candidate = root_dir / candidate
        resolved = candidate.resolve()

    root_resolved = root_dir.resolve()
    if not resolved.is_relative_to(root_resolved):
        raise ValueError(f"EXEC_MCP_LOG_DIR は root 配下のみ許可: {resolved}")

    try:
        resolved.mkdir(parents=True, exist_ok=True)
    except OSError:
        fallback = default_dir.resolve()
        fallback.mkdir(parents=True, exist_ok=True)
        logging.getLogger("exec_mcp").warning(
            "EXEC_MCP_LOG_DIR=%s を使えないため %s にフォールバックします",
            resolved,
            fallback,
        )
        resolved = fallback
    return resolved


def _write_log_file(
    *,
    root_dir: Path,
    tool_name: str,
    argv: list[str],
    cwd: Path,
    exit_code: int | None,
    duration_ms: int | None,
    stdout_full: str,
    stderr_full: str,
) -> str:
    log_dir = _resolve_log_dir(root_dir)
    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    safe_tool = "".join(
        ch if (ch.isalnum() or ch in ("-", "_")) else "_" for ch in tool_name
    )[:64] or "tool"
    path = log_dir / f"{safe_tool}_{ts}_{os.getpid()}.log"
    header = [
        f"tool={tool_name}",
        f"ts_utc={ts}",
        f"cwd={cwd}",
        f"argv={' '.join(argv)}",
        f"exit_code={exit_code}",
        f"duration_ms={duration_ms}",
    ]
    body = (
        "\n".join(header)
        + "\n\n--- stdout ---\n"
        + (stdout_full or "")
        + "\n\n--- stderr ---\n"
        + (stderr_full or "")
        + "\n"
    )
    path.write_text(body, encoding="utf-8", errors="replace")
    return str(path)


def _write_named_result_log(
    *,
    root_dir: Path,
    log_name: str,
    tool_name: str,
    cwd: Path,
    exit_code: int | None,
    duration_ms: int | None,
    command_summary: str,
    stdout_full: str,
    stderr_full: str,
) -> str:
    tmp_dir = (root_dir / DEFAULT_TMP_DIR).resolve()
    tmp_dir.mkdir(parents=True, exist_ok=True)
    safe_name = "".join(
        ch if (ch.isalnum() or ch in ("-", "_")) else "_" for ch in log_name
    )[:120] or "exec_mcp"
    path = tmp_dir / f"{safe_name}_test_result.txt"
    header = [
        f"tool={tool_name}",
        f"cwd={cwd}",
        f"command={command_summary}",
        f"exit_code={exit_code}",
        f"duration_ms={duration_ms}",
    ]
    body = (
        "\n".join(header)
        + "\n\n--- stdout ---\n"
        + (stdout_full or "")
        + "\n\n--- stderr ---\n"
        + (stderr_full or "")
        + "\n"
    )
    path.write_text(body, encoding="utf-8", errors="replace")
    return str(path)


def _register_active_run(
    *, tool_name: str, argv: list[str], cwd: Path, process: asyncio.subprocess.Process
) -> str:
    run_id = uuid.uuid4().hex[:12]
    active_run = ActiveRun(
        run_id=run_id,
        tool_name=tool_name,
        argv=argv,
        cwd=str(cwd),
        started_at_monotonic=time.monotonic(),
        started_at_utc=datetime.now(timezone.utc),
        process=process,
    )
    with _ACTIVE_RUNS_LOCK:
        _ACTIVE_RUNS[run_id] = active_run
    return run_id


def _unregister_active_run(run_id: str) -> ActiveRun | None:
    with _ACTIVE_RUNS_LOCK:
        return _ACTIVE_RUNS.pop(run_id, None)


def _snapshot_active_run(run: ActiveRun) -> dict[str, Any]:
    duration_ms = int((time.monotonic() - run.started_at_monotonic) * 1000)
    return {
        "run_id": run.run_id,
        "tool": run.tool_name,
        "argv": run.argv,
        "cwd": run.cwd,
        "pid": run.process.pid,
        "started_at": run.started_at_utc.isoformat(),
        "duration_ms": duration_ms,
        "cancel_requested": run.cancel_requested,
        "returncode": run.process.returncode,
    }


def _snapshot_active_runs() -> list[dict[str, Any]]:
    with _ACTIVE_RUNS_LOCK:
        runs = list(_ACTIVE_RUNS.values())
    return [_snapshot_active_run(run) for run in runs]


async def _signal_process_tree(process: asyncio.subprocess.Process, *, force: bool) -> None:
    if process.returncode is not None:
        return

    if os.name != "nt" and process.pid is not None:
        sig = signal.SIGKILL if force else signal.SIGTERM
        try:
            os.killpg(process.pid, sig)
        except ProcessLookupError:
            return
    else:
        if force:
            process.kill()
        else:
            process.terminate()


async def _stop_process_tree(
    process: asyncio.subprocess.Process,
    *,
    force: bool,
    wait_timeout_sec: float = 5.0,
) -> None:
    await _signal_process_tree(process, force=force)
    try:
        await asyncio.wait_for(process.wait(), timeout=wait_timeout_sec)
    except asyncio.TimeoutError:
        await _signal_process_tree(process, force=True)
        await process.wait()


async def _stop_active_runs(
    *, force: bool = False, run_id: str | None = None, tool_name: str | None = None
) -> list[dict[str, Any]]:
    with _ACTIVE_RUNS_LOCK:
        active_runs = list(_ACTIVE_RUNS.values())

    selected_runs = [
        run
        for run in active_runs
        if (run_id is None or run.run_id == run_id)
        and (tool_name is None or run.tool_name == tool_name)
    ]

    for run in selected_runs:
        run.cancel_requested = True

    await asyncio.gather(
        *[
            _stop_process_tree(run.process, force=force)
            for run in selected_runs
            if run.process.returncode is None
        ],
        return_exceptions=True,
    )
    return [_snapshot_active_run(run) for run in selected_runs]


def _codex_exec_env(root_dir: Path) -> dict[str, str]:
    env = os.environ.copy()
    cache_dir = _resolve_root_path(
        root_dir,
        env.get("CODEX_DOCKER_CACHE_DIR"),
        DEFAULT_CACHE_DIR,
        env_name="CODEX_DOCKER_CACHE_DIR",
        allow_outside_root=True,
    )
    codex_home = _validated_exec_path(
        root_dir, env.get("CODEX_HOME"), DEFAULT_CODEX_HOME, env_name="CODEX_HOME"
    )
    cargo_home = _validated_exec_path(
        root_dir, env.get("CARGO_HOME"), f"{DEFAULT_CACHE_DIR}/cargo", env_name="CARGO_HOME"
    )
    rustup_home = _validated_exec_path(
        root_dir,
        env.get("RUSTUP_HOME"),
        f"{DEFAULT_CACHE_DIR}/rustup",
        env_name="RUSTUP_HOME",
    )
    home_dir = _validated_exec_path(
        root_dir, env.get("HOME"), f"{DEFAULT_CACHE_DIR}/home", env_name="HOME"
    )
    target_dir = _validated_exec_path(
        root_dir,
        env.get("CARGO_TARGET_DIR") or env.get("CODEX_DOCKER_TARGET_DIR"),
        DEFAULT_TARGET_DIR,
        env_name="CARGO_TARGET_DIR",
    )

    expected_target_dev = env.get("EXEC_MCP_EXPECTED_CARGO_TARGET_DEV", "").strip()
    if not expected_target_dev:
        raise RuntimeError(
            "EXEC_MCP_EXPECTED_CARGO_TARGET_DEV must be set to validate CARGO_TARGET_DIR"
        )
    if not target_dir.is_symlink():
        raise RuntimeError(
            f"CARGO_TARGET_DIR must be a symlink to the validated external filesystem: {target_dir}"
        )
    try:
        actual_target_dev = str(target_dir.stat().st_dev)
    except OSError as exc:
        raise RuntimeError(
            f"could not determine filesystem device for CARGO_TARGET_DIR={target_dir}"
        ) from exc
    if actual_target_dev != expected_target_dev:
        raise RuntimeError(
            f"CARGO_TARGET_DIR={target_dir} is on filesystem device "
            f"{actual_target_dev!r}, expected {expected_target_dev!r}"
        )

    for path in (cache_dir, cargo_home, rustup_home, home_dir, codex_home, target_dir):
        path.mkdir(parents=True, exist_ok=True)

    env["CODEX_HOME"] = str(codex_home)
    env["CODEX_SHELL_STARTUP_FILES"] = env.get("CODEX_SHELL_STARTUP_FILES", "clean")
    env["RUST_BACKTRACE"] = env.get("RUST_BACKTRACE", "1")
    env["CARGO_HOME"] = str(cargo_home)
    env["RUSTUP_HOME"] = str(rustup_home)
    env["HOME"] = str(home_dir)
    env["CARGO_TARGET_DIR"] = str(target_dir)
    env["USER"] = env.get("USER", "ubuntu")
    env["USERNAME"] = env.get("USERNAME", env["USER"])
    env["LOGNAME"] = env.get("LOGNAME", env["USER"])
    env["CARGO_BUILD_JOBS"] = env.get("CARGO_BUILD_JOBS", "4")
    env["CARGO_PROFILE_DEV_DEBUG"] = env.get("CARGO_PROFILE_DEV_DEBUG", "1")
    env["CARGO_PROFILE_TEST_DEBUG"] = env.get("CARGO_PROFILE_TEST_DEBUG", "1")
    env["CARGO_PROFILE_DEV_CODEGEN_UNITS"] = env.get(
        "CARGO_PROFILE_DEV_CODEGEN_UNITS", "16"
    )
    env["CARGO_PROFILE_TEST_CODEGEN_UNITS"] = env.get(
        "CARGO_PROFILE_TEST_CODEGEN_UNITS", "16"
    )
    env["CARGO_PROFILE_DEV_LTO"] = env.get("CARGO_PROFILE_DEV_LTO", "off")
    env["CARGO_PROFILE_TEST_LTO"] = env.get("CARGO_PROFILE_TEST_LTO", "off")
    env["CARGO_PROFILE_DEV_INCREMENTAL"] = env.get(
        "CARGO_PROFILE_DEV_INCREMENTAL", "false"
    )
    env["CARGO_PROFILE_TEST_INCREMENTAL"] = env.get(
        "CARGO_PROFILE_TEST_INCREMENTAL", "false"
    )
    env["CARGO_PROFILE_DEV_OPT_LEVEL"] = env.get("CARGO_PROFILE_DEV_OPT_LEVEL", "0")
    env["CARGO_PROFILE_TEST_OPT_LEVEL"] = env.get("CARGO_PROFILE_TEST_OPT_LEVEL", "0")
    return env


def _codex_env_report(root_dir: Path) -> dict[str, Any]:
    raw_env = os.environ.copy()
    resolved_root = root_dir.resolve()
    cache_dir = _resolve_root_path(
        root_dir,
        raw_env.get("CODEX_DOCKER_CACHE_DIR"),
        DEFAULT_CACHE_DIR,
        env_name="CODEX_DOCKER_CACHE_DIR",
        allow_outside_root=True,
    )
    codex_home = _validated_exec_path(
        root_dir, raw_env.get("CODEX_HOME"), DEFAULT_CODEX_HOME, env_name="CODEX_HOME"
    )
    cargo_home = _validated_exec_path(
        root_dir,
        raw_env.get("CARGO_HOME"),
        f"{DEFAULT_CACHE_DIR}/cargo",
        env_name="CARGO_HOME",
    )
    rustup_home = _validated_exec_path(
        root_dir,
        raw_env.get("RUSTUP_HOME"),
        f"{DEFAULT_CACHE_DIR}/rustup",
        env_name="RUSTUP_HOME",
    )
    home_dir = _validated_exec_path(
        root_dir, raw_env.get("HOME"), f"{DEFAULT_CACHE_DIR}/home", env_name="HOME"
    )
    target_dir = _validated_exec_path(
        root_dir,
        raw_env.get("CARGO_TARGET_DIR") or raw_env.get("CODEX_DOCKER_TARGET_DIR"),
        DEFAULT_TARGET_DIR,
        env_name="CARGO_TARGET_DIR",
    )

    return {
        "root_dir": str(resolved_root),
        "build_dir": str(_resolve_cwd(root_dir, DEFAULT_CODEX_RS_DIR)),
        "cache_root": str(cache_dir),
        "codex_home": str(codex_home),
        "cargo_home": str(cargo_home),
        "rustup_home": str(rustup_home),
        "home": str(home_dir),
        "target_dir": str(target_dir),
        "raw_env": {
            "CODEX_HOME": raw_env.get("CODEX_HOME"),
            "CARGO_HOME": raw_env.get("CARGO_HOME"),
            "RUSTUP_HOME": raw_env.get("RUSTUP_HOME"),
            "HOME": raw_env.get("HOME"),
            "CARGO_TARGET_DIR": raw_env.get("CARGO_TARGET_DIR"),
            "CODEX_DOCKER_TARGET_DIR": raw_env.get("CODEX_DOCKER_TARGET_DIR"),
            "CODEX_DOCKER_TARGET_DIR_IN_CONTAINER": raw_env.get(
                "CODEX_DOCKER_TARGET_DIR_IN_CONTAINER"
            ),
            "CODEX_SHELL_STARTUP_FILES": raw_env.get("CODEX_SHELL_STARTUP_FILES"),
            "RUST_BACKTRACE": raw_env.get("RUST_BACKTRACE"),
            "USER": raw_env.get("USER"),
            "USERNAME": raw_env.get("USERNAME"),
            "LOGNAME": raw_env.get("LOGNAME"),
        },
        "mounts": {
            "cargo": f"{cache_dir}/cargo -> /home/ubuntu/.cargo",
            "rustup": f"{cache_dir}/rustup -> /home/ubuntu/.rustup",
            "home": f"{cache_dir}/home -> /home/ubuntu",
            "target": f"{target_dir} -> /var/cache/codex/target",
        },
        "path_checks": {
            "cache_root": _path_snapshot(cache_dir, list_entries=True),
            "cache_cargo": _path_snapshot(cache_dir / "cargo"),
            "cache_rustup": _path_snapshot(cache_dir / "rustup"),
            "cache_home": _path_snapshot(cache_dir / "home"),
            "cache_target": _path_snapshot(cache_dir / "target"),
            "codex_home": _path_snapshot(codex_home),
            "cargo_home": _path_snapshot(cargo_home),
            "rustup_home": _path_snapshot(rustup_home),
            "home": _path_snapshot(home_dir, list_entries=True),
            "target_dir": _path_snapshot(target_dir, list_entries=True),
        },
        "du_checks": {
            "cache_root": _dir_usage_snapshot(cache_dir),
            "cache_cargo": _dir_usage_snapshot(cache_dir / "cargo"),
            "cache_rustup": _dir_usage_snapshot(cache_dir / "rustup"),
            "cache_home": _dir_usage_snapshot(cache_dir / "home"),
            "cache_target": _dir_usage_snapshot(cache_dir / "target"),
            "codex_home": _dir_usage_snapshot(codex_home),
            "home": _dir_usage_snapshot(home_dir),
            "target_dir": _dir_usage_snapshot(target_dir),
        },
    }


def _environment_probe(argv: list[str], *, timeout_sec: int = 5) -> dict[str, Any]:
    try:
        result = subprocess.run(
            argv,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout_sec,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired) as exc:
        return {"argv": argv, "ok": False, "error": type(exc).__name__}
    return {
        "argv": argv,
        "ok": result.returncode == 0,
        "returncode": result.returncode,
        "stdout": result.stdout.strip()[-4000:],
        "stderr": result.stderr.strip()[-4000:],
    }


def _test_environment_report(root_dir: Path) -> dict[str, Any]:
    env = os.environ
    target_dir = _validated_exec_path(
        root_dir,
        env.get("CARGO_TARGET_DIR") or env.get("CODEX_DOCKER_TARGET_DIR"),
        DEFAULT_TARGET_DIR,
        env_name="CARGO_TARGET_DIR",
    )
    status: dict[str, str] = {}
    try:
        for line in Path("/proc/self/status").read_text(encoding="utf-8").splitlines():
            key, separator, value = line.partition(":")
            if separator and key in {"CapEff", "CapBnd", "NoNewPrivs", "Seccomp"}:
                status[key] = value.strip()
    except OSError as exc:
        status["error"] = str(exc)

    namespaces = {}
    for name in ("mnt", "net", "user", "pid"):
        try:
            namespaces[name] = os.readlink(f"/proc/self/ns/{name}")
        except OSError as exc:
            namespaces[name] = f"error: {exc}"

    proxy_names = (
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
    )
    binaries = {
        name: shutil.which(name)
        for name in ("bwrap", "ip", "cargo", "rustc", "findmnt", "stat")
    }
    probes = []
    if binaries["bwrap"] and binaries["ip"]:
        probes.append(
            _environment_probe(
                [
                    binaries["bwrap"],
                    "--unshare-net",
                    "--ro-bind",
                    "/",
                    "/",
                    "--dev",
                    "/dev",
                    "--proc",
                    "/proc",
                    binaries["ip"],
                    "link",
                    "set",
                    "lo",
                    "up",
                ]
            )
        )
    for command in ((binaries["bwrap"], "--version"), (binaries["ip"], "-o", "link")):
        if command[0]:
            probes.append(_environment_probe(list(command)))

    target_snapshot = _path_snapshot(target_dir)
    target_dev = None
    try:
        target_dev = target_dir.stat().st_dev
    except OSError:
        pass
    return {
        "identity": {
            "uid": os.getuid(),
            "gid": os.getgid(),
            "user": env.get("USER"),
            "uname": platform.uname()._asdict(),
        },
        "test_environment": {
            name: env.get(name)
            for name in (
                "CODEX_TEST_ENVIRONMENT",
                "CODEX_SANDBOX",
                "CODEX_SANDBOX_NETWORK_DISABLED",
                "CARGO_BUILD_JOBS",
                "CARGO_TARGET_DIR",
                "EXEC_MCP_EXPECTED_CARGO_TARGET_DEV",
            )
        },
        "test_environment_presence": {
            name: bool(env.get(name))
            for name in (
                "CODEX_TEST_REMOTE_EXEC_SERVER_URL",
                "CODEX_TEST_REMOTE_ENV",
                "CODEX_TEST_REMOTE_ENV_CONTAINER_NAME",
            )
        },
        "proxy_environment": {name: bool(env.get(name)) for name in proxy_names},
        "security": {"status": status, "namespaces": namespaces},
        "binaries": binaries,
        "target": {
            "path": str(target_dir),
            "snapshot": target_snapshot,
            "st_dev": target_dev,
            "expected_st_dev": env.get("EXEC_MCP_EXPECTED_CARGO_TARGET_DEV"),
        },
        "probes": probes,
    }


def _summary_log_name(prefix: str, package: str | None = None, test_name: str | None = None) -> str:
    parts = [prefix]
    if package:
        parts.append(package)
    if test_name:
        parts.append(test_name)
    raw = "_".join(parts)
    collapsed = re.sub(r"[^0-9A-Za-z_-]+", "_", raw).strip("_")
    return collapsed[:120] or prefix


def _validate_non_empty(value: str, name: str) -> str:
    normalized = value.strip()
    if not normalized:
        raise ValueError(f"{name} は空にできません")
    if "\x00" in normalized or "\n" in normalized or "\r" in normalized:
        raise ValueError(f"{name} に改行や NUL は使えません")
    return normalized


async def _run_subprocess(
    *,
    tool_name: str,
    argv: list[str],
    cwd: Path,
    env: dict[str, str],
    timeout_sec: int,
) -> dict[str, Any]:
    if timeout_sec <= 0:
        raise ValueError("timeout_sec は 1 以上にしてください")

    log = logging.getLogger("exec_mcp")
    started = time.monotonic()
    log.info("tool_start tool=%s argv=%s cwd=%s", tool_name, argv, cwd)
    process: asyncio.subprocess.Process | None = None
    run_id: str | None = None
    try:
        process = await asyncio.create_subprocess_exec(
            *argv,
            cwd=str(cwd),
            env=env,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            start_new_session=os.name != "nt",
        )
        run_id = _register_active_run(
            tool_name=tool_name, argv=argv, cwd=cwd, process=process
        )
        try:
            stdout_bytes, stderr_bytes = await asyncio.wait_for(
                process.communicate(), timeout=timeout_sec
            )
            timed_out = False
        except asyncio.TimeoutError:
            await _stop_process_tree(process, force=False)
            stdout_bytes, stderr_bytes = await process.communicate()
            timed_out = True
        stdout_full = (stdout_bytes or b"").decode("utf-8", errors="replace")
        stderr_full = (stderr_bytes or b"").decode("utf-8", errors="replace")
        duration_ms = int((time.monotonic() - started) * 1000)
        active_run = _ACTIVE_RUNS.get(run_id) if run_id is not None else None
        return {
            "ok": process.returncode == 0 and not timed_out,
            "exit_code": 124 if timed_out else process.returncode,
            "duration_ms": duration_ms,
            "stdout_full": stdout_full,
            "stderr_full": stderr_full,
            "timeout": timed_out,
            "cancelled_by_stop": bool(active_run and active_run.cancel_requested),
            "error_type": None,
            "error_message": None,
            "run_id": run_id,
        }
    except asyncio.CancelledError:
        if process is not None:
            try:
                await _stop_process_tree(process, force=False)
            except Exception:
                log.exception("failed to stop cancelled tool=%s", tool_name)
        raise
    except OSError as exc:
        duration_ms = int((time.monotonic() - started) * 1000)
        stderr_full = f"{type(exc).__name__}: {exc}\n"
        log.error("tool_oserror tool=%s duration_ms=%s error=%s", tool_name, duration_ms, stderr_full.strip())
        return {
            "ok": False,
            "exit_code": 126,
            "duration_ms": duration_ms,
            "stdout_full": "",
            "stderr_full": stderr_full,
            "timeout": False,
            "error_type": type(exc).__name__,
            "error_message": str(exc),
            "run_id": run_id,
        }
    finally:
        if run_id is not None:
            _unregister_active_run(run_id)


def _response_from_single_run(
    *,
    tool_name: str,
    argv: list[str],
    cwd: Path,
    result: dict[str, Any],
    max_output_chars: int,
    log_name: str | None = None,
) -> dict[str, Any]:
    root_dir = _root_dir()
    stdout_full = result["stdout_full"]
    stderr_full = result["stderr_full"]
    stdout, stdout_truncated = _limit_text(stdout_full, max_output_chars)
    stderr, stderr_truncated = _limit_text(stderr_full, max_output_chars)

    log_file: str | None = None
    if log_name:
        log_file = _write_named_result_log(
            root_dir=root_dir,
            log_name=log_name,
            tool_name=tool_name,
            cwd=cwd,
            exit_code=result["exit_code"],
            duration_ms=result["duration_ms"],
            command_summary=" ".join(argv),
            stdout_full=stdout_full,
            stderr_full=stderr_full,
        )
    elif (not result["ok"]) or stdout_truncated or stderr_truncated:
        log_file = _write_log_file(
            root_dir=root_dir,
            tool_name=tool_name,
            argv=argv,
            cwd=cwd,
            exit_code=result["exit_code"],
            duration_ms=result["duration_ms"],
            stdout_full=stdout_full,
            stderr_full=stderr_full,
        )

    response: dict[str, Any] = {
        "ok": result["ok"],
        "exit_code": result["exit_code"],
        "tool": tool_name,
        "argv": argv,
        "cwd": str(cwd),
        "duration_ms": result["duration_ms"],
        "stdout": stdout,
        "stderr": stderr,
        "stdout_truncated": stdout_truncated,
        "stderr_truncated": stderr_truncated,
    }
    if result.get("run_id") is not None:
        response["run_id"] = result["run_id"]
    if result.get("cancelled_by_stop"):
        response["cancelled_by_stop"] = True
    if result["timeout"]:
        response["timeout"] = True
    if result["error_type"] is not None:
        response["error_type"] = result["error_type"]
        response["error_message"] = result["error_message"]
    if log_file is not None:
        response["log_file"] = log_file
    return response


async def _run_fixed(
    *,
    tool_name: str,
    argv: list[str],
    timeout_sec: int,
    max_output_chars: int,
) -> dict[str, Any]:
    root_dir = _root_dir()
    resolved_cwd = _resolve_cwd(root_dir, None)
    result = await _run_subprocess(
        tool_name=tool_name,
        argv=argv,
        cwd=resolved_cwd,
        env=os.environ.copy(),
        timeout_sec=timeout_sec,
    )
    return _response_from_single_run(
        tool_name=tool_name,
        argv=argv,
        cwd=resolved_cwd,
        result=result,
        max_output_chars=max_output_chars,
    )


async def _run_codex_command(
    *,
    tool_name: str,
    argv: list[str],
    cwd: str = DEFAULT_CODEX_RS_DIR,
    timeout_sec: int,
    max_output_chars: int = MAX_OUTPUT_CHARS,
    log_name: str | None = None,
) -> dict[str, Any]:
    root_dir = _root_dir()
    env = _codex_exec_env(root_dir)
    resolved_cwd = _resolve_cwd(root_dir, cwd)
    result = await _run_subprocess(
        tool_name=tool_name,
        argv=argv,
        cwd=resolved_cwd,
        env=env,
        timeout_sec=timeout_sec,
    )
    return _response_from_single_run(
        tool_name=tool_name,
        argv=argv,
        cwd=resolved_cwd,
        result=result,
        max_output_chars=max_output_chars,
        log_name=log_name,
    )


async def _run_codex_sequence(
    *,
    tool_name: str,
    steps: list[StepSpec],
    timeout_sec: int,
    log_name: str,
    max_output_chars: int = MAX_OUTPUT_CHARS,
) -> dict[str, Any]:
    root_dir = _root_dir()
    env = _codex_exec_env(root_dir)
    sequence_started = time.monotonic()
    status = 0
    timeout = False
    aggregate_stdout_parts: list[str] = []
    aggregate_stderr_parts: list[str] = []
    step_responses: list[dict[str, Any]] = []

    for step in steps:
        resolved_cwd = _resolve_cwd(root_dir, step.cwd)
        result = await _run_subprocess(
            tool_name=f"{tool_name}:{step.name}",
            argv=step.argv,
            cwd=resolved_cwd,
            env=env,
            timeout_sec=timeout_sec,
        )
        aggregate_stdout_parts.append(
            f"==> {step.name} ({' '.join(step.argv)})\n{result['stdout_full']}"
        )
        aggregate_stderr_parts.append(
            f"==> {step.name} ({' '.join(step.argv)})\n{result['stderr_full']}"
        )
        if not result["ok"]:
            status = result["exit_code"]
        if result["timeout"]:
            timeout = True
        if result.get("cancelled_by_stop"):
            status = result["exit_code"]
        step_response: dict[str, Any] = {
            "name": step.name,
            "argv": step.argv,
            "cwd": str(resolved_cwd),
            "ok": result["ok"],
            "exit_code": result["exit_code"],
            "duration_ms": result["duration_ms"],
        }
        if result["timeout"]:
            step_response["timeout"] = True
        if result["error_type"] is not None:
            step_response["error_type"] = result["error_type"]
            step_response["error_message"] = result["error_message"]
        step_responses.append(step_response)
        if result["timeout"] or result.get("cancelled_by_stop"):
            break

    duration_ms = int((time.monotonic() - sequence_started) * 1000)
    stdout_full = "\n".join(aggregate_stdout_parts)
    stderr_full = "\n".join(aggregate_stderr_parts)
    stdout, stdout_truncated = _limit_text(stdout_full, max_output_chars)
    stderr, stderr_truncated = _limit_text(stderr_full, max_output_chars)
    log_file = _write_named_result_log(
        root_dir=root_dir,
        log_name=log_name,
        tool_name=tool_name,
        cwd=_resolve_cwd(root_dir, DEFAULT_CODEX_RS_DIR),
        exit_code=status,
        duration_ms=duration_ms,
        command_summary=" && ".join(" ".join(step.argv) for step in steps),
        stdout_full=stdout_full,
        stderr_full=stderr_full,
    )

    response: dict[str, Any] = {
        "ok": status == 0,
        "exit_code": status,
        "tool": tool_name,
        "cwd": str(_resolve_cwd(root_dir, DEFAULT_CODEX_RS_DIR)),
        "duration_ms": duration_ms,
        "stdout": stdout,
        "stderr": stderr,
        "stdout_truncated": stdout_truncated,
        "stderr_truncated": stderr_truncated,
        "steps": step_responses,
        "log_file": log_file,
    }
    if timeout:
        response["timeout"] = True
    return response


def _linux_sandbox_step() -> StepSpec | None:
    if platform.system() != "Linux":
        return None
    return StepSpec(
        name="build-linux-sandbox",
        argv=["cargo", "build", "-p", "codex-linux-sandbox", "-p", "codex-bwrap"],
        cwd=DEFAULT_CODEX_RS_DIR,
    )


@mcp.tool
async def run_cargo_test() -> dict[str, Any]:
    """`cargo test` を実行して結果を返します（固定・引数なし）。"""
    return await _run_codex_command(
        tool_name="cargo_test",
        argv=["cargo", "test"],
        timeout_sec=60 * 30,
        max_output_chars=MAX_OUTPUT_CHARS,
        log_name="cargo_test",
    )


@mcp.tool
async def run_cargo_check() -> dict[str, Any]:
    """`cargo check` を実行して結果を返します（固定・引数なし）。"""
    return await _run_codex_command(
        tool_name="cargo_check",
        argv=["cargo", "check"],
        timeout_sec=60 * 10,
        max_output_chars=MAX_OUTPUT_CHARS,
        log_name="cargo_check",
    )


@mcp.tool
def check_env() -> dict[str, Any]:
    """Rust/compose のビルド環境として解決されるパス情報を返します。"""
    root_dir = _root_dir()
    report = _codex_env_report(root_dir)
    report["test_environment"] = _test_environment_report(root_dir)
    return report


@mcp.tool
async def run_cargo_build() -> dict[str, Any]:
    """`cargo build` を実行して結果を返します（固定・引数なし）。"""
    return await _run_codex_command(
        tool_name="cargo_build",
        argv=["cargo", "build"],
        timeout_sec=60 * 10,
        max_output_chars=MAX_OUTPUT_CHARS,
        log_name="cargo_build",
    )


@mcp.tool
async def run_cargo_fmt_check() -> dict[str, Any]:
    """`cargo fmt -- --check` を実行して結果を返します（固定・引数なし）。"""
    return await _run_codex_command(
        tool_name="cargo_fmt_check",
        argv=["cargo", "fmt", "--all", "--", "--check"],
        timeout_sec=60 * 10,
        max_output_chars=200_000,
        log_name="cargo_fmt_check",
    )


@mcp.tool
async def run_cargo_clippy() -> dict[str, Any]:
    """`cargo clippy -- -D warnings` を実行して結果を返します（固定・引数なし）。"""
    return await _run_codex_command(
        tool_name="cargo_clippy",
        argv=["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
        timeout_sec=60 * 20,
        max_output_chars=MAX_OUTPUT_CHARS,
        log_name="cargo_clippy",
    )


@mcp.tool
async def run_make_build_equivalent() -> dict[str, Any]:
    """`make build` 相当で codex CLI だけをビルドします。"""
    return await _run_codex_command(
        tool_name="make_build_equivalent",
        argv=["cargo", "build", "-p", "codex-cli", "--bin", "codex"],
        timeout_sec=60 * 20,
        log_name="build_cli",
    )


@mcp.tool
async def run_make_all_equivalent() -> dict[str, Any]:
    """`make all` 相当で `cargo +nightly fmt` と全機能テストを順に実行します。"""
    steps = [
        StepSpec(
            name="fmt",
            argv=["cargo", "+nightly", "fmt"],
            cwd=DEFAULT_CODEX_RS_DIR,
        )
    ]
    linux_sandbox = _linux_sandbox_step()
    if linux_sandbox is not None:
        steps.append(linux_sandbox)
    steps.append(
        StepSpec(
            name="test-all",
            argv=["cargo", "test", "--all-features"],
            cwd=DEFAULT_CODEX_RS_DIR,
        )
    )
    return await _run_codex_sequence(
        tool_name="make_all_equivalent",
        steps=steps,
        timeout_sec=60 * 60,
        log_name="all",
    )


@mcp.tool
async def run_make_almost_equivalent() -> dict[str, Any]:
    """`make almost` 相当で `cargo +nightly fmt` と `test-almost` を順に実行します。"""
    skip_tests = _load_almost_skip_tests()
    steps = [
        StepSpec(
            name="fmt",
            argv=["cargo", "+nightly", "fmt"],
            cwd=DEFAULT_CODEX_RS_DIR,
        )
    ]
    linux_sandbox = _linux_sandbox_step()
    if linux_sandbox is not None:
        steps.append(linux_sandbox)
    steps.append(
        StepSpec(
            name="test-almost",
            argv=[
                "cargo",
                "test",
                "--",
                *[item for test in skip_tests for item in ("--skip", test)],
            ],
            cwd=DEFAULT_CODEX_RS_DIR,
        )
    )
    return await _run_codex_sequence(
        tool_name="make_almost_equivalent",
        steps=steps,
        timeout_sec=60 * 60,
        log_name="almost",
    )


@mcp.tool
async def run_cargo_test_selected(
    package: str,
    test_name: str,
    target_kind: str = "auto",
    target_name: str | None = None,
    all_features: bool = False,
    exact: bool = False,
    build_linux_sandbox: bool = False,
) -> dict[str, Any]:
    """特定の Rust テストだけを実行します。"""
    normalized_package = _validate_non_empty(package, "package")
    normalized_test_name = _validate_non_empty(test_name, "test_name")
    normalized_target_name = (
        _validate_non_empty(target_name, "target_name")
        if target_name is not None
        else None
    )

    if target_kind not in {"auto", "lib", "test", "bin"}:
        raise ValueError("target_kind は auto/lib/test/bin のいずれかにしてください")
    if target_kind in {"test", "bin"} and normalized_target_name is None:
        raise ValueError("target_kind が test/bin の場合は target_name が必要です")
    if target_kind in {"auto", "lib"} and normalized_target_name is not None:
        raise ValueError("target_kind が auto/lib の場合は target_name を指定できません")

    argv = ["cargo", "test", "-p", normalized_package]
    if all_features:
        argv.append("--all-features")
    if target_kind == "lib":
        argv.append("--lib")
    elif target_kind == "test":
        argv.extend(["--test", normalized_target_name])
    elif target_kind == "bin":
        argv.extend(["--bin", normalized_target_name])
    argv.append(normalized_test_name)
    if exact:
        argv.extend(["--", "--exact"])

    log_name = _summary_log_name("selected_test", normalized_package, normalized_test_name)
    if build_linux_sandbox and platform.system() == "Linux":
        steps = [_linux_sandbox_step()]
        steps.append(
            StepSpec(
                name="selected-test",
                argv=argv,
                cwd=DEFAULT_CODEX_RS_DIR,
            )
        )
        return await _run_codex_sequence(
            tool_name="cargo_test_selected",
            steps=[step for step in steps if step is not None],
            timeout_sec=60 * 30,
            log_name=log_name,
        )

    return await _run_codex_command(
        tool_name="cargo_test_selected",
        argv=argv,
        timeout_sec=60 * 30,
        log_name=log_name,
    )


@mcp.tool
def list_active_runs() -> dict[str, Any]:
    """実行中の build/test ジョブを一覧します。"""
    return {"active_runs": _snapshot_active_runs()}


@mcp.tool
async def stop_active_runs(
    run_id: str | None = None,
    tool_name: str | None = None,
    force: bool = False,
) -> dict[str, Any]:
    """実行中の build/test ジョブを外から停止します。"""
    stopped_runs = await _stop_active_runs(
        force=force, run_id=run_id, tool_name=tool_name
    )
    return {
        "ok": True,
        "force": force,
        "run_id": run_id,
        "tool_name": tool_name,
        "stopped_runs": stopped_runs,
        "active_runs": _snapshot_active_runs(),
    }


def main() -> None:
    _configure_logging()
    parser = argparse.ArgumentParser(description="exec_mcp (FastMCP) server")
    parser.add_argument(
        "--transport",
        default=os.environ.get("EXEC_MCP_TRANSPORT", "http"),
        choices=["stdio", "http", "sse"],
    )
    parser.add_argument("--host", default=os.environ.get("EXEC_MCP_HOST", "0.0.0.0"))
    parser.add_argument(
        "--port", type=int, default=int(os.environ.get("EXEC_MCP_PORT", "9765"))
    )
    parser.add_argument("--path", default=os.environ.get("EXEC_MCP_PATH", "/mcp/"))
    args = parser.parse_args()

    if args.transport == "stdio":
        mcp.run(transport="stdio")
        return

    if args.transport == "sse":
        mcp.run(transport="sse", host=args.host, port=args.port)
        return

    mcp.run(transport="http", host=args.host, port=args.port, path=args.path)


if __name__ == "__main__":
    main()
