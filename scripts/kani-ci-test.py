import copy
import importlib.util
from pathlib import Path
import sys
import unittest


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


if __name__ == "__main__":
    unittest.main()
