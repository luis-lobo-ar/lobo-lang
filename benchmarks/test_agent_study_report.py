import importlib.util
import pathlib
import unittest

SPEC = importlib.util.spec_from_file_location('reporter', pathlib.Path(__file__).with_name('agent-study-report.py'))
reporter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(reporter)


class ReportTests(unittest.TestCase):
    def test_printed_client_source_is_not_a_transport_timeout(self):
        events = [{'type': 'item.completed', 'item': {'type': 'command_execution', 'exit_code': 0,
                   'aggregated_output': 'print("Check adapter deadline", file=sys.stderr)'}}]
        self.assertEqual(reporter.transport_deadlines(events), 0)
        events.append({'type': 'item.completed', 'item': {'type': 'command_execution', 'exit_code': 2,
                       'aggregated_output': 'Check adapter deadline\n'}})
        self.assertEqual(reporter.transport_deadlines(events), 1)

    def test_compaction_preserves_verification_outcomes_and_feedback_metrics(self):
        event = {'args': ['--detail', 'tree.4'], 'feedback_bytes': 10000, 'seconds': 0.1,
                 'result': {'status': 'ok', 'snapshot': 's2', 'bundle': {'sources': [{'text': 'large'}]},
                            'test': {'id': 'tree.4'}, 'result': {'status': 'failed'}}}
        compact = reporter.compact_event(event)
        self.assertEqual(compact['feedback_bytes'], 10000)
        self.assertNotIn('bundle', compact['result'])
        verify = {'args': [], 'result': {'status': 'failed', 'counts': {'failed': 2}}}
        self.assertEqual(reporter.compact_event(verify), verify)

    def test_audit_flags_global_context_but_not_shell_binary_or_task_paths(self):
        commands = [{'type': 'item.completed', 'item': {'type': 'command_execution',
                     'command': '/usr/bin/zsh -lc "cat /home/user/.codex/skills/a/SKILL.md"'}},
                    {'type': 'item.completed', 'item': {'type': 'command_execution',
                     'command': '/usr/bin/zsh -lc "cat /tmp/task/tree.lobo"'}}]
        audit = reporter.audit_events(commands, pathlib.Path('/tmp/task'))
        self.assertEqual(audit['external_path_references'], ['/home/user/.codex/skills/a/SKILL.md'])

    def test_unknown_interrupted_latency_is_not_a_fast_repair(self):
        spec = importlib.util.spec_from_file_location('study', pathlib.Path(__file__).with_name('agent-repair-study.py'))
        study = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(study)
        rows = [{'repeat': i, 'task': 'collections', 'arm': arm, 'accepted': True,
                 'elapsed_seconds': 10+j, 'timeout': False, 'usage': None, 'events': []}
                for i in range(5) for j, arm in enumerate(study.ARMS)]
        rows[1].update(accepted=False, elapsed_seconds=None, infrastructure_error='interrupted')
        analysis = reporter.reanalyze(study, rows)
        group = analysis['by_task_arm']['collections/json_full']
        self.assertEqual(group['accepted'], 4)
        self.assertEqual(group['unknown_latency_trials'], 1)
        self.assertEqual(group['elapsed_seconds_all']['observed'], 4)
        pair = analysis['paired']['collections/json_full-minus-text_full']
        self.assertEqual(pair['complete_pairs'], 5)
        self.assertEqual(pair['known_latency_pairs'], 4)


if __name__ == '__main__':
    unittest.main()
