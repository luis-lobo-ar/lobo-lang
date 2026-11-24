import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('sensitivity',Path(__file__).with_name('context_sensitivity.py'))
s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)


class SensitivityTests(unittest.TestCase):
    def test_removes_top_level_metadata_without_changing_input(self):
        value={'data':'0007','limit':1,'frames':[''],'error':0}
        self.assertEqual(s.project('frames',value),{'data':'0007','limit':1})
        self.assertIn('frames',value)

    def test_does_not_invent_missing_inputs_or_relax_nested_shapes(self):
        self.assertEqual(s.project('frames',{'frames':['']}),{})
        self.assertFalse(s.c.BASE.valid_input('frames',s.project('frames',{'data':'0007'})))
        value={'budget':1,'jobs':[{'id':0,'cost':1,'deps':[],'note':'extra'}]}
        self.assertFalse(s.c.BASE.valid_input('scheduler',s.project('scheduler',value)))

    def test_projection_cannot_fix_wrong_semantics(self):
        value={'initial':10,'events':[[0,3],[0,98],[1,-1]],'note':'extra'}
        self.assertNotEqual(s.c.oracle('ledger',s.project('ledger',value))['balance'],10)


if __name__=='__main__':unittest.main()
