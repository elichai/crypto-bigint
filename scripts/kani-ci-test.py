import copy
from contextlib import redirect_stdout
import importlib.util
import io
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch


sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("kani_ci", Path(__file__).with_name("kani-ci.py"))
ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci)


class RunnerTests(unittest.TestCase):
    def test_all_new_harnesses_are_selected_once(self):
        names = [f"module::proof_{index}" for index in range(100)]
        pending = [{"harness": names[0], "words": [32], "reason": "timeout"}]
        shards = [ci.select_harnesses(names, pending, 32, "gate", shard, 4)[0]
                  for shard in range(4)]
        flattened = sum(shards, [])
        self.assertCountEqual(flattened, names[1:])
        self.assertEqual(len(flattened), len(set(flattened)))
        self.assertEqual(ci.select_harnesses(names, pending, 32, "pending", 0, 1)[0], names[:1])
        self.assertCountEqual(ci.select_harnesses(names, pending, 64, "gate", 0, 1)[0], names)

    def test_stale_or_duplicate_quarantine_fails(self):
        item = {"harness": "proof", "words": [32, 64], "reason": "timeout"}
        with self.assertRaises(ValueError):
            ci.select_harnesses(["different"], [item], 64, "gate", 0, 1)
        with self.assertRaises(ValueError):
            ci.select_harnesses(["proof"], [item, item], 64, "gate", 0, 1)

    def payload(self):
        return {"verification_results": {
            "summary": {"status": "completed", "executed": 1, "successful": 1, "failed": 0},
            "results": [{"harness_id": "proof", "status": "Success"}]},
            "error_details": [{"harness_id": "proof", "has_errors": False}]}

    def test_success_requires_exit_status_and_complete_results(self):
        payload = self.payload()
        self.assertEqual(ci.validate_results(payload, ["proof"], 0), [])
        with self.assertRaises(ValueError):
            ci.validate_results(payload, ["proof"], 1)
        with self.assertRaises(ValueError):
            ci.validate_results(payload, ["proof", "new_proof"], 0)
        payload["verification_results"]["results"] *= 2
        with self.assertRaises(ValueError):
            ci.validate_results(payload, ["proof"], 0)
        payload = self.payload()
        payload["error_details"] *= 2
        with self.assertRaises(ValueError):
            ci.validate_results(payload, ["proof"], 0)

    def test_timeout_tool_error_and_counterexample_never_pass(self):
        for status in ["Failure", "Undetermined", "Timeout"]:
            payload = self.payload()
            payload["verification_results"]["results"][0]["status"] = status
            self.assertEqual(ci.validate_results(payload, ["proof"], 1), ["proof"])
        payload = copy.deepcopy(self.payload())
        payload["error_details"][0]["has_errors"] = True
        self.assertEqual(ci.validate_results(payload, ["proof"], 0), ["proof"])

    def test_run_streams_split_progress_preserves_log_and_nonzero_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            script = Path(directory) / "verifier.py"
            acknowledged = Path(directory) / "progress-seen"
            script.write_text("from pathlib import Path\nimport sys, time\n"
                              "sys.stdout.write('Checking har'); sys.stdout.flush()\n"
                              "time.sleep(0.5)\n"
                              "print('ness module::proof...', flush=True)\n"
                              "while not Path(sys.argv[1]).exists(): time.sleep(0.01)\n"
                              "print('solver detail retained only in file')\n"
                              "print('VERIFICATION:- FAILED')\n"
                              "print('CBMC failed with status 10')\n"
                              "sys.stdout.write('Verification Time: 0.1s')\n"
                              "sys.exit(7)\n")
            log = Path(directory) / "verification.log"
            class ProgressConsole(io.StringIO):
                def write(self, text):
                    if text.startswith("Checking harness"):
                        acknowledged.touch()
                    return super().write(text)

            console = ProgressConsole()
            with redirect_stdout(console):
                code = ci.run([sys.executable, str(script), str(acknowledged)],
                              os.environ.copy(), log, 5)
            self.assertEqual(code, 7)
            self.assertIn("Checking harness module::proof...", console.getvalue())
            self.assertIn("VERIFICATION:- FAILED", console.getvalue())
            self.assertIn("CBMC failed with status 10", console.getvalue())
            self.assertIn("Verification Time: 0.1s", console.getvalue())
            self.assertNotIn("solver detail", console.getvalue())
            self.assertIn("solver detail retained only in file", log.read_text())

    def test_thread_start_failure_still_kills_and_reaps_verifier(self):
        with tempfile.TemporaryDirectory() as directory:
            script = Path(directory) / "verifier.py"
            script.write_text("import time\ntime.sleep(60)\n")
            command = [sys.executable, str(script)]
            process = subprocess.Popen(command, stdout=subprocess.DEVNULL, start_new_session=True)
            try:
                with patch.object(ci.subprocess, "Popen", return_value=process), \
                     patch.object(ci.threading.Thread, "start", side_effect=RuntimeError("no thread")), \
                     redirect_stdout(io.StringIO()), self.assertRaisesRegex(RuntimeError, "no thread"):
                    ci.run(command, os.environ.copy(), Path(directory) / "verification.log", 5)
                self.assertIsNotNone(process.poll(), "verifier must be reaped on thread failure")
                with self.assertRaises(ProcessLookupError):
                    os.kill(process.pid, 0)
            finally:
                if process.poll() is None:
                    os.killpg(process.pid, ci.signal.SIGKILL)
                process.wait()

    def test_run_timeout_kills_verifier_and_solver_descendant(self):
        with tempfile.TemporaryDirectory() as directory:
            solver = Path(directory) / "solver.py"
            solver.write_text("from pathlib import Path\nimport os, sys, time\n"
                              "Path(sys.argv[1]).write_text(str(os.getpid()))\n"
                              "time.sleep(2)\nPath(sys.argv[2]).touch()\n")
            verifier = Path(directory) / "verifier.py"
            verifier.write_text("from pathlib import Path\nimport os, subprocess, sys, time\n"
                                "Path(sys.argv[1]).write_text(str(os.getpid()))\n"
                                "subprocess.Popen([sys.executable, *sys.argv[2:]])\n"
                                "print('Checking harness timeout::proof...', flush=True)\n"
                                "time.sleep(60)\n")
            pid_file = Path(directory) / "verifier.pid"
            child_pid = Path(directory) / "solver.pid"
            child_marker = Path(directory) / "solver-survived"
            log = Path(directory) / "verification.log"
            console = io.StringIO()
            with redirect_stdout(console), self.assertRaises(subprocess.TimeoutExpired):
                ci.run([sys.executable, str(verifier), str(pid_file), str(solver),
                        str(child_pid), str(child_marker)], os.environ.copy(), log, 1)
            self.assertTrue(child_pid.exists(), "solver must start before timeout")
            with self.assertRaises(ProcessLookupError):
                os.kill(int(pid_file.read_text()), 0)
            time.sleep(2.1)
            self.assertFalse(child_marker.exists(), "solver must not outlive verifier cleanup")
            self.assertIn("Checking harness timeout::proof...", console.getvalue())


if __name__ == "__main__":
    unittest.main()
