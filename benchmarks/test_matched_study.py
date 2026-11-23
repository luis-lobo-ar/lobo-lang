import importlib.util
from pathlib import Path
import unittest
SPEC=importlib.util.spec_from_file_location('matched',Path(__file__).with_name('matched-study')/'study.py')
m=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(m)
class MatchedTests(unittest.TestCase):
 def test_schedule_is_balanced_and_includes_every_phase(self):
  rows=m.schedule();self.assertEqual(len(rows),54)
  for phase in m.PHASES:
   for language in m.LANGUAGES:self.assertEqual(sum(r['phase']==phase and r['language']==language for r in rows),6)
  for language in m.LANGUAGES:
   self.assertEqual([sum(rows[i+p]['language']==language for i in range(0,54,3)) for p in range(3)],[6,6,6])
 def test_ledger_freezing_undo_and_peak_are_independent(self):
  events=[(0,5),(1,3),(2,0),(1,1),(0,2),(4,0),(3,0),(4,0),(4,0)]
  self.assertEqual(m.oracle('ledger',{'events':events,'initial':0},True),{'balance':5,'rejected':2,'peak':5,'audit':[5,2,2,2,4,4,4,2,5]})
 def test_scheduler_selects_fitting_ready_jobs_and_detects_cycles(self):
  jobs=[{'id':0,'cost':4,'deps':[1]},{'id':1,'cost':2,'deps':[]},{'id':2,'cost':1,'deps':[2]}]
  self.assertEqual(m.oracle('scheduler',{'jobs':jobs,'budget':3}),{'order':[1],'spent':2,'blocked':2})
  priority=[{'id':0,'cost':1,'deps':[],'priority':1},{'id':1,'cost':1,'deps':[],'priority':9}]
  self.assertEqual(m.oracle('scheduler',{'jobs':priority,'budget':1},True)['order'],[1])
 def test_frames_error_priority_and_extension(self):
  self.assertEqual(m.oracle('frames',{'data':'02010102ff','limit':2}),{'frames':['0101'],'consumed':4,'error':3})
  self.assertEqual(m.oracle('frames',{'data':'fffe0200010102','limit':2},True),{'frames':['0101'],'consumed':7,'error':0})
  self.assertEqual(m.oracle('frames',{'data':'fe01','limit':2},True),{'frames':[],'consumed':0,'error':1})
 def test_native_list_decoder_uses_the_actual_wire_shape(self):
  self.assertEqual(m.untyped({"type":"List","value":{"element":"I64","elements":[{"type":"I64","value":"7"}]}}),[7])
 def test_usage_missing_and_cached_counts_not_added(self):
  self.assertIsNone(m.usage([]))
  value={'input_tokens':100,'cached_input_tokens':80,'output_tokens':5};self.assertEqual(m.usage([{'type':'turn.completed','usage':value}]),value)
 def test_comprehension_rejects_boolean_predictions_and_wrong_impact(self):
  for family in m.FAMILIES:
   answer=m.correct_comprehension(family)
   self.assertTrue(m.grade_comprehension(family,answer)['accepted'])
   answer['impacted']=[]
   self.assertFalse(m.grade_comprehension(family,answer)['accepted'])
  self.assertFalse(m.equal(1,True))
 def test_mutants_are_detectable_and_fixture_changes_rejected(self):
  import tempfile
  for family in m.FAMILIES:
   for language in m.LANGUAGES:
    self.assertNotEqual(m.mutant(family,language),m.reference(family,language))
  with tempfile.TemporaryDirectory() as tmp:
   directory=Path(tmp);files=m.sources('ledger','python');m.save_sources(directory,files)
   self.assertEqual(m.integrity(directory,files,'python'),[])
   (directory/'model.py').write_text('broken')
   self.assertIn('modified:model.py',m.integrity(directory,files,'python'))
if __name__=='__main__':unittest.main()
