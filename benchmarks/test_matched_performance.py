import importlib.util
from pathlib import Path
import unittest
SPEC=importlib.util.spec_from_file_location('performance',Path(__file__).with_name('matched-study')/'performance.py')
m=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(m)
class PerformanceTests(unittest.TestCase):
 def test_full_payload_is_consumed_in_base_and_extended_inputs(self):
  for changed in [False,True]:
   args=m.inputs('frames',changed);value=m.study.oracle('frames',args,changed)
   self.assertEqual(value['consumed'],len(bytes.fromhex(args['data'])))
   self.assertEqual(value['error'],0);self.assertEqual(len(value['frames']),32)
 def test_zero_batch_does_no_work_and_single_batch_uses_result(self):
  for family in m.study.FAMILIES:
   for changed in [False,True]:
    args=m.inputs(family,changed)
    self.assertEqual(m.expected(family,changed,args,0),0)
    self.assertEqual(m.expected(family,changed,args,1),m.checksum(family,m.study.oracle(family,args,changed)))
    self.assertNotEqual(m.expected(family,changed,args,100),0)
 def test_distribution_keeps_range_and_count(self):
  self.assertEqual(m.distribution([3,1,5,2,4]),{'n':5,'median_ms':3,'min_ms':1,'max_ms':5})
if __name__=='__main__':unittest.main()
