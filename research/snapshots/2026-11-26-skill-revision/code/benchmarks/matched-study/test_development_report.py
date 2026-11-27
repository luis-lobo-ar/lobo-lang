import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('development_report',Path(__file__).with_name('development_report.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
class ReportTests(unittest.TestCase):
    def row(self,repeat,arm,accepted,usage):
        return dict(assignment=dict(index=repeat*2+(arm=='skill'),family='splice',repeat=repeat,arm=arm),accepted=accepted,usage=usage,agent_wall_s=10,grading=dict(passed=70 if accepted else 1,total=70),command_tool_items=2,check_requests=1,status='completed')
    def test_pairs_include_repetitions(self):
        rows=[self.row(n,a,a=='skill',None) for n in range(2) for a in ('control','skill')]
        p=r.pairs(rows);self.assertEqual(p['pairs'],2);self.assertEqual(p['fail_to_pass'],2)
    def test_failure_cost_and_unknown_preserved(self):
        rows=[self.row(0,'control',False,dict(input_tokens=10,output_tokens=3,cached_input_tokens=0)),self.row(1,'control',True,None)]
        a=r.aggregate(rows);self.assertEqual(a['usage']['reported_input_plus_output_tokens'],13);self.assertEqual(a['usage']['unknown'],1);self.assertEqual(a['accepted'],1);self.assertEqual(a['attempted'],2)
