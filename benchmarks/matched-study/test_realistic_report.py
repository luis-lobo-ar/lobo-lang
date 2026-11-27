import importlib.util,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('realistic_report',Path(__file__).with_name('realistic_report.py'));r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
class ReportTests(unittest.TestCase):
 def test_matching_uses_repetition_and_retains_unknown(self):
  def row(n,arm,t):return dict(assignment=dict(family='catalog',repeat=n,arm=arm),status='completed',grading=dict(accepted=True),usage=None if t is None else dict(input_tokens=t,output_tokens=0,cached_input_tokens=0),agent_wall_s=10,accepted=True)
  value=r.contrast([row(0,'control',10),row(0,'v2',20),row(1,'control',10),row(1,'v2',None)])
  self.assertEqual(value['matched_pairs'],2);self.assertEqual(value['complete_token_pairs'],1);self.assertEqual(value['median_token_ratio'],2)
 def test_criteria_separate_candidate_and_completion(self):
  row=dict(status='wall_limit',accepted=False,grading=dict(accepted=True))
  self.assertEqual(r.outcome(row),dict(timely_complete=False,candidate_correct=True,status='wall_limit'))
