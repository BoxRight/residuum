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


class InteractivePolicyArgsTests(unittest.TestCase):
    def test_policy_follows_the_manifest_provider(self):
        args = runner().interactive_policy_args

        def doc(provider):
            return {"worker": {}, "controller": {"model": {"provider": provider}}}
        self.assertEqual(args(doc("simulated"), "branching"), ["--policy", "branching"])
        self.assertEqual(args(doc("anthropic"), None), [])
        with self.assertRaisesRegex(RuntimeError, "needs --interactive-policy"):
            args(doc("simulated"), None)
        with self.assertRaisesRegex(RuntimeError, "only for the simulated provider"):
            args(doc("anthropic"), "branching")
        with self.assertRaisesRegex(RuntimeError, "names no controller.model.provider"):
            args({"worker": {}}, None)


class InteractiveDeadlineTests(unittest.TestCase):
    def test_remote_only_deadline_within_bounds(self):
        deadline = runner().interactive_deadline

        def doc(provider):
            return {"controller": {"model": {"provider": provider}}}
        self.assertEqual(deadline(doc("simulated"), None), 15)
        self.assertEqual(deadline(doc("anthropic"), 1500), 1500)
        self.assertEqual(deadline(doc("anthropic"), 1), 1)
        for provider, value, pattern in [("simulated", 600, "only for a real provider"),
                                         ("anthropic", None, "needs an explicit"),
                                         ("anthropic", 1501, "within 1..1500"),
                                         ("anthropic", 0, "within 1..1500")]:
            with self.assertRaisesRegex(RuntimeError, pattern):
                deadline(doc(provider), value)


class EndClassificationTests(unittest.TestCase):
    def test_normal_end_whatever_the_order_and_technical_ends(self):
        classify = runner().classify_supervised_end
        self.assertIsNone(classify(["worker", "controller"], {"worker": 0, "controller": 0}, False, None))
        self.assertIsNone(classify(["controller", "worker"], {"worker": 0, "controller": 0}, False, None))
        self.assertEqual(classify(["worker", "controller"], {"worker": 0, "controller": 4}, False, None), "worker_lost")
        self.assertEqual(classify(["worker"], {"worker": 0, "controller": None}, True, None), "worker_lost")
        self.assertEqual(classify(["controller"], {"worker": None, "controller": 0}, True, None), "worker_did_not_exit")
        self.assertEqual(classify(["controller", "worker"], {"worker": 5, "controller": 0}, False, None), "worker_nonzero_exit")
        self.assertEqual(classify(["controller", "worker"], {"worker": 0, "controller": 3}, False, None), "controller_nonzero_exit")
        self.assertEqual(classify(["worker", "controller"], {"worker": 0, "controller": 6}, False, None), "worker_lost")
        self.assertEqual(classify([], {"worker": None, "controller": None}, False, "timeout"), "timeout")
        self.assertEqual(classify(["worker", "controller"], {"worker": 0, "controller": 0}, False, "near_memory_limit"),
                         "near_memory_limit")


if __name__ == "__main__":
    unittest.main()
