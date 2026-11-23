import importlib.util
import pathlib
import re
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('skill_eval', pathlib.Path(__file__).with_name('skill-evaluation.py'))
evaluation = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(evaluation)


class EvaluationTests(unittest.TestCase):
    def test_skill_markdown_links_resolve(self):
        for path in evaluation.SKILL.rglob('*.md'):
            for target in re.findall(r'\]\(([^)#]+)', path.read_text()):
                if not re.match(r'[a-z]+:', target):
                    self.assertTrue((path.parent/target).exists(), (path, target))

    def test_schedule_is_twenty_attempts_with_five_per_task_arm(self):
        rows = evaluation.schedule()
        self.assertEqual(len(rows), 20)
        for name in evaluation.TASKS:
            for arm in ['guide', 'skill']:
                self.assertEqual(sum(t == name and a == arm for _, t, a in rows), 5)
        for i in range(0, 20, 2):
            self.assertEqual({a for _, _, a in rows[i:i+2]}, {'guide', 'skill'})

    def test_seeded_stubs_fail_and_independent_native_oracles_accept_repairs(self):
        for name, task in evaluation.TASKS.items():
            with evaluation.Session(task, evaluation.cases(name, True)) as session:
                self.assertEqual(session.verify(task['sources'], 'all')['status'], 'failed')
                self.assertEqual(session.verify(task['fixed'], 'all')['status'], 'checks_satisfied')

    def test_baseline_contains_all_skill_knowledge_and_specs_match(self):
        guide = evaluation.flat_guide()
        for path, contents in evaluation.skill_files().items():
            self.assertIn(contents, guide, path)
        self.assertNotIn('function encode', guide)
        self.assertNotIn('function clip', guide)
        self.assertEqual(evaluation.fixture_files('guide')['spec/core-v0.9.md'],
                         evaluation.fixture_files('skill')['spec/core-v0.9.md'])

    def test_checkpoints_are_atomic_and_started_attempts_are_not_replaced(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory)/'results.json'
            data = {'trials': [{'state': 'running'}]}
            evaluation.save(path, data)
            self.assertEqual(evaluation.read_report(path), data)
            with self.assertRaises(ValueError):
                evaluation.require_pending(data, 0)

    def test_native_skill_examples_exercise_records_variants_loops_and_text(self):
        evaluation.check_examples()

    def test_summary_retains_failures_and_unknown_usage(self):
        rows = [{'repeat': r, 'task': t, 'arm': a, 'state': 'complete', 'accepted': True,
                 'timeout': False, 'elapsed_seconds': 10, 'usage': None, 'events': [],
                 'fixture_integrity': True} for r, t, a in evaluation.schedule()]
        rows[0].update(accepted=False, elapsed_seconds=None, infrastructure_error='interrupted')
        group = evaluation.summarize({'trials': rows})['by_task_arm']['rle/guide']
        self.assertEqual(group['accepted'], 4)
        self.assertEqual(group['scheduled'], 5)
        self.assertEqual(group['input_tokens']['observed'], 0)
        self.assertEqual(group['elapsed_seconds_all']['observed'], 4)


if __name__ == '__main__':
    unittest.main()
