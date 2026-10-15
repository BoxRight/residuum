"""Controller with the simulated provider over the recorded worker lines and
over stubs: full cycle, projections per condition, candidate tracking
against the resolved observation, claims, budgets, protocol errors, and the
rule that no technical failure becomes normative incompatibility. Run
individually through ``scripts/test_safe.py --python-test``; the recorded
fixture requires the Rust generator test first."""
import json
import tempfile
import unittest
from pathlib import Path

from tools.normative_explorer import backend, controller, ledger, manifest, provider_sim

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "target" / "exploration" / "process"
SCRATCH = ROOT / "target" / "exploration" / "controller-tests"


def scratch(prefix):
    """Ledgers are read after the episode ends, so they live under target/
    rather than in a temporary directory that vanishes with the block."""
    SCRATCH.mkdir(parents=True, exist_ok=True)
    return tempfile.mkdtemp(prefix=prefix, dir=SCRATCH)


def recorded():
    if not (FIXTURE / "expected.jsonl").exists():
        raise AssertionError("recorded fixture missing: run the boundary generator test first")
    worker = json.loads((FIXTURE / "manifest.json").read_text())
    requests = {}
    for line in (FIXTURE / "requests.jsonl").read_text().split("\n")[:-1]:
        try:
            env = json.loads(line)
        except json.JSONDecodeError:
            continue
        requests.setdefault(env["action_id"], env["request"])
    return worker, requests


def controller_section(condition, **overrides):
    section = {
        "controller_version": "normative-explorer/0.1",
        "condition": condition,
        "model": {"provider": "simulated", "model_id": "script-1", "parameters": {}},
        "max_model_requests": 40,
        "max_actions": 30,
        "max_output_tokens": 100000,
        "valuation_source": "riggs_rule_observations.tsv",
        "explorer_instructions": "test",
        "observations": [{"dossier": "historicalRiggs", "required": True,
                          "source": "riggs_rule_observations.tsv", "status": "documented"}],
    }
    section.update(overrides)
    return section


def write_manifest(tmp, worker, condition, **overrides):
    path = Path(tmp) / "episode.json"
    path.write_text(json.dumps({"worker": worker, "controller": controller_section(condition, **overrides)}))
    return manifest.load(path)


def act(tool, **arguments):
    arguments.setdefault("rationale", "scripted")
    arguments.setdefault("expected_observation", None)
    return {"tool": tool, "arguments": arguments}


def recorded_script(requests, condition):
    """Mirror of the Rust generator's decodable requests, as model actions."""
    s = [
        act("inspect_theory"),
        act("begin_forward", dossier="historicalRiggs"),
        act("advance_session", session="s1", max_steps=2),
        act("advance_session", session="s1", max_steps=8,
            claims=[{"kind": "residuum_result", "text": "the previous advance was pending",
                     "assertion": {"ref": "a3", "attribute": "status", "value": "step_limit"}},
                    {"kind": "hypothesis_change", "text": "a3 and a4 differ",
                     "assertion": {"ref": "a4", "attribute": "differs_from", "value": "a3"}},
                    {"kind": "residuum_result", "text": "a transfer appeared",
                     "assertion": {"ref": "a4", "attribute": "contains_event", "value": "transferred"}},
                    {"kind": "residuum_result", "text": "ghost reference",
                     "assertion": {"ref": "a99", "attribute": "status", "value": "complete"}},
                    {"kind": "investigation_reason", "text": "prose only", "assertion": None}]),
        act("propose_candidate", source=requests["a5"]["propose_candidate"]["text"],
            legal_hypothesis={"statement": "no one may profit from their own wrong",
                              "grounds": ["Riggs v. Palmer"], "status": "hypothesis"}),
        act("propose_candidate", source=requests["a6"]["propose_candidate"]["text"]),
        act("begin_candidate_evaluation", candidate="c1", dossier="historicalRiggs"),
        act("advance_session", session="s2", max_steps=8,
            claims=[{"kind": "residuum_result", "text": "the impediment was derived",
                     "assertion": {"ref": "a8", "attribute": "component", "value": "1=true"}},
                    {"kind": "residuum_result", "text": "the candidate rule justified the impediment",
                     "assertion": {"ref": "a8", "attribute": "contains_event",
                                   "value": "hypotheticalSuccessionImpediment"}}]),
        act("begin_residual", dossier="historicalRiggs", goal=requests["a9"]["begin_residual"]["goal"], bindings=[]),
        act("advance_session", session="s3", max_steps=8),
        # Protocol error at the controller: the session was never opened.
        act("advance_session", session="s99", max_steps=1),
    ]
    if condition != "A":
        s.append(act("minimal_supports", candidate="c1", goal=requests["a12"]["minimal_supports"]["goal"]))
    else:
        # Not offered in A: a protocol error, never a worker call.
        s.append(act("minimal_supports", candidate="c1", goal=requests["a12"]["minimal_supports"]["goal"]))
    s.append(act("request_valuation", case_spec={"dossier": "killingWithoutAcquisitionPurpose"},
                 expected_discrimination="purpose requirement"))
    s.append(act("finish_episode", candidate_ids=["c1"], explanation="c1 compatible with the historical observation"))
    return s


class RecordedEpisodeTests(unittest.TestCase):
    def run_condition(self, condition):
        worker, requests = recorded()
        tmp = scratch(f"recorded-{condition}-")
        m = write_manifest(tmp, worker, condition)
        led = ledger.Ledger(Path(tmp) / "ledger", m.episode_hash, clock=lambda: "t")
        be = backend.RecordedBackend(FIXTURE / "requests.jsonl", FIXTURE / "expected.jsonl")
        provider = provider_sim.ScriptedProvider(recorded_script(requests, condition))
        ctl = controller.Controller(m, led, be, provider)
        outcome = ctl.run()
        return ctl, led, be, provider, outcome

    def test_full_cycle_condition_c(self):
        ctl, led, be, provider, outcome = self.run_condition("C")
        self.assertEqual(outcome, "verified_compatible_candidate")
        self.assertEqual(ctl.state.state, "FINISHED")
        # Worker calls: eleven valid actions; the s99 reference never left the controller.
        self.assertEqual([e["action_id"] for e in be.calls], [f"a{i}" for i in range(1, 12)])
        self.assertTrue(all(e["manifest_hash"] == "h-process" for e in be.calls))
        rebuilt = led.reconstruct()
        self.assertEqual(set(rebuilt["actions"]), {f"a{i}" for i in range(1, 12)})
        statuses = [rebuilt["actions"][f"a{i}"]["status"] for i in range(1, 12)]
        self.assertEqual(statuses, ["complete", "complete", "step_limit", "complete", "complete",
                                    "structural_reject", "complete", "complete", "complete",
                                    "unresolved_parameters", "complete"])
        actions_file = led.read_file("actions")
        self.assertEqual(sum(1 for a in actions_file if a["kind"] == "protocol_error"), 1)
        self.assertIn("unknown session", next(a for a in actions_file if a["kind"] == "protocol_error")["message"])
        # Candidate tracking against the resolved observation; generation,
        # filtering and partition events are separate records.
        self.assertEqual(ctl.classify_candidate("c1"), "verified")
        candidates = led.read_file("candidates")
        self.assertEqual([c["kind"] for c in candidates], ["candidate.generated", "candidate.rejected"])
        progress = led.read_file("progress")
        kinds = [p["kind"] for p in progress]
        self.assertEqual(kinds.count("filter.applied"), 1)
        self.assertEqual(kinds.count("partition.refined"), 0)
        self.assertEqual(led.read_file("partitions"), [])
        self.assertEqual(led.read_file("valuations")[0]["status"], "pending")
        # The legal hypothesis is recorded with the action, never as a valuation.
        proposal = next(a for a in actions_file if a.get("tool") == "propose_candidate")
        self.assertEqual(proposal["legal_hypothesis"]["status"], "hypothesis")
        # Claims: deterministic verdicts.
        verdicts = {(c["assertion"]["ref"] if c["assertion"] else None, c["assertion"]["attribute"] if c["assertion"] else None): c["verdict"]
                    for c in led.read_file("claims")}
        self.assertEqual(verdicts[("a3", "status")], "match")
        self.assertEqual(verdicts[("a4", "differs_from")], "match")
        self.assertEqual(verdicts[("a4", "contains_event")], "mismatch")
        self.assertEqual(verdicts[("a99", "status")], "unreferenced")
        self.assertEqual(verdicts[(None, None)], "unchecked")
        self.assertEqual(verdicts[("a8", "component")], "match")
        self.assertEqual(verdicts[("a8", "contains_event")], "match")
        # The model saw C views: provenance present, every tool offered.
        self.assertIn("compare_partition", provider.seen_tools[0])
        a4 = next(h for h in ctl.history if h["action_id"] == "a4")
        self.assertTrue(any("derivations" in r for r in a4["view"]["response"]["payload"]["forward"]["rounds"]))
        finished = next(p for p in progress if p["kind"] == "episode.finished")
        self.assertEqual(finished["referenced"], {"c1": "verified"})
        self.assertIn("not a legal approval", finished["scope"])

    def test_condition_a_delivers_no_provenance_and_refuses_b_tools(self):
        ctl, led, be, provider, outcome = self.run_condition("A")
        self.assertEqual(outcome, "verified_compatible_candidate")
        self.assertNotIn("minimal_supports", provider.seen_tools[0])
        self.assertEqual(len(be.calls), 10, "minimal_supports never reached the worker in A")
        protocol_errors = [a for a in led.read_file("actions") if a["kind"] == "protocol_error"]
        self.assertEqual(len(protocol_errors), 2)
        self.assertTrue(any("not offered in condition A" in e["message"] for e in protocol_errors))
        from tools.normative_explorer.projections import RESERVED_A, keys_of
        for item in ctl.history:
            self.assertFalse(keys_of(item["view"]) & RESERVED_A, item["action_id"])
        # The full record still has provenance.
        a4 = ctl.records["a4"]
        self.assertTrue(sum(len(r["derivations"]) for r in a4["response"]["payload"]["forward"]["rounds"]) > 0)
        # A claim on data A does not receive is out_of_view, not a mismatch.
        self.assertEqual(next(c["verdict"] for c in led.read_file("claims")
                              if c["assertion"] and c["assertion"]["attribute"] == "contains_event"
                              and c["assertion"]["ref"] == "a8"), "out_of_view")


def stub_response(action_id, status, payload):
    return {"response": {"action_id": action_id, "status": status, "steps_consumed": 0,
                         "budget": {"actions_remaining": 9, "steps_remaining": 9, "sessions_remaining": 9,
                                    "candidates_remaining": 9}, "payload": payload}}


def report(cid, status):
    return {"id": cid, "module_name": f"Candidate{cid}", "original_text": "rule r() = I <= x() @ t", "source": "...",
            "file": "/private/path", "rules": [], "stages": ["parsed", "elaborated", "extracted", "admitted"],
            "status": status}


class FailurePathTests(unittest.TestCase):
    def setUp(self):
        self.worker, _ = recorded()

    def test_technical_failures_never_become_incompatibility(self):
        certificate = {"events_preserved": False, "proofs_preserved": True, "added_heads": ["x"], "foreign_heads": ["owns"]}
        lines = [
            lambda e: stub_response(e["action_id"], "complete", {"candidate": {"candidate": "c1", "report": report("c1", "syntax")}}),
            lambda e: stub_response(e["action_id"], "complete", {"session_opened": {"session": "s1", "kind": "evaluation"}}),
            lambda e: stub_response(e["action_id"], "step_limit", {"evaluation": {
                "session": "s1", "candidate": "c1", "dossier": "historicalRiggs", "rounds": [], "complete": False,
                "closure": None, "report": None}}),
            lambda e: stub_response(e["action_id"], "complete", {"evaluation": {
                "session": "s1", "candidate": "c1", "dossier": "historicalRiggs", "rounds": [], "complete": True,
                "closure": {"known": [], "derivations": []},
                "report": report("c1", {"certificate_failed": certificate})}}),
            lambda e: {"fault": {"wire_version": "residuum-wire/1", "class": "output_limit", "action_id": e["action_id"],
                                 "message": "not deliverable under this budget"}},
            lambda e: stub_response(e["action_id"], "error", {"error": {"message": "budget: steps exhausted"}}),
        ]
        script = [
            act("propose_candidate", source="rule r() = I <= x() @ t"),
            act("begin_candidate_evaluation", candidate="c1", dossier="historicalRiggs"),
            act("advance_session", session="s1", max_steps=1),
            act("advance_session", session="s1", max_steps=8),
            act("inspect_theory"),
            act("inspect_theory"),
            act("finish_episode", candidate_ids=["c1"], explanation="pending"),
        ]
        tmp = scratch("technical-")
        m = write_manifest(tmp, self.worker, "B")
        led = ledger.Ledger(Path(tmp) / "ledger", m.episode_hash, clock=lambda: "t")
        ctl = controller.Controller(m, led, backend.StubBackend(lines), provider_sim.ScriptedProvider(script))
        outcome = ctl.run()
        self.assertEqual(outcome, "awaiting_independent_valuation")
        self.assertEqual(ctl.classify_candidate("c1"), "pending")
        record = ctl.candidates["c1"].evaluations["historicalRiggs"]
        self.assertEqual(record["status"], "certificate_failed")
        self.assertIsNone(record["compatible"])
        progress = led.read_file("progress")
        self.assertFalse(any(p["kind"] == "filter.applied" for p in progress))
        self.assertTrue(any(p["kind"] == "evaluation.technical" and p["status"] == "certificate_failed" for p in progress))
        trace = led.read_file("evaluator-trace")
        self.assertIn("fault:output_limit", [t["status"] for t in trace])
        self.assertIn("error", [t["status"] for t in trace])
        text = json.dumps(led.read_file("progress"))
        self.assertNotIn("incompatible", text)

    def test_provider_cannot_change_the_contract_and_budgets_end_the_episode(self):
        script = [
            {"tool": "set_manifest", "arguments": {"budgets": {}, "rationale": "x", "expected_observation": None}},
            {"tool": "inspect_theory", "arguments": {"condition": "C", "rationale": "x", "expected_observation": None}},
            {"tool": "begin_forward", "arguments": {"dossier": "historicalRiggs", "theory": "c7", "rationale": "x",
                                                    "expected_observation": None}},
            "not even an object",
            act("inspect_theory"),
            act("inspect_theory"),
        ]
        with tempfile.TemporaryDirectory() as tmp:
            m = write_manifest(tmp, self.worker, "A", max_actions=4, max_model_requests=10)
            led = ledger.Ledger(Path(tmp) / "ledger", m.episode_hash, clock=lambda: "t")
            be = backend.StubBackend([lambda e: stub_response(e["action_id"], "complete", {"theory": {
                "module": "Baseline", "rules": [], "seeded_verbs": [], "derived_verbs": [], "effect_verbs": [],
                "dossiers": ["historicalRiggs"]}})] * 3)
            ctl = controller.Controller(m, led, be, provider_sim.ScriptedProvider(script))
            outcome = ctl.run()
            self.assertEqual(outcome, "budget_exhausted")
            errors = [a["message"] for a in led.read_file("actions") if a["kind"] == "protocol_error"]
            self.assertEqual(len(errors), 4)
            self.assertTrue(any("unknown tool" in e for e in errors))
            self.assertTrue(any("unknown arguments" in e for e in errors))
            self.assertTrue(any("not an admitted candidate" in e for e in errors))
            self.assertEqual(len(be.calls), 0, "no invalid action reached the worker; budget spent on attempts")
            self.assertEqual(m.condition, "A")
            # A manifest changed on disk aborts the next turn.
            m2 = write_manifest(tmp, self.worker, "A")
            led2 = ledger.Ledger(Path(tmp) / "ledger2", m2.episode_hash, clock=lambda: "t")
            ctl2 = controller.Controller(m2, led2, be, provider_sim.ScriptedProvider([act("inspect_theory")]))
            ctl2.preflight()
            (Path(tmp) / "episode.json").write_text(json.dumps({"worker": self.worker, "controller": controller_section("C")}))
            ctl2.turn()
            self.assertEqual(ctl2.state.state, "ABORTED")
            self.assertEqual(ctl2.outcome, "technical_abort")
            self.assertTrue(any(p["kind"] == "episode.aborted" and "changed on disk" in p["message"]
                                for p in led2.read_file("progress")))


if __name__ == "__main__":
    unittest.main()
