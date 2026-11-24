import importlib.util
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('development',Path(__file__).with_name('development.py'))
s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)

class DevelopmentTests(unittest.TestCase):
    def test_fair_schedule_and_limits(self):
        rows=s.schedule();self.assertEqual(len(rows),12)
        for task in s.C.TASKS:
            for repeat in range(2):
                pair=[r for r in rows if r['family']==task and r['repeat']==repeat]
                self.assertEqual({r['arm'] for r in pair},{'control','skill'})
                self.assertTrue(all(r['reasoning']=='high' for r in pair))
        self.assertEqual(s.B.DEADLINE,120);self.assertEqual(s.B.TOOL_LIMIT,16);self.assertEqual(s.B.CHECK_LIMIT,6)
    def test_only_skill_availability_differs(self):
        for task in s.C.TASKS:
            a=s.fixtures(dict(family=task,arm='control'));b=s.fixtures(dict(family=task,arm='skill'))
            self.assertEqual(set(b)-set(a),{'.agents/skills/lobo-development/SKILL.md'})
            self.assertEqual(a,{k:v for k,v in b.items() if k in a})
            self.assertNotIn('SKILL.md',' '.join(a))
    def test_oracles_boundaries(self):
        self.assertEqual(s.C.oracle('splice',dict(text='aéZ',start=1,count=1,replacement='🐺')),dict(case='Ok',payload=dict(text='a🐺Z',removed='é',scalar_delta=0,byte_delta=2)))
        self.assertEqual(s.C.oracle('splice',dict(text='',start=-1,count=-1,replacement='')),dict(case='Err',payload=1))
        self.assertEqual(s.C.oracle('batch',dict(samples=[[2,3],[1,4]])),dict(case='Ok',payload=dict(values=[6,4],total=10,maximum=6)))
        self.assertEqual(s.C.oracle('batch',dict(samples=[[-1,0]])),dict(case='Err',payload=11))
        self.assertEqual(s.C.oracle('dispatch',dict(entries=[[1,8,True],[2,4,True],[1,9,False]])),dict(kept=[dict(key=2,value=4,active=True)],dropped=2))
    def test_native_list_record_wire_contract(self):
        self.assertEqual(s.C.arguments('batch',dict(samples=[[2,3],[1,4]])),['List:Count:2,I64:2,I64:3,I64:1,I64:4'])
    def test_fixture_integrity_and_stable_ids(self):
        row=dict(family='splice',arm='control');files=s.fixtures(row)
        with tempfile.TemporaryDirectory() as t:
            p=Path(t);s.B.save_sources(p,files)
            self.assertEqual(s.integrity(p,files),[])
            (p/'logic.lobo').write_text(s.C.logic('splice','reference'))
            self.assertEqual(s.integrity(p,files),[])
            (p/'model.lobo').write_text('changed')
            self.assertIn('modified:model.lobo',s.integrity(p,files))
    def test_user_home_has_no_installed_lobo_skill(self):
        with tempfile.TemporaryDirectory() as t:
            home=s.isolated_home(Path(t))
            self.assertTrue((home/'auth.json').is_file())
            self.assertEqual(list((home/'skills').iterdir()),[])
            self.assertEqual((home/'auth.json').stat().st_mode&0o777,0o600)
    def test_unknown_usage_is_unknown(self):
        self.assertIsNone(s.B.usage([]))

    def test_runner_prompt_delta_is_exact(self):
        self.assertEqual(s.B.run_trial.__code__.co_consts.count(s._PROMPT_COMMON),1)
        with tempfile.TemporaryDirectory() as t:
            args=s.command(Path(t),dict(phase='construction'),Path(t)/'last.txt')
            self.assertEqual(args[args.index('-m')+1],'gpt-5.6-luna')
            self.assertIn('model_reasoning_effort="high"',args)
