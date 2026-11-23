import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ReleaseInputsTests(unittest.TestCase):
    def test_source_snapshot_includes_the_library_needed_by_compiler_tests(self):
        spec = importlib.util.spec_from_file_location("release_builder", ROOT / "scripts/release.py")
        release = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(release)
        inputs = release.inputs()
        self.assertTrue("libraries/http/http.lobo" in inputs, "release snapshot omitted required HTTP library source")


if __name__ == "__main__":
    unittest.main()
