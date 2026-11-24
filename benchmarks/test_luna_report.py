import importlib.util
from pathlib import Path
import unittest
SPEC=importlib.util.spec_from_file_location('luna_report',Path(__file__).with_name('matched-study')/'luna_report.py')
m=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(m)
class LunaReportTests(unittest.TestCase):
 def test_failure_categories_do_not_turn_ungraded_scope_into_functional_failure(self):
  row={'status':'completed','accepted':False,'grading':{'issues':['unexpected:test.py'],'build':None,'cases':[]}}
  self.assertEqual(m.failure_categories(row),['scope_rejected'])
  row={'status':'wall_limit','accepted':False,'grading':{'issues':[],'build':{'ok':True},'cases':[{'ok':True}]}}
  self.assertEqual(m.failure_categories(row),['wall_limit'])
 def test_pair_counts_keep_failures_and_unknown_usage(self):
  assignment={'index':0,'family':'ledger','phase':'construction','language':'lobo','repeat':0}
  left={'assignment':assignment,'accepted':False,'status':'wall_limit','usage':None,'agent_wall_s':120}
  right={'assignment':assignment,'accepted':True,'status':'completed','usage':{'input_tokens':10,'output_tokens':5},'agent_wall_s':10}
  pair=m.compare_pairs([left],[right])[0]
  self.assertFalse(pair['left_accepted']);self.assertTrue(pair['right_accepted']);self.assertIsNone(pair['left_over_right_tokens']);self.assertEqual(pair['left_over_right_wall'],12)
if __name__=='__main__':unittest.main()
