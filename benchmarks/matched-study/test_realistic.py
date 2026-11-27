import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('realistic',Path(__file__).with_name('realistic.py'));s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
class RealisticTests(unittest.TestCase):
 def test_budget_and_matched_schedule(self):
  rows=s.schedule();self.assertEqual(len(rows),12)
  for task in s.C.TASKS:
   for repeat in range(3):self.assertEqual({r['arm'] for r in rows if r['family']==task and r['repeat']==repeat},{'control','v2'})
  self.assertEqual((s.B.DEADLINE,s.B.TOOL_LIMIT,s.B.CHECK_LIMIT),(120,16,6))
 def test_only_v2_entrypoint_differs(self):
  for task in s.C.TASKS:
   a=s.fixtures(dict(family=task,arm='control'));b=s.fixtures(dict(family=task,arm='v2'))
   self.assertEqual(set(b)-set(a),{s.SKILL});self.assertEqual(a,{k:v for k,v in b.items() if k in a})
 def test_route_oracle(self):
  self.assertEqual(s.C.http_oracle(dict(method='GET',path='/api/products/02'))['status'],404)
  self.assertEqual(s.C.http_oracle(dict(method='GET',path='/api/products/11'))['status'],404)
  self.assertEqual(s.C.http_oracle(dict(method='POST',path='/health'))['status'],405)
  self.assertEqual(s.C.http_oracle(dict(method='GET',path='/api/products/available'))['body']['total'],1)
 def test_transaction_oracle_idempotency_conflict_and_error(self):
  x=s.C.sql_oracle(dict(accounts=[[1,10],[2,0]],operations=[[1,2,3,'x'],[1,2,3,'x'],[1,2,4,'x']]))
  self.assertEqual(x[-1]['accounts'],[[1,7],[2,3]]);self.assertEqual(x[1]['reply']['payload']['reason'],'replay');self.assertEqual(x[2]['reply']['payload']['reason'],'conflict')
  y=s.C.sql_oracle(dict(accounts=[[1,10],[2,999999]],operations=[[1,2,2,'overflow']]))
  self.assertEqual(y[0]['reply']['case'],'Err');self.assertEqual(y[0]['accounts'],[[1,10],[2,999999]])
 def test_bundles_and_multi_file_integrity(self):
  self.assertEqual(set(s.EDITABLE),{'logic.lobo','logic.lobo.ids.json','policy.lobo','policy.lobo.ids.json'})
  for task in s.C.TASKS:
   files=s.C.files(task);sources=s.C.sources(task)
   self.assertTrue(all(p in files for p in sources));self.assertLessEqual(len(sources),16)
 def test_signature_and_unknown_usage_helpers(self):
  self.assertIsNone(s.B.usage([]))
  self.assertEqual(s.prompt('v2'),s.prompt('control')+'Use $lobo-development-v2 at '+s.SKILL+'.\n')
 def test_fixed_files_and_both_identity_maps_are_enforced(self):
  import tempfile,json
  with tempfile.TemporaryDirectory() as p:
   folder=Path(p);files=s.C.files('transfer');s.B.save_sources(folder,files)
   self.assertEqual(s.integrity(folder,files),[])
   (folder/'policy.lobo').write_text(files['policy.lobo']+'\n')
   self.assertEqual(s.integrity(folder,files),[])
   metadata=json.loads(files['policy.lobo.ids.json']);metadata['identities']['valid']='wrong';(folder/'policy.lobo.ids.json').write_text(json.dumps(metadata))
   self.assertTrue(any('changed_identity:policy' in x for x in s.integrity(folder,files)))
   (folder/'app.lobo').write_text('changed');self.assertIn('modified:app.lobo',s.integrity(folder,files))
 def test_unicode_length_and_invalid_before_replay(self):
  good=s.C.sql_oracle(dict(accounts=[[1,10],[2,0]],operations=[[1,2,1,'夜'*24],[1,2,1,'夜'*25]]))
  self.assertEqual(good[0]['reply']['payload']['reason'],'applied');self.assertEqual(good[1]['reply']['payload']['reason'],'invalid')
  self.assertEqual(len(s.C.cases('catalog',True)),30);self.assertEqual(len(s.C.cases('transfer',True)),20)
