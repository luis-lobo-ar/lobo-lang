import hashlib
import importlib.util
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('reassessment', Path(__file__).with_name('reassess.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class ReassessmentTests(unittest.TestCase):
    def test_metadata_can_eliminate_apparent_source_savings(self):
        result=module.compare_representation(b'0123456789',b'12345',b'abcde')
        self.assertEqual(result['source_reduction_percent'],50)
        self.assertEqual(result['combined_reduction_percent'],0)
        self.assertIsNone(result['billed_token_effect'])

    def test_missing_usage_is_retained(self):
        result=module.summarize_trials([{'accepted':True,'usage':{'input_tokens':10}}, {'accepted':False,'usage':None}, {'accepted':False,'usage':{}}])
        self.assertEqual(result,{'scheduled':3,'accepted':1,'usage_known':1,'usage_unknown':2})

    def test_a_modified_snapshot_cannot_pass_hash_verification(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);(root/'sample').write_bytes(b'old')
            files=[{'snapshot':'sample','sha256':hashlib.sha256(b'old').hexdigest()}]
            module.verify_files(root,files)
            (root/'sample').write_bytes(b'new')
            with self.assertRaises(ValueError):module.verify_files(root,files)

    def test_repeated_knowledge_records_cannot_inflate_size(self):
        files=[{'version':'old','original_name':'guide','bytes':8}]*2
        with self.assertRaises(ValueError):module.knowledge_sizes(files)

if __name__=='__main__':unittest.main()
