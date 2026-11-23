"""Behavioral checks for the local real-agent pilot harness."""
import importlib.util
import pathlib
import tempfile
import subprocess
import sys
import unittest

SPEC = importlib.util.spec_from_file_location("pilot", pathlib.Path(__file__).with_name("agent-repair.py"))
pilot = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(pilot)


class PilotTests(unittest.TestCase):
    def test_fixture_guide_preserves_lobo_braces_and_substitutes_requirement(self):
        guide = pilot.guide_for("clamp")
        self.assertIn("{ expression }", guide)
        self.assertIn(pilot.TASKS["clamp"]["description"], guide)
        compile(pilot.CLIENT.format(address="/tmp/example.sock"), "check.py", "exec")

    def test_missing_usage_is_unknown_and_cached_tokens_stay_separate(self):
        self.assertIsNone(pilot.read_usage([]))
        usage = {"input_tokens": 100, "cached_input_tokens": 70, "output_tokens": 12}
        self.assertEqual(pilot.read_usage([{"type": "turn.completed", "usage": usage}]), usage)

    def test_alternates_order_without_dropping_tasks(self):
        schedule = pilot.schedule()
        self.assertEqual(len(schedule), 10)
        for i, task in enumerate(pilot.TASKS):
            self.assertEqual([t for t, _ in schedule[2*i:2*i+2]], [task, task])
            self.assertEqual([a for _, a in schedule[2*i:2*i+2]],
                             ["conventional", "structured"] if i % 2 == 0 else ["structured", "conventional"])

    def test_source_symlinks_are_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root = pathlib.Path(d)
            (root / "other").write_text("not a candidate")
            (root / "program.lobo").symlink_to(root / "other")
            with self.assertRaises(ValueError):
                pilot.read_source(root)

    def test_file_adapter_client_receives_real_native_failure_then_pass(self):
        with tempfile.TemporaryDirectory() as d:
            root = pathlib.Path(d)
            inbox = root / ".check-io"
            (root / "program.lobo").write_text(pilot.TASKS["clamp"]["source"])
            (root / "check.py").write_text(pilot.CLIENT.format(address=str(inbox)))
            adapter = pilot.Adapter(root, inbox, "clamp", "structured")
            try:
                self.assertTrue(inbox.is_dir(), "file transport must use a directory, not a socket")
                bad = subprocess.run([sys.executable, "check.py"], cwd=root, capture_output=True, text=True, timeout=45)
                self.assertEqual(bad.returncode, 1, bad.stderr)
                self.assertIn('"status":"failed"', bad.stdout)
                (root / "program.lobo").write_text(pilot.TASKS["clamp"]["fixed"])
                good = subprocess.run([sys.executable, "check.py"], cwd=root, capture_output=True, text=True, timeout=45)
                self.assertEqual(good.returncode, 0, good.stderr)
                self.assertEqual(len(adapter.checks), 2)
            finally:
                adapter.close()

    def test_seeded_defects_fail_but_repairs_pass_native_holdouts(self):
        for name, task in pilot.TASKS.items():
            with self.subTest(task=name):
                with pilot.Session(task, pilot.test_cases(name, heldout=True)) as session:
                    self.assertEqual(session.verify(task["source"], "all")["status"], "failed")
                    self.assertEqual(session.verify(task["fixed"], "all")["status"], "checks_satisfied")

    def test_reuse_does_not_hide_failed_task_and_full_reruns_everything(self):
        task = pilot.TASKS["clamp"]
        with pilot.Session(task, pilot.test_cases("clamp")) as session:
            first = session.verify(task["source"], "affected")
            self.assertEqual(first["status"], "failed")
            second = session.verify(task["fixed"], "affected")
            self.assertEqual(second["status"], "checks_satisfied")
            self.assertGreater(second["counts"]["reused"], 0)
            full = session.verify(task["fixed"], "all")
            self.assertTrue(full["full_execution"])
            self.assertEqual(full["counts"]["reused"], 0)


if __name__ == "__main__":
    unittest.main()
