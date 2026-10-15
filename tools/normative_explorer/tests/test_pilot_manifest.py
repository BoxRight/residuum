"""The pilot manifest under review validates, selects the Anthropic
provider without a policy, and builds the documented request offline.
Run individually through ``scripts/test_safe.py --python-test``."""
import json
import unittest
from pathlib import Path

from tools.normative_explorer import manifest, provider_anthropic, run_episode, schemas

ROOT = Path(__file__).resolve().parents[3]
PILOT = ROOT / "docs" / "legal-geometry" / "exploration" / "pilot-001" / "episode.json"
PILOT2 = ROOT / "docs" / "legal-geometry" / "exploration" / "pilot-002" / "episode.json"
PILOT3 = ROOT / "docs" / "legal-geometry" / "exploration" / "pilot-003" / "episode.json"


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


    def test_pilot_002_differs_from_pilot_001_only_in_identity_and_paths(self):
        first, second = json.loads(PILOT.read_text()), json.loads(PILOT2.read_text())
        for doc in (first, second):
            doc["worker"].pop("episode_id"), doc["worker"].pop("manifest_hash"), doc["worker"].pop("workdir")
        self.assertEqual(first, second)
        m = manifest.load(PILOT2)
        self.assertEqual((m.worker["episode_id"], m.worker["manifest_hash"]), ("riggs-pilot-002", "h-pilot-002"))
        self.assertTrue(m.worker["workdir"].endswith("target/exploration/pilot-002/work"))
        self.assertEqual(manifest.instrumentation_problems(m), [])
        # The integration difference: a mixed catalogue within the limits.
        provider, _ = run_episode.build_provider(m, None, {"anthropic": lambda t: object()})
        totals = provider_anthropic.check_strict_limits(provider.tools)
        self.assertEqual(totals, {"strict_tools": 4, "optional_parameters": 0, "union_parameters": 16})

    def test_pilot_003_request_differs_from_pilot_002_only_in_strict_flags(self):
        def body(path):
            m = manifest.load(path)
            provider, _ = run_episode.build_provider(m, None, {"anthropic": lambda t: object()})
            provider.vocabulary = {"verbs": [{"name": "kills", "arity": 2}], "entities": [], "constants": []}
            return m, provider.build_request([], schemas.catalog(m.condition), 16000)
        m2, second = body(PILOT2)
        m3, third = body(PILOT3)
        self.assertEqual(m3.controller["model"]["tool_strictness"], "none")
        self.assertEqual(sum(t["strict"] for t in second["tools"]), 4)
        self.assertEqual(sum(t["strict"] for t in third["tools"]), 0)
        # Pilot 002's request with every strict flag set to false and its
        # identity renamed is exactly pilot 003's request: same tools, schemas,
        # descriptions, arguments, instructions, theory, budgets.
        for tool in second["tools"]:
            tool["strict"] = False
        renamed = json.loads(json.dumps(second).replace("riggs-pilot-002", "riggs-pilot-003")
                             .replace("h-pilot-002", "h-pilot-003"))
        self.assertEqual(renamed, third)
        # And the manifests differ only in identity, paths and strictness.
        a, b = json.loads(PILOT2.read_text()), json.loads(PILOT3.read_text())
        b["controller"]["model"].pop("tool_strictness")
        for doc in (a, b):
            doc["worker"].pop("episode_id"), doc["worker"].pop("manifest_hash"), doc["worker"].pop("workdir")
        self.assertEqual(a, b)
        self.assertEqual(manifest.instrumentation_problems(m3), [])


if __name__ == "__main__":
    unittest.main()
