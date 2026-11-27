import importlib.util,unittest
from pathlib import Path
sp=importlib.util.spec_from_file_location('v3report',Path(__file__).with_name('v3_report.py'));s=importlib.util.module_from_spec(sp);sp.loader.exec_module(s)
class ReportTests(unittest.TestCase):
 def test_pairing_unknown_and_correct_timeout(self):
  def row(i,arm,usage,accepted):return dict(assignment=dict(family='feed',repeat=i,arm=arm,index=i),usage=None if usage is None else dict(input_tokens=usage,output_tokens=0),agent_wall_s=100,accepted=accepted,status='completed' if accepted else 'wall_limit',grading=dict(accepted=True,passed=40,total=40))
  rows=[row(0,'v2',10,False),row(0,'requirements',20,True),row(1,'v2',10,True),row(1,'requirements',None,False)]
  result=s.contrast(rows,'requirements');self.assertEqual(result['matched_pairs'],2);self.assertEqual(result['complete_token_pairs'],1);self.assertEqual(result['median_token_ratio'],2);self.assertEqual(result['fail_to_pass'],1);self.assertEqual(result['pass_to_fail'],1)
 def test_strict_json_types(self):
  self.assertFalse(s.S.strict({'n':[1]},{'n':[True]}));self.assertTrue(s.S.strict({'n':[1]},{'n':[1]}))
 def test_baseline_substitution_does_not_mutate_rows(self):
  import copy
  def row(arm):return dict(assignment=dict(family='feed',repeat=0,arm=arm),usage=None,agent_wall_s=120,accepted=False,status='wall_limit',grading=dict(accepted=False,passed=0,total=40))
  rows=[row('v2'),row('finish')];before=copy.deepcopy(rows);s.contrast(rows,'finish');self.assertEqual(rows,before)
