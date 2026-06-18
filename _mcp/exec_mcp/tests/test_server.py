from __future__ import annotations

import asyncio
import os
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from server import _root_dir, _snapshot_active_runs, _run_subprocess, stop_active_runs


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
