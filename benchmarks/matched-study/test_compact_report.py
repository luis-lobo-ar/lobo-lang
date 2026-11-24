import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('r',Path(__file__).with_name('compact_report.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)


class ReportTests(unittest.TestCase):
    def test_errors_and_unknown_usage_remain_in_denominator(self):
        rows=[dict(accepted=False,status='completed',agent_wall_s=2,usage=None,grading=dict(format_valid=True,passed=9,total=10,checks={'predictions_0':True,'admissions_0':False})),dict(accepted=True,status='completed',agent_wall_s=3,usage=dict(input_tokens=100,output_tokens=10,cached_input_tokens=80,reasoning_output_tokens=4),grading=dict(format_valid=True,passed=10,total=10,checks={'predictions_0':True,'admissions_0':True}))]
        g=r.aggregate(rows)
        self.assertEqual(g['attempted'],2);self.assertEqual(g['accepted'],1)
        self.assertEqual(g['usage']['reported_input_plus_output_tokens'],110)
        self.assertEqual(g['usage']['unknown'],1)
        self.assertEqual(g['admissions_passed'],1)


if __name__=='__main__':unittest.main()
