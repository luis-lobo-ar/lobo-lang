import importlib.util,json,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('v3',Path(__file__).with_name('v3.py'));s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
class V3Tests(unittest.TestCase):
 def test_isolated_factors_and_full_candidate(self):
  baseline=s.skill('v2');self.assertEqual(s.skill('v3'),baseline+''.join(s.PARTS.values()))
  for factor in s.PARTS:self.assertEqual(s.skill(factor),baseline+s.PARTS[factor])
 def test_matched_schedule_and_budgets(self):
  rows=s.schedule();self.assertEqual(len(rows),20)
  for task in s.C.TASKS:
   for repeat in range(2):self.assertEqual({r['arm'] for r in rows if r['family']==task and r['repeat']==repeat},set(s.ARMS))
  self.assertEqual((s.B.DEADLINE,s.B.TOOL_LIMIT,s.B.CHECK_LIMIT),(120,16,6));self.assertEqual(len({s.prompt(arm) for arm in s.ARMS}),1)
 def test_only_skill_bytes_differ(self):
  for task in s.C.TASKS:
   original=s.fixtures(dict(family=task,arm='v2'));base={k:v for k,v in original.items() if k!=s.SKILL}
   for arm in s.ARMS:self.assertEqual(base,{k:v for k,v in s.fixtures(dict(family=task,arm=arm)).items() if k!=s.SKILL})
 def test_http_oracle(self):
  f=lambda path:s.C.http_oracle(dict(method='GET',path=path))
  self.assertEqual(f('/api/events?limit=2&after=0')['body']['next'],3)
  self.assertEqual(f('/api/events?after=3&limit=2')['body']['next'],12)
  self.assertIsNone(f('/api/events?after=12')['body']['next'])
  self.assertEqual(f('/api/events?limit=2&limit=2')['status'],400)
  self.assertEqual(f('/api/events?after=00')['status'],400)
 def test_sql_oracle(self):
  c=dict(inventory=[[1,10],[2,5]],operations=[[1,2,1,3,'a'],[1,5,0,0,'a'],[1,2,2,1,'a']])
  values=s.C.sql_oracle(c);self.assertEqual(values[0]['inventory'],[[1,5],[2,5]]);self.assertEqual(values[1]['reply']['payload']['reason'],'replay');self.assertEqual(values[2]['reply']['payload']['reason'],'conflict')
  error=s.C.sql_oracle(dict(inventory=[[1,10],[2,5]],operations=[[1,2,2,1,'reject']]))[0]
  self.assertEqual(error['inventory'],[[1,10],[2,5]]);self.assertEqual(error['reply']['case'],'Err');self.assertTrue(error['clean'])
 def test_fixture_integrity_covers_both_modules(self):
  for task in s.C.TASKS:
   files=s.C.files(task)
   self.assertTrue(all(n in files for n in s.C.sources(task)));self.assertLessEqual(len(s.C.sources(task)),16)
   with tempfile.TemporaryDirectory() as tmp:
    folder=Path(tmp);s.B.save_sources(folder,files);self.assertEqual(s.H.integrity(folder,files),[])
    (folder/'app.lobo').write_text('changed');self.assertIn('modified:app.lobo',s.H.integrity(folder,files))
 def test_cart_canonicalization_is_order_and_representation_independent(self):
  values=s.C.sql_oracle(dict(inventory=[[1,10],[2,10]],operations=[[2,1,1,2,'same'],[1,2,2,1,'same'],[1,3,1,2,'dup'],[1,5,-9223372036854775808,0,'dup']]))
  self.assertEqual(values[0]['requests'],[['same',1,2,2,1]])
  self.assertEqual([v['reply']['payload']['reason'] for v in values],['reserved','replay','reserved','replay'])
 def test_feed_exhaustion_overflow_and_case_counts(self):
  maximum=s.C.http_oracle(dict(method='GET',path='/api/events?after=9223372036854775807'))
  self.assertEqual(maximum['body'],dict(items=[],count=0,next=None))
  self.assertEqual(s.C.http_oracle(dict(method='GET',path='/api/events?after=9223372036854775808'))['status'],400)
  for task,count in [('feed',40),('reservation',30)]:
   self.assertEqual(len(s.C.cases(task)),5);self.assertEqual(len(s.C.cases(task,True)),count)
