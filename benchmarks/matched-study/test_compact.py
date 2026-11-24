import copy
import importlib.util
from pathlib import Path
import re
import unittest

spec=importlib.util.spec_from_file_location('compact',Path(__file__).with_name('compact.py'))
s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)


class CompactTests(unittest.TestCase):
    def test_three_arms_only_change_guide_and_topic_card(self):
        for topic in s.TOPICS:
            a=s.prompt(dict(family=topic,arm='legacy'))
            b=s.prompt(dict(family=topic,arm='clean'))
            c=s.prompt(dict(family=topic,arm='targeted'))
            self.assertEqual(a.replace(s.LEGACY,s.CLEAN,1),b)
            self.assertEqual(b+'\n'+s.card(topic),c)
            self.assertIn(s.LANGUAGE_BODY,s.CLEAN)
            self.assertLess(len(s.CLEAN.encode())+len(s.card(topic).encode()),len(s.LEGACY.encode()))
            for forbidden in ('check.py','editable','identities sync'):
                self.assertNotIn(forbidden,s.CLEAN)

    def test_cards_are_short_sourced_and_cover_all_buffer_operations(self):
        for topic in s.TOPICS:
            self.assertLessEqual(len(s.card(topic).encode()),1800)
            for ref in s.PACKET['cards'][topic]['sources']:
                self.assertEqual(s.sha(s.ROOT/ref['path']),ref['sha256'])
                self.assertIn(ref['anchor'],(s.ROOT/ref['path']).read_text())
        case=s.corpus('ranges');used=set(re.findall(r'\b(?:Bytes|Text)::\w+',case['source']+'\n'+''.join(c['source'] for c in case['admissions'])))
        self.assertLessEqual(used,set(s.PACKET['buffer_signatures']))
        for name in used:self.assertIn(name,s.card('ranges'))

    def test_scoring_remains_strict_and_separates_shape(self):
        for topic in s.TOPICS:
            answer=s.correct(topic);self.assertTrue(s.grade(topic,answer)['accepted'])
            wrong=copy.deepcopy(answer);wrong['predictions'][0]+=1
            self.assertFalse(s.grade(topic,wrong)['accepted']);self.assertTrue(s.grade(topic,wrong)['format_valid'])
            wrong=copy.deepcopy(answer);wrong['extra']='metadata'
            self.assertFalse(s.grade(topic,wrong)['format_valid']);self.assertEqual(s.grade(topic,wrong)['passed'],10)

    def test_fresh_questions_do_not_reuse_old_static_modules(self):
        for topic in s.TOPICS:
            case=s.corpus(topic)
            old={a['source'] for form in (0,1) for a in s.L.corpus.micro(topic,form)['admissions']}
            self.assertTrue(all(a['source'] not in old for a in case['admissions']))
            self.assertNotEqual(case['source'],s.L.corpus.micro(topic,0)['source'])
            self.assertNotEqual(case['source'],s.L.corpus.micro(topic,1)['source'])

    def test_small_balanced_schedule_and_common_runner(self):
        rows=s.schedule();self.assertEqual(len(rows),24)
        self.assertTrue(all(r['reasoning']=='high' for r in rows[:12]))
        for effort in ('high','low'):
            for topic in s.TOPICS:
                self.assertEqual({r['arm'] for r in rows if r['family']==topic and r['reasoning']==effort},set(s.ARMS))
        cmd=s.L.C.command(Path('/tmp/study'),'low',Path('/tmp/last'))
        self.assertIn('read-only',cmd);self.assertIn('shell_tool',cmd)


if __name__=='__main__':unittest.main()
