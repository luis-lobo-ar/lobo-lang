import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('diagnostics',Path(__file__).with_name('learning_diagnostics.py'))
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)


class DiagnosticsTests(unittest.TestCase):
    def test_format_improvement_is_separate_from_task_acceptance(self):
        def row(arm,valid):return dict(assignment=dict(stage='format',reasoning='high',family='frames',language='lobo',arm=arm,index=0),accepted=False,grading=dict(format_valid=valid,passed=10,total=11))
        value=d.profile_pairs([row('ambiguous',False),row('explicit',True)],'format')
        self.assertEqual(value['pairs'],1);self.assertEqual(value['format_fail_to_pass'],1)
        self.assertEqual(value['acceptance_fail_to_pass'],0)

    def test_counterexample_requires_different_outputs_not_a_fixed_example(self):
        args={'initial':2,'events':[[0,0],[1,0]]}
        result=d.describe('ledger','counterexample',args)
        self.assertTrue(result['input_valid']);self.assertNotEqual(result['original'],result['mutant'])


if __name__=='__main__':unittest.main()
