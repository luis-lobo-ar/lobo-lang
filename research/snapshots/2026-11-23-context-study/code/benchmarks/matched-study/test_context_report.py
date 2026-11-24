import importlib.util
import copy
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('context_report',Path(__file__).with_name('context_report.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)


def row(arm,accepted=True,usage=None):
    return dict(assignment=dict(reasoning='high',family='ledger',language='lobo',arm=arm,index=0),accepted=accepted,status='completed',grading=dict(passed=11 if accepted else 10,total=11),usage=usage,agent_wall_s=2)


class ReportingTests(unittest.TestCase):
    def test_unknown_usage_is_unknown(self):
        a=r.account([row('original')])
        self.assertEqual(a['known'],0);self.assertEqual(a['unknown'],1)
        self.assertIsNone(a['reported_input_plus_output_tokens'])

    def test_cache_and_reasoning_are_subsets(self):
        a=r.account([row('original',usage=dict(input_tokens=100,output_tokens=10,cached_input_tokens=80,reasoning_output_tokens=5))])
        self.assertEqual(a['reported_input_plus_output_tokens'],110)
        self.assertEqual(a['cached_input_tokens'],80)
        self.assertEqual(a['reasoning_output_tokens'],5)
        self.assertIsNone(a['billing'])

    def test_matched_discordances_and_missing_cost_pairs(self):
        rows=[row('original',True,dict(input_tokens=100,output_tokens=10)),row('dedup',False)]
        result=r.paired(rows,'original','dedup')
        self.assertEqual(result['pairs'],1)
        self.assertEqual(result['pass_to_fail'],1)
        self.assertEqual(result['fail_to_pass'],0)
        self.assertEqual(result['token_ratio']['n'],0)
        self.assertIsNone(result['token_ratio']['median'])

    def test_failed_attempts_remain_in_cost_denominator(self):
        rows=[row('original',True,dict(input_tokens=100,output_tokens=10)),row('dedup',False,dict(input_tokens=70,output_tokens=20))]
        a=r.account(rows)
        self.assertEqual(a['known'],2)
        self.assertEqual(a['reported_input_plus_output_tokens'],200)

    def test_item_diagnostics_retain_wrong_answer_and_oracle(self):
        answer=copy.deepcopy(r.c.correct('ledger'))
        expected=answer['predictions'][0]['balance']
        answer['predictions'][0]['balance']+=1
        result=row('dedup',False);result['grading']=r.c.grade('ledger',answer)
        values=r.explain(result,answer)
        self.assertEqual(len(values),1)
        self.assertEqual(values[0]['item'],'prediction_0')
        self.assertEqual(values[0]['actual']['balance'],expected+1)
        self.assertEqual(values[0]['expected']['balance'],expected)


if __name__=='__main__':unittest.main()
