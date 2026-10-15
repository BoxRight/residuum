"""Catalogue, action validation, reference checks and A/B/C projections over
the responses recorded by the boundary crate's generator test. Run
individually through ``scripts/test_safe.py --python-test``; the recorded
fixture requires running the Rust generator test first."""
import json
import unittest
from pathlib import Path

from tools.normative_explorer import dispatcher, manifest, projections, schemas
from tools.normative_explorer.tests.test_ledger import document

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "target" / "exploration" / "process"


def recorded_lines():
    expected = FIXTURE / "expected.jsonl"
    if not expected.exists():
        raise AssertionError(
            "recorded fixture missing: run scripts/test_safe.py --test-target boundary --test "
            "process_script_tests::process_script_fixture_is_written_with_in_memory_expectation first")
    return [json.loads(l) for l in expected.read_text().splitlines()]


def event():
    return {"polarity": "positive", "verb": "x", "args": [], "kind": "derived",
            "time": {"var": "t"}, "transition": None}


def action(tool, **arguments):
    arguments.setdefault("rationale", "why")
    arguments.setdefault("expected_observation", None)
    return {"tool": tool, "arguments": arguments}


class CatalogueAndValidationTests(unittest.TestCase):
    def test_catalogue_per_condition(self):
        a, b, c = (set(schemas.catalog(k)) for k in "ABC")
        self.assertNotIn("minimal_supports", a)
        self.assertIn("minimal_supports", b)
        self.assertNotIn("compare_partition", b)
        self.assertIn("compare_partition", c)
        self.assertTrue(a < b < c)
        self.assertIn("finish_episode", a)

    def test_strict_validation(self):
        ok = schemas.validate_action(action("advance_session", session="s1", max_steps=3), "A")
        self.assertEqual(ok.tool, "advance_session")
        self.assertTrue(ok.worker_side)
        self.assertFalse(schemas.validate_action(action("finish_episode", candidate_ids=["c1"], explanation="done"), "A").worker_side)
        bad = [
            (action("advance_session", session="s1", max_steps=0), "A"),
            (action("advance_session", session="s1", max_steps=3, force=True), "A"),
            (action("advance_session", session="s 1", max_steps=3), "A"),
            (action("minimal_supports", candidate="c1", goal=event()), "A"),
            (action("compare_partition", candidates=["c1", "c1"], goal=event()), "C"),
            (action("drop_everything"), "C"),
            ({"tool": "inspect_theory"}, "C"),
            (action("begin_residual", dossier="d", goal={"verb": "x"}), "C"),
            (action("propose_candidate", source="   "), "C"),
            (action("inspect_theory", rationale="x" * 401), "C"),
            (action("inspect_theory", claims=[{"kind": "residuum_result", "assertion": None}]), "C"),
            (action("inspect_theory", claims=[{"kind": "residuum_result", "text": "t",
                                               "assertion": {"ref": "a1", "attribute": "colour", "value": "v"}}]), "C"),
            (action("inspect_theory", legal_hypothesis={"statement": "s", "grounds": [], "status": "valuation"}), "C"),
        ]
        for act, condition in bad:
            with self.assertRaises(schemas.ProtocolError, msg=str(act)):
                schemas.validate_action(act, condition)
        # Claims and hypotheses in the agreed shape are accepted and kept.
        rich = schemas.validate_action(action(
            "inspect_theory",
            claims=[{"kind": "residuum_result", "text": "the impediment was derived",
                     "assertion": {"ref": "a12", "attribute": "component", "value": "1=true"}}],
            legal_hypothesis={"statement": "no profit from one's own wrong", "grounds": ["Riggs"], "status": "hypothesis"},
        ), "A")
        self.assertEqual(rich.claims[0]["assertion"]["ref"], "a12")
        self.assertEqual(rich.legal_hypothesis["status"], "hypothesis")


class DispatcherTests(unittest.TestCase):
    def setUp(self):
        self.manifest = manifest.validate(document("B"), Path("m.json"))
        self.registry = dispatcher.Registry()
        self.dispatcher = dispatcher.Dispatcher(self.manifest, self.registry)

    def test_envelope_and_reference_checks(self):
        env = self.dispatcher.envelope("a1", schemas.validate_action(action("inspect_theory"), "B"))
        self.assertEqual(env["manifest_hash"], "h-1")
        self.assertEqual(env["episode_id"], "ep-1")
        self.assertEqual(env["request"], {"inspect_theory": None})
        with self.assertRaises(schemas.ProtocolError):
            self.dispatcher.envelope("a2", schemas.validate_action(action("advance_session", session="s1", max_steps=1), "B"))
        self.registry.learn({"payload": {"session_opened": {"session": "s1", "kind": "forward"}}})
        env = self.dispatcher.envelope("a2", schemas.validate_action(action("advance_session", session="s1", max_steps=1), "B"))
        self.assertEqual(env["request"]["advance_session"]["max_steps"], 1)
        with self.assertRaises(schemas.ProtocolError):
            self.dispatcher.envelope("a3", schemas.validate_action(action("begin_forward", dossier="nope"), "B"))
        with self.assertRaises(schemas.ProtocolError):
            self.dispatcher.envelope("a3", schemas.validate_action(action("begin_forward", dossier="d1", theory="c1"), "B"))
        self.registry.learn({"payload": {"candidate": {"candidate": "c1", "report": {"status": "syntax"}}}})
        env = self.dispatcher.envelope("a3", schemas.validate_action(action("begin_forward", dossier="d1", theory="c1"), "B"))
        self.assertEqual(env["request"]["begin_forward"]["theory"], "c1")
        # A tool outside the condition's catalogue never becomes an envelope.
        with self.assertRaises(schemas.ProtocolError):
            schemas.validate_action(action("compare_partition", candidates=["c1"], goal=event()), "B")
        # Controller-side tools have no worker request.
        with self.assertRaises(schemas.ProtocolError):
            self.dispatcher.to_request(schemas.validate_action(action("request_valuation", case_spec={}, expected_discrimination="x"), "B"))


class ProjectionTests(unittest.TestCase):
    def test_recorded_responses_project_without_leaks(self):
        lines = recorded_lines()
        self.assertEqual(len(lines), 17)
        kinds = set()
        for line in lines:
            for condition, reserved in (("A", projections.RESERVED_A), ("B", projections.RESERVED_B), ("C", projections.NEVER)):
                try:
                    view = projections.project(line, condition)
                except projections.ProjectionError:
                    # Only payloads outside the condition's catalogue are refused.
                    kind = next(iter(line["response"]["payload"]))
                    self.assertIn((condition, kind), {("A", "supports"), ("B", "partition"), ("A", "partition")})
                    continue
                leaked = projections.keys_of(view) & reserved
                self.assertFalse(leaked, f"{condition}: {leaked}")
                if "response" in line:
                    kinds.add(next(iter(line["response"]["payload"])))
                    self.assertEqual(view["response"]["action_id"], line["response"]["action_id"])
                    self.assertEqual(view["response"]["status"], line["response"]["status"])
        self.assertTrue({"forward", "evaluation", "residual", "candidate", "supports", "theory", "error"} <= kinds)
        # The full record keeps what the views drop.
        forward = next(l for l in lines if "response" in l and "forward" in l["response"]["payload"]
                       and l["response"]["payload"]["forward"]["complete"])
        full_derivations = sum(len(r["derivations"]) for r in forward["response"]["payload"]["forward"]["rounds"])
        self.assertGreater(full_derivations, 0)
        a = projections.project(forward, "A")["response"]["payload"]["forward"]
        b = projections.project(forward, "B")["response"]["payload"]["forward"]
        self.assertTrue(all("derivations" not in r for r in a["rounds"]))
        self.assertEqual(sum(len(r["derivations"]) for r in b["rounds"]), full_derivations)
        self.assertEqual(a["known"], forward["response"]["payload"]["forward"]["known"])
        evaluation = next(l for l in lines if "response" in l and "evaluation" in l["response"]["payload"])
        a = projections.project(evaluation, "A")["response"]["payload"]["evaluation"]
        self.assertNotIn("closure", a)
        self.assertEqual(a["report"]["status"]["evaluated"]["certificate"], {"holds": True})
        self.assertNotIn("file", a["report"])
        c = projections.project(evaluation, "C")["response"]["payload"]["evaluation"]
        self.assertIn("closure", c)
        self.assertIn("events_preserved", c["report"]["status"]["evaluated"]["certificate"])
        residual = next(l for l in lines if "response" in l and "residual" in l["response"]["payload"]
                        and l["response"]["payload"]["residual"]["error"])
        a = projections.project(residual, "A")["response"]["payload"]["residual"]
        self.assertTrue(a["error"])
        self.assertTrue(all("branches" not in r for r in a["rounds"]))

    def test_faults_and_unknown_kinds(self):
        fault = {"fault": {"wire_version": "residuum-wire/1", "class": "framing", "action_id": None, "message": "m"}}
        self.assertEqual(projections.project(fault, "A"), {"fault": {"class": "framing", "action_id": None, "message": "m"}})
        with self.assertRaises(projections.ProjectionError):
            projections.project({"response": {"action_id": "a1", "status": "complete", "steps_consumed": 0,
                                              "budget": {}, "payload": {"secret": {}}}}, "C")
        with self.assertRaises(projections.ProjectionError):
            projections.project(fault, "D")


if __name__ == "__main__":
    unittest.main()
