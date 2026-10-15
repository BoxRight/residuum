"""The pilot manifest under review validates, selects the Anthropic
provider without a policy, and builds the documented request offline.
Run individually through ``scripts/test_safe.py --python-test``."""
import json
import unittest
from pathlib import Path

from tools.normative_explorer import manifest, provider_anthropic, run_episode, schemas

ROOT = Path(__file__).resolve().parents[3]
PILOT = ROOT / "docs" / "legal-geometry" / "exploration" / "pilot-001" / "episode.json"


class PilotManifestTests(unittest.TestCase):
    def test_pilot_manifest_is_valid_and_builds_the_documented_request(self):
        m = manifest.load(PILOT)
        model = m.controller["model"]
        self.assertEqual((model["provider"], model["model_id"]), ("anthropic", "claude-sonnet-5-5"))
        self.assertEqual(model["client"], {"timeout_seconds": 120})
        self.assertEqual(m.controller["max_cost_usd"], 2.5)
        self.assertEqual(m.controller["pricing"], {"input_per_mtok": 2.0, "output_per_mtok": 10.0,
                                                   "cache_write_multiplier": 1.25, "cache_read_per_mtok": 0.10})
        self.assertEqual(manifest.instrumentation_problems(m), [])
        self.assertEqual(m.worker["budgets"]["max_actions"], m.controller["max_actions"] + manifest.TECHNICAL_ACTIONS)
        self.assertEqual([o["status"] for o in m.controller["observations"]], ["documented"])
        built = []
        provider, policy = run_episode.build_provider(m, None, {"anthropic": lambda t: built.append(t) or object()})
        self.assertIsNone(policy)
        self.assertEqual(built, [120])
        provider.vocabulary = {"verbs": []}
        body = provider.build_request([], schemas.catalog(m.condition), 16000)
        input_upper, output_upper = provider.request_bounds([], schemas.catalog(m.condition), 16000)
        self.assertGreater(input_upper, len(json.dumps(body, ensure_ascii=False).encode()))
        self.assertEqual(output_upper, 16000)
        self.assertEqual(body["output_config"], {"effort": "medium"})
        self.assertEqual(body["thinking"], {"type": "adaptive"})
        self.assertNotIn("fallbacks", body)
        text = json.dumps(body)
        self.assertNotIn("candidateImpediment", text, "no catalogue rule is sent")
        self.assertNotIn("wrkspc_", text)
        self.assertNotIn("sk-ant", text)
        with self.assertRaises(Exception):
            run_episode.build_provider(m, "branching", {"anthropic": lambda t: object()})
        # Worker paths are relative to this directory and resolve inside the repository.
        for value in m.worker["imports"] + [m.worker["workdir"]]:
            self.assertFalse(Path(value).is_absolute())
            self.assertTrue(str((PILOT.parent / value).resolve()).startswith(str(ROOT)))


if __name__ == "__main__":
    unittest.main()
