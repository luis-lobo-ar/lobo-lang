"""Behavioral tests for the isolated context treatments, before participant runs."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

HERE = Path(__file__).parent
spec = importlib.util.spec_from_file_location('context_study', HERE/'context.py')
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)


class ContextTests(unittest.TestCase):
    def test_original_builder_matches_frozen_prompts(self):
        snapshot = json.loads((c.ROOT/'research/snapshots/2026-11-23-comprehension-audit/prompts.json').read_text())
        for row in snapshot['prompts']:
            files = row['initial_files']
            self.assertEqual(c.prompt(files, 'original', row['family']), row['prompt'])

    def test_dedup_removes_only_duplicate_blocks(self):
        for family in c.BASE.FAMILIES:
            for language in c.BASE.LANGUAGES:
                files = c.bundle(family, language)
                original = c.prompt(files, 'original', family)
                dedup = c.prompt(files, 'dedup', family)
                removed = '\n'.join('FILE '+name+'\n'+files[name] for name in ('TASK.txt', 'GUIDE.txt'))+'\n'
                self.assertEqual(original.replace(removed, '', 1), dedup)
                self.assertEqual(original.count(files['TASK.txt']), 2)
                self.assertEqual(dedup.count(files['TASK.txt']), 1)
                self.assertIn(files['check.py'], dedup)
                self.assertIn(files['AGENTS.md'], dedup)

    def test_schema_is_only_added_once_and_language_neutral(self):
        for family in c.BASE.FAMILIES:
            for language in c.BASE.LANGUAGES:
                files = c.bundle(family, language)
                for plain, treated in [('original', 'schema'), ('dedup', 'both')]:
                    self.assertEqual(c.prompt(files, treated, family), c.prompt(files, plain, family)+'\n'+c.input_schema(family))
                    self.assertEqual(c.prompt(files, treated, family).count(c.input_schema(family)), 1)
            self.assertNotIn('predictions', c.input_schema(family))

    def test_identity_values_stay_stable(self):
        for family in c.BASE.FAMILIES:
            old = json.loads(c.BASE.sources(family, 'lobo')['logic.lobo.ids.json'])
            new = json.loads(c.bundle(family, 'lobo')['logic.lobo.ids.json'])
            self.assertEqual(sorted(old['identities'].values()), sorted(new['identities'].values()))

    def test_questions_and_targets_are_fresh(self):
        for family in c.BASE.FAMILIES:
            old = [json.loads(json.dumps(x)) for x in c.BASE.prediction_inputs(family)]
            self.assertTrue(all(x not in old for x in c.inputs(family)))
            self.assertNotEqual(c.target(family), {'ledger': {'balance':7,'rejected':1,'peak':9}, 'scheduler': {'order':[1,0],'spent':3,'blocked':0}, 'frames': {'frames':['01','02'],'consumed':6,'error':0}}[family])
            self.assertEqual(len(c.inputs(family)), 6)

    def test_positive_and_negative_grading(self):
        for family in c.BASE.FAMILIES:
            answer = c.correct(family)
            self.assertTrue(c.grade(family, answer)['accepted'])
            for key in answer:
                wrong = copy.deepcopy(answer); wrong[key] = None
                self.assertFalse(c.grade(family, wrong)['accepted'], (family, key))
            wrong = copy.deepcopy(answer)
            first = next(k for k,v in wrong['predictions'][0].items() if type(v) is int)
            wrong['predictions'][0][first] = float(wrong['predictions'][0][first])
            self.assertFalse(c.grade(family, wrong)['accepted'])

    def test_new_oracles_distinguish_old_programs(self):
        for family in c.BASE.FAMILIES:
            self.assertTrue(any(c.oracle(family, x) != c.BASE.oracle(family, x) for x in c.inputs(family)))
            counter = c.correct(family)['counterexample']
            self.assertNotEqual(c.oracle(family, counter), c.oracle(family, counter, mutant=True))

    def test_schedule_balanced_and_counterbalanced(self):
        rows = c.schedule()
        self.assertEqual(len(rows), 72)
        for effort in ('high','low'):
            cohort = [r for r in rows if r['reasoning']==effort]
            self.assertEqual(len(cohort), 36)
            for family in c.BASE.FAMILIES:
                for language in c.BASE.LANGUAGES:
                    block = [r for r in cohort if (r['family'],r['language'])==(family,language)]
                    self.assertEqual(set(r['arm'] for r in block), set(c.ARMS))
        self.assertTrue(all(r['reasoning']=='high' for r in rows[:36]))

    def test_arm_positions_are_balanced(self):
        for effort in ('high','low'):
            rows=[r for r in c.schedule() if r['reasoning']==effort]
            for position in range(4):
                counts=[sum(rows[i]['arm']==arm for i in range(position,len(rows),4)) for arm in c.ARMS]
                self.assertLessEqual(max(counts)-min(counts),1)

    def test_command_keeps_read_only_and_shell_disabled(self):
        for effort in ('high','low'):
            cmd=c.command(Path('/tmp/isolated'),effort,Path('/tmp/last'))
            self.assertIn('read-only',cmd)
            self.assertIn('shell_tool',cmd)
            self.assertIn('model_reasoning_effort="'+effort+'"',cmd)
            self.assertEqual(cmd[cmd.index('-m')+1],'gpt-5.6-luna')


if __name__ == '__main__': unittest.main()
