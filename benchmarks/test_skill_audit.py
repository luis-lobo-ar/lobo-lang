import importlib.util
import pathlib
import unittest

SPEC = importlib.util.spec_from_file_location('audit', pathlib.Path(__file__).with_name('skill-evaluation-audit.py'))
audit = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(audit)


class AuditTests(unittest.TestCase):
    def test_parent_instruction_probes_are_distinct_from_permitted_skill_reads(self):
        events = [{'type': 'item.completed', 'item': {'type': 'command_execution', 'command': c}} for c in [
            "cat /home/luis/.codex/skills/solid/SKILL.md", "rg --files -g AGENTS.md . ..",
            "if test -f /tmp/AGENTS.md; then cat /tmp/AGENTS.md; fi"]]
        result = audit.inspect(events, pathlib.Path('/tmp/task'))
        self.assertEqual(len(result['parent_scope_probes']), 2)
        self.assertEqual(result['outside_file_changes'], [])


if __name__ == '__main__':
    unittest.main()
