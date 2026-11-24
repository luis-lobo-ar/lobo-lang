import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('learning_report',Path(__file__).with_name('learning_report.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)


class ReportingTests(unittest.TestCase):
    def test_shape_and_task_success_are_separate(self):
        rows=[dict(accepted=False,status='completed',agent_wall_s=1,usage=None,grading=dict(format_valid=False,passed=10,total=10,checks={'predictions_0':True})),dict(accepted=False,status='completed',agent_wall_s=2,usage=dict(input_tokens=100,output_tokens=10,cached_input_tokens=80,reasoning_output_tokens=5),grading=dict(format_valid=True,passed=9,total=10,checks={'predictions_0':False}))]
        g=r.aggregate(rows)
        self.assertEqual(g['format_valid'],1);self.assertEqual(g['all_task_checks_correct'],1)
        self.assertEqual(g['accepted'],0);self.assertEqual(g['usage']['unknown'],1)
        self.assertEqual(g['usage']['reported_input_plus_output_tokens'],110)

    def test_no_cross_stage_pairs(self):
        def row(stage,arm,accepted):
            return dict(assignment=dict(stage=stage,reasoning='low',family='ranges',language='lobo',arm=arm,index=0),accepted=accepted,agent_wall_s=1,usage=None,grading=dict(passed=10 if accepted else 9,total=10,format_valid=True,checks={}))
        rows=[row('reference','base',False),row('reference','flat',True),row('graph','flat',True),row('graph','graph',False)]
        p=r.contrast(rows,'reference')
        self.assertEqual(p['pairs'],1);self.assertEqual(p['fail_to_pass'],1);self.assertEqual(p['pass_to_fail'],0)


if __name__=='__main__':unittest.main()
