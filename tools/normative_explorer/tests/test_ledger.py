"""Manifest freezing, ledger chain and reconstruction, state machine and
budgets. Run individually through ``scripts/test_safe.py --python-test``."""
import json
import tempfile
import unittest
from pathlib import Path

from tools.normative_explorer import episode, ledger, manifest


def worker_section():
    return {
        "wire_version": "residuum-wire/1",
        "episode_id": "ep-1",
        "manifest_hash": "h-1",
        "imports": ["theory.res"],
        "workdir": "work",
        "profile": {"heads": ["x"], "body_verbs": ["y"], "evidential_not_verbs": [],
                    "max_atoms": 4, "max_binders": 16, "max_after_depth": 2,
                    "max_unit_nodes": 1, "require_connected": False},
        "decision": {"components": [], "criterion": {"component": 0}},
        "dossiers": {"d1": []},
        "budgets": {"max_actions": 10, "max_steps_per_action": 8, "max_steps_total": 50,
                    "max_sessions": 4, "max_candidates": 3, "max_clauses": 2,
                    "search_limits": {"max_depth": 32, "max_nodes": 4096, "max_branches": 4096}},
        "transport": None,
    }


def controller_section(condition="C"):
    return {
        "controller_version": "normative-explorer/0.1",
        "condition": condition,
        "model": {"provider": "simulated", "model_id": "script-1", "parameters": {}},
        "max_model_requests": 20,
        "max_actions": 15,
        "max_output_tokens": 4000,
        "valuation_source": "stipulation:test",
        "explorer_instructions": "test",
        "observations": [{"dossier": "d1", "required": True, "source": "stipulation:test", "status": "documented"}],
    }


def document(condition="C"):
    return {"worker": worker_section(), "controller": controller_section(condition)}


class ManifestTests(unittest.TestCase):
    def test_freeze_detects_any_change(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "m.json"
            path.write_text(json.dumps(document()))
            frozen = manifest.load(path)
            self.assertEqual(frozen.episode_id, "ep-1")
            self.assertEqual(frozen.worker_hash, "h-1")
            manifest.recheck(frozen)
            changed = document()
            changed["controller"]["condition"] = "A"
            path.write_text(json.dumps(changed))
            with self.assertRaises(manifest.ManifestError):
                manifest.recheck(frozen)

    def test_rejections(self):
        cases = []
        d = document(); d["worker"]["extra"] = 1; cases.append(d)
        d = document(); del d["controller"]["condition"]; cases.append(d)
        d = document(); d["controller"]["condition"] = "D"; cases.append(d)
        d = document(); d["worker"]["dossiers"] = {}; cases.append(d)
        d = document(); d["worker"]["budgets"]["max_actions"] = 0; cases.append(d)
        d = document(); d["worker"]["episode_id"] = "bad id"; cases.append(d)
        d = document(); d["controller"]["model"]["secret"] = "k"; cases.append(d)
        d = document(); d["worker"]["wire_version"] = "residuum-wire/9"; cases.append(d)
        for bad in cases:
            with self.assertRaises(manifest.ManifestError):
                manifest.validate(bad, Path("x"))
        self.assertEqual(manifest.worker_manifest_text(manifest.validate(document(), Path("x"))).count("\n") > 5, True)


class LedgerTests(unittest.TestCase):
    def test_chain_and_reconstruction(self):
        with tempple_dir() as tmp:
            led = ledger.Ledger(Path(tmp), "hash-1", clock=lambda: "t")
            a1 = led.next_id("a")
            self.assertEqual(a1, "a1")
            led.append("action.validated", {"action_id": a1, "tool": "inspect_theory", "request": {"inspect_theory": None}}, file="actions")
            digest = led.store_artifact('{"response": 1}')
            led.append("action.result", {"action_id": a1, "status": "complete", "artifact": digest})
            view = led.store_artifact('{"view": 1}')
            led.append("view.recorded", {"action_id": a1, "artifact": view}, file="visible-views")
            events = led.read_events()
            self.assertEqual([e["seq"] for e in events], [1, 2, 3])
            self.assertEqual(events[0]["prev"], ledger.GENESIS)
            self.assertEqual(events[1]["prev"], events[0]["hash"])
            rebuilt = led.reconstruct()
            self.assertEqual(rebuilt["actions"]["a1"]["status"], "complete")
            self.assertEqual(led.read_artifact(rebuilt["actions"]["a1"]["result_artifact"]), '{"response": 1}')
            self.assertEqual(led.read_file("actions")[0]["seq"], 1)
            # Resume continues the chain.
            again = ledger.Ledger(Path(tmp), "hash-1", clock=lambda: "t")
            again.append("note", {"x": 1})
            self.assertEqual(again.read_events()[-1]["seq"], 4)
            # Tampering is detected.
            path = Path(tmp) / "events.jsonl"
            lines = path.read_text().splitlines()
            lines[1] = lines[1].replace('"complete"', '"error"')
            path.write_text("\n".join(lines) + "\n")
            with self.assertRaises(ledger.LedgerError):
                ledger.Ledger(Path(tmp), "hash-1").read_events()
            with self.assertRaises(ledger.LedgerError):
                ledger.Ledger(Path(tmp), "other")

    def test_reconstruction_recognises_an_interrupted_episode(self):
        with tempple_dir() as tmp:
            led = ledger.Ledger(Path(tmp), "h", clock=lambda: "t")
            self.assertFalse(led.reconstruct()["interrupted"])
            led.append("episode.started", {})
            led.append("action.validated", {"action_id": "a1", "tool": "inspect_theory", "request": "inspect_theory"})
            rebuilt = led.reconstruct()
            self.assertTrue(rebuilt["interrupted"])
            self.assertIsNone(rebuilt["terminal"])
            led.append("episode.aborted", {"message": "worker lost"})
            rebuilt = led.reconstruct()
            self.assertFalse(rebuilt["interrupted"])
            self.assertEqual(rebuilt["terminal"], "episode.aborted")

    def test_unknown_file_is_refused(self):
        with tempple_dir() as tmp:
            led = ledger.Ledger(Path(tmp), "h")
            with self.assertRaises(ledger.LedgerError):
                led.append("x", {}, file="secrets")


class StateAndBudgetTests(unittest.TestCase):
    def test_transitions(self):
        s = episode.EpisodeState()
        for target in ("PREFLIGHT", "READY", "MODEL_REQUESTED", "MODEL_RECEIVED", "ACTION_VALIDATED",
                       "TOOL_RUNNING", "RESULT_RECORDED", "VIEW_RECORDED", "READY", "FINISHED"):
            s.to(target)
        self.assertTrue(s.terminal)
        with self.assertRaises(episode.StateError):
            s.to("READY")
        t = episode.EpisodeState()
        with self.assertRaises(episode.StateError):
            t.to("TOOL_RUNNING")

    def test_budgets(self):
        b = episode.Budgets(max_model_requests=2, max_actions=1, max_output_tokens=10)
        b.reserve_request(); b.reserve_request()
        with self.assertRaises(episode.BudgetExhausted):
            b.reserve_request()
        b.charge_action()
        with self.assertRaises(episode.BudgetExhausted):
            b.charge_action()
        self.assertEqual(b.output_cap(), 10)
        b.charge_usage({"output_tokens": 10})
        with self.assertRaises(episode.BudgetExhausted):
            b.output_cap()
        b.charge_usage(None)
        self.assertEqual(b.snapshot()["usage_unknown"], 1)


def tempple_dir():
    return tempfile.TemporaryDirectory()


if __name__ == "__main__":
    unittest.main()
