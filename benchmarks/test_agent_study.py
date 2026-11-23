"""Native and transport behavior of the repeated library-repair study."""
import importlib.util
import pathlib
import subprocess
import sys
import tempfile
import unittest
from concurrent.futures import ThreadPoolExecutor

SPEC = importlib.util.spec_from_file_location("study", pathlib.Path(__file__).with_name("agent-repair-study.py"))
study = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(study)


class StudyTests(unittest.TestCase):
    def test_every_task_arm_has_five_repeats_and_order_rotates(self):
        rows = study.schedule()
        self.assertEqual(len(rows), 30)
        for name in study.TASKS:
            for arm in study.ARMS:
                self.assertEqual(sum(t == name and a == arm for _, t, a in rows), 5)
        for offset in range(0, 30, 3):
            self.assertEqual(set(a for _, _, a in rows[offset:offset+3]), set(study.ARMS))

    def test_all_three_seeded_defects_are_detected_independently(self):
        for name, task in study.TASKS.items():
            cases = study.cases(name, heldout=True)
            self.assertEqual(len(study.cases(name)), 64)
            self.assertLessEqual(len(cases), 256)
            with study.Session(task, cases) as session:
                for mutant in [task["sources"], *task["single_mutants"]]:
                    self.assertEqual(session.verify(mutant, "all")["status"], "failed", name)
                self.assertEqual(session.verify(task["fixed"], "all")["status"], "checks_satisfied", name)

    def test_partial_repair_invalidates_transitive_checks_without_hiding_other_faults(self):
        task = study.TASKS["collections"]
        with study.Session(task, study.cases("collections")) as session:
            first = session.verify(task["sources"], "affected")
            self.assertEqual(first["status"], "failed")
            partial = dict(task["sources"])
            partial["rules.lobo"] = task["fixed"]["rules.lobo"]
            middle = session.verify(partial, "affected")
            self.assertEqual(middle["status"], "failed")
            self.assertGreater(middle["counts"]["reused"], 0)
            last = session.verify(task["fixed"], "affected")
            self.assertEqual(last["status"], "checks_satisfied")
            full = session.verify(task["fixed"], "all")
            self.assertTrue(full["full_execution"])
            self.assertEqual(full["counts"]["executed"], 64)

    def test_detail_replay_and_usage_are_recorded_through_real_client(self):
        task = study.TASKS["tree"]
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            for name, source in task["sources"].items():
                (root/name).write_text(source)
            adapter = study.Adapter(root, task, "json_affected", study.cases("tree"))
            (root/"check.py").write_text(study.CLIENT.format(address=str(root/".check-io")))
            try:
                def invoke(*args):
                    return subprocess.run([sys.executable, "check.py", *args], cwd=root,
                                          capture_output=True, text=True, timeout=45)
                bad = invoke()
                self.assertEqual(bad.returncode, 1, bad.stderr)
                test = adapter.events[-1]["result"]["failures"][0]["test"]
                self.assertEqual(invoke("--detail", test).returncode, 0)
                replay = invoke("--replay", test)
                self.assertEqual(replay.returncode, 0, replay.stderr)
                self.assertIn('"reproduced":true', replay.stdout)
                with ThreadPoolExecutor(max_workers=4) as clients:
                    calls = [clients.submit(invoke, "--detail", test) for _ in range(4)]
                    self.assertTrue(all(call.result().returncode == 0 for call in calls))
                for name, source in task["fixed"].items():
                    (root/name).write_text(source)
                self.assertEqual(invoke().returncode, 0)
                self.assertEqual([e["args"] for e in adapter.events],
                                 [[], ["--detail", test], ["--replay", test]] + [["--detail", test]]*4 + [[]])
            finally:
                adapter.close()

    def test_paired_analysis_keeps_failed_trials_and_unknown_usage(self):
        rows = [{"repeat": i, "task": "collections", "arm": arm, "accepted": i != 0,
                 "elapsed_seconds": 10 + j, "timeout": i == 0, "usage": None, "events": []}
                for i in range(5) for j, arm in enumerate(study.ARMS)]
        result = study.analyze(rows)
        self.assertEqual(result["by_task_arm"]["collections/text_full"]["accepted"], 4)
        self.assertEqual(result["by_task_arm"]["collections/text_full"]["input_tokens"]["observed"], 0)
        self.assertEqual(result["paired"]["collections/json_full-minus-text_full"]["complete_pairs"], 5)


if __name__ == "__main__":
    unittest.main()
