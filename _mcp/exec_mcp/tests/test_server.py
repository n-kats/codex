from __future__ import annotations

import asyncio
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from server import (
    _load_almost_skip_tests,
    _root_dir,
    _run_subprocess,
    _snapshot_active_runs,
    _validated_exec_path,
    stop_active_runs,
)


class ExecMcpServerTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        await stop_active_runs(force=True)

    async def asyncTearDown(self) -> None:
        await stop_active_runs(force=True)

    async def test_stop_active_runs_stops_long_running_process(self) -> None:
        env = os.environ.copy()
        command = [
            "python3",
            "-c",
            "import time; time.sleep(30)",
        ]
        task = asyncio.create_task(
            _run_subprocess(
                tool_name="test_sleep",
                argv=command,
                cwd=_root_dir(),
                env=env,
                timeout_sec=60,
            )
        )

        try:
            for _ in range(40):
                if _snapshot_active_runs():
                    break
                await asyncio.sleep(0.05)

            self.assertTrue(_snapshot_active_runs())

            stopped = await stop_active_runs(tool_name="test_sleep")
            self.assertEqual(len(stopped["stopped_runs"]), 1)
            self.assertEqual(stopped["stopped_runs"][0]["tool"], "test_sleep")

            result = await asyncio.wait_for(task, timeout=10)
            self.assertTrue(result["cancelled_by_stop"])
            self.assertFalse(result["timeout"])
            self.assertNotEqual(result["exit_code"], 0)
            self.assertEqual(_snapshot_active_runs(), [])
        finally:
            if not task.done():
                await stop_active_runs(force=True, tool_name="test_sleep")
                await asyncio.wait_for(task, timeout=10)

    def test_resolve_exec_path_preserves_symlink_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root_dir = Path(temp_dir)
            real_target_dir = root_dir / "real-target"
            link_target_dir = root_dir / "link-target"
            real_target_dir.mkdir()
            os.symlink(real_target_dir, link_target_dir)

            resolved = _validated_exec_path(
                root_dir,
                str(link_target_dir),
                "_tmp/default-target",
                env_name="CARGO_TARGET_DIR",
            )

            self.assertEqual(resolved, link_target_dir)
            self.assertTrue(resolved.is_symlink())

    def test_load_almost_skip_tests_includes_flaky_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root_dir = Path(temp_dir)
            (root_dir / "skip_test_list.txt").write_text(
                "skip_one\n# comment\n", encoding="utf-8"
            )
            (root_dir / "flaky_test_list.txt").write_text(
                "flaky_one\nflaky_two # inline comment\n", encoding="utf-8"
            )

            with patch("server._root_dir", return_value=root_dir):
                self.assertEqual(
                    _load_almost_skip_tests(),
                    ("skip_one", "flaky_one", "flaky_two"),
                )
