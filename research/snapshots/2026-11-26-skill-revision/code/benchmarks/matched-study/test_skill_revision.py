import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('revision',Path(__file__).with_name('skill_revision.py'))
s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
class RevisionTests(unittest.TestCase):
 def test_matched_three_arm_schedule(self):
  rows=s.schedule();self.assertEqual(len(rows),12)
  for task in s.C.TASKS:
   for repeat in range(2):self.assertEqual({r['arm'] for r in rows if r['family']==task and r['repeat']==repeat},set(s.ARMS))
  self.assertEqual(s.B.DEADLINE,120);self.assertEqual(s.B.TOOL_LIMIT,16);self.assertEqual(s.B.CHECK_LIMIT,6)
 def test_shared_knowledge_and_exact_skills(self):
  for task in s.C.TASKS:
   a=s.fixtures(dict(family=task,arm='control'))
   for arm in ('original','revised'):
    b=s.fixtures(dict(family=task,arm=arm));path=s.SKILLS[arm]
    self.assertEqual(set(b)-set(a),{path});self.assertEqual(a,{k:v for k,v in b.items() if k in a});self.assertEqual(b[path],(s.ROOT/path).read_text())
 def test_oracle_boundary_cases(self):
  self.assertEqual(s.C.oracle('locate',dict(text='éababa',needle='aba',start=2)),dict(case='Found',payload=dict(index=3,prefix='éab',suffix='',match_bytes=3)))
  self.assertEqual(s.C.oracle('locate',dict(text='',needle='',start=0)),dict(case='Found',payload=dict(index=0,prefix='',suffix='',match_bytes=0)))
  self.assertEqual(s.C.oracle('coalesce',dict(spans=[[0,2],[3,3],[4,1],[7,9],[8,10]])),dict(spans=[dict(start=0,end=3),dict(start=7,end=10)],points=8,rejected=1))
 def test_native_wire_and_cases(self):
  self.assertEqual(s.C.arguments('coalesce',dict(spans=[[0,3],[5,4]])),['List:Count:2,I64:0,I64:3,I64:5,I64:4'])
  for task in s.C.TASKS:self.assertEqual(len(s.C.cases(task)),5);self.assertEqual(len(s.C.cases(task,True)),70)
 def test_prompt_delta_and_shorter_revision(self):
  base=s.prompt('control')
  for arm in ('original','revised'):self.assertTrue(s.prompt(arm).startswith(base))
  self.assertLess((s.ROOT/s.SKILLS['revised']).stat().st_size,(s.ROOT/s.SKILLS['original']).stat().st_size)
 def test_code_clone_does_not_change_old_runner(self):
  self.assertEqual(s.B.run_trial.__code__.co_consts.count(s.D._PROMPT_COMMON),1)

 def test_payload_free_case_normalization(self):
  for payload in ({},dict(payload=None)):
   self.assertEqual(s.normalize(dict(type='Variant',value=dict(id='x',case='Missing',**payload))),dict(case='Missing',payload=None))
