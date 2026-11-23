import importlib.util
from pathlib import Path
import unittest
SPEC=importlib.util.spec_from_file_location('report',Path(__file__).with_name('matched-study')/'report.py')
m=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(m)
class ReportTests(unittest.TestCase):
 def test_unknown_usage_stays_unknown_and_cache_is_not_added(self):
  value=m.account([{'usage':{'input_tokens':100,'cached_input_tokens':80,'output_tokens':5}},{'usage':None}])
  self.assertEqual(value['known'],1);self.assertEqual(value['unknown'],1)
  self.assertEqual(value['reported_input_plus_output_tokens'],105)
  self.assertEqual(value['reported_uncached_input_tokens'],20)
  self.assertFalse(value['complete'])
 def test_failed_attempt_remains_in_the_group_denominator(self):
  rows=[{'accepted':True,'usage':None,'agent_wall_s':1,'command_tool_items':1,'checks':[],'grading':{'accepted':True}}, {'accepted':False,'usage':None,'agent_wall_s':2,'command_tool_items':2,'checks':[],'grading':{'accepted':False}}]
  value=m.aggregate(rows);self.assertEqual(value['attempted'],2);self.assertEqual(value['accepted'],1)
 def test_result_integrity_rejects_duplicate_assignments(self):
  schedule=[{'index':0},{'index':1}]
  with self.assertRaises(ValueError):m.verify_assignments([{'assignment':{'index':0}},{'assignment':{'index':0}}],schedule)
if __name__=='__main__':unittest.main()
