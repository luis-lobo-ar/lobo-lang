import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('revision_report',Path(__file__).with_name('skill_revision_report.py'));r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
class ReportTests(unittest.TestCase):
 def row(self,n,arm,ok,u=None):return dict(assignment=dict(index=n*3+('control','original','revised').index(arm),family='locate',repeat=n,arm=arm),accepted=ok,usage=u,agent_wall_s=10,grading=dict(passed=70 if ok else 0,total=70),command_tool_items=2,check_requests=1,status='completed')
 def test_direct_revision_pairs_keep_repeats(self):
  rows=[self.row(n,a,a=='revised') for n in range(2) for a in ('original','revised')];p=r.contrast(rows,'original','revised');self.assertEqual(p['pairs'],2);self.assertEqual(p['fail_to_pass'],2)
 def test_unknown_and_failed_costs_remain(self):
  a=r.aggregate([self.row(0,'original',False,dict(input_tokens=11,output_tokens=5)),self.row(1,'original',True)]);self.assertEqual(a['usage']['reported_input_plus_output_tokens'],16);self.assertEqual(a['usage']['unknown'],1);self.assertEqual(a['accepted'],1)
