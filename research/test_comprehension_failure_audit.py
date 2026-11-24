import importlib.util
from pathlib import Path
import unittest
SPEC=importlib.util.spec_from_file_location('audit',Path(__file__).with_name('comprehension_failure_audit.py'))
m=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(m)
class ComprehensionAuditTests(unittest.TestCase):
 def test_reports_duplicate_guides_without_claiming_token_savings(self):
  value=m.prompt_metrics('TASK! GUIDE! FILE check.py\nTASK! GUIDE!','TASK!','GUIDE!')
  self.assertEqual(value['task_occurrences'],2);self.assertEqual(value['guide_occurrences'],2)
  self.assertEqual(value['duplicate_task_guide_bytes'],len('TASK!GUIDE!'))
  self.assertTrue(value['contains_check_client']);self.assertIsNone(value['token_saving'])
 def test_shared_failure_requires_same_family_and_item(self):
  peers=[{'language':'rust','family':'frames','item':'prediction_1'},{'language':'python','family':'ledger','item':'witness'}]
  self.assertEqual(m.counterparts({'language':'lobo','family':'frames','item':'prediction_1'},peers),['rust'])
  self.assertEqual(m.counterparts({'language':'lobo','family':'scheduler','item':'witness'},peers),[])
 def test_failed_items_preserve_each_error_and_each_attempt(self):
  rows=[{'configuration':'high','assignment':{'index':15,'family':'frames','language':'lobo'},'checks':{'prediction_1':False,'impacted':False,'witness':True}}, {'configuration':'low','assignment':{'index':15,'family':'frames','language':'lobo'},'checks':{'prediction_1':False}}]
  items=m.error_items(rows);self.assertEqual(len(items),3)
  self.assertEqual([i['configuration'] for i in items],['high','high','low'])
 def test_configuration_matched_overlap_does_not_pool_models(self):
  item={'configuration':'luna-high','language':'lobo','family':'ledger','item':'prediction_5'}
  peers=[{'configuration':'luna-low','language':'python','family':'ledger','item':'prediction_5'}]
  self.assertEqual(m.counterparts(item,peers),['python'])
  self.assertEqual(m.counterparts(item,peers,same_configuration=True),[])
if __name__=='__main__':unittest.main()
