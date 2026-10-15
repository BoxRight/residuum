"""Pure helpers of scripts/test_safe.py used by the supervised mode, imported
without running the runner. Run individually through ``scripts/test_safe.py
--python-test``."""
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def runner():
    spec = importlib.util.spec_from_file_location("test_safe", ROOT / "scripts" / "test_safe.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class NormalizationTests(unittest.TestCase):
    def test_relative_worker_paths_keep_the_original_base(self):
        module = runner()
        base = ROOT / "examples" / "experiments"
        document = {"worker": {"imports": ["legal_geometry/riggs_problem_manifest.res", "/abs/other.res"],
                               "workdir": "../../target/exploration/x", "episode_id": "e"},
                    "controller": {}}
        worker, changes = module.normalize_worker_manifest(document, base)
        self.assertEqual(worker["imports"][0], str((base / "legal_geometry/riggs_problem_manifest.res").resolve()))
        self.assertEqual(worker["imports"][1], "/abs/other.res")
        self.assertEqual(worker["workdir"], str((base / "../../target/exploration/x").resolve()))
        self.assertEqual([c["field"] for c in changes], ["imports[0]", "workdir"])
        self.assertEqual(document["worker"]["imports"][0], "legal_geometry/riggs_problem_manifest.res",
                         "the original document is not modified")
        self.assertTrue(all(Path(c["resolved"]).is_absolute() for c in changes))


if __name__ == "__main__":
    unittest.main()
