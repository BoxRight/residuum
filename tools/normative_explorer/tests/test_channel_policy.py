"""Channel backend over prepared descriptors (no process), the branching
policy through the controller, and the supervised entry point's argument
and failure handling. Run individually through ``scripts/test_safe.py
--python-test``."""
import json
import os
import unittest
from pathlib import Path

from tools.normative_explorer import backend, channel, controller, ledger, policies, provider_sim, run_episode
from tools.normative_explorer.tests.test_controller import (act, controller_section, recorded, report, scratch,
                                                            stub_response, write_manifest)


def response_line(action_id, status="complete", payload=None):
    return json.dumps({"response": {"wire_version": "residuum-wire/1", "action_id": action_id, "status": status,
                                    "steps_consumed": 0, "budget": {"actions_remaining": 1, "steps_remaining": 1,
                                                                    "sessions_remaining": 1, "candidates_remaining": 1},
                                    "payload": payload or {"theory": {"module": "B", "rules": [], "seeded_verbs": [],
                                                                      "derived_verbs": [], "effect_verbs": [],
                                                                      "dossiers": []}}}}) + "\n"


def envelope(action_id):
    return {"wire_version": "residuum-wire/1", "protocol_version": "residuum-worker/0.1", "episode_id": "e",
            "action_id": action_id, "manifest_hash": "h", "request": "inspect_theory"}


class ChannelTests(unittest.TestCase):
    def make(self, **kwargs):
        to_r, to_w = os.pipe()        # controller writes, "worker" reads
        from_r, from_w = os.pipe()    # "worker" writes, controller reads
        be = channel.ChannelBackend(os.fdopen(to_w, "wb", buffering=0), os.fdopen(from_r, "rb", buffering=0),
                                    request_timeout=kwargs.pop("request_timeout", 1.0), **kwargs)
        return be, os.fdopen(to_r, "rb", buffering=0), os.fdopen(from_w, "wb", buffering=0)

    def test_request_is_written_and_answer_correlated(self):
        be, worker_in, worker_out = self.make()
        worker_out.write(response_line("a1").encode())
        line = be.handle(envelope("a1"))
        self.assertEqual(line["response"]["action_id"], "a1")
        self.assertNotIn("wire_version", line["response"])
        sent = json.loads(worker_in.readline())
        self.assertEqual(sent["action_id"], "a1")
        self.assertEqual(sent["request"], "inspect_theory")
        self.assertEqual(be.calls, 1)
        # A fault without an action id is accepted; one for another id is not.
        worker_out.write((json.dumps({"fault": {"wire_version": "residuum-wire/1", "class": "framing",
                                                 "action_id": None, "message": "m"}}) + "\n").encode())
        self.assertIn("fault", be.handle(envelope("a2")))
        worker_out.write(response_line("a9").encode())
        with self.assertRaises(channel.ChannelError):
            be.handle(envelope("a3"))
        be.close()
        with self.assertRaises(channel.ChannelError):
            be.handle(envelope("a4"))

    def test_framing_eof_timeout_and_size_are_technical_errors(self):
        be, worker_in, worker_out = self.make(request_timeout=0.2)
        worker_out.write(b"\n")
        with self.assertRaisesRegex(channel.ChannelError, "empty line"):
            be.handle(envelope("a1"))
        worker_out.write(b"garbage {\n")
        with self.assertRaisesRegex(channel.ChannelError, "misframed"):
            be.handle(envelope("a2"))
        worker_out.write(b'{"other": 1}\n')
        with self.assertRaisesRegex(channel.ChannelError, "neither"):
            be.handle(envelope("a3"))
        worker_out.write(response_line("a4").replace("residuum-wire/1", "residuum-wire/9").encode())
        with self.assertRaisesRegex(channel.ChannelError, "wire version"):
            be.handle(envelope("a4"))
        with self.assertRaisesRegex(channel.ChannelError, "no answer within"):
            be.handle(envelope("a5"))
        small, worker_in2, worker_out2 = self.make(max_line_bytes=200, request_timeout=0.5)
        worker_out2.write(b"x" * 300 + b"\n")
        with self.assertRaisesRegex(channel.ChannelError, "exceeds 200 bytes"):
            small.handle(envelope("a6"))
        with self.assertRaisesRegex(channel.ChannelError, "exceeds the line limit"):
            small.handle(dict(envelope("a7"), request={"propose_candidate": {"text": "y" * 400}}))
        worker_out.close()
        with self.assertRaisesRegex(channel.ChannelError, "unexpected EOF"):
            be.handle(envelope("a8"))
        # Writing into a pipe nobody reads is a lost worker.
        lost, worker_in3, worker_out3 = self.make()
        worker_in3.close()
        with self.assertRaisesRegex(channel.ChannelError, "worker lost"):
            lost.handle(envelope("a9"))


def session(sid, kind):
    return {"session_opened": {"session": sid, "kind": kind}}


def forward(sid, complete, known=()):
    return {"forward": {"session": sid, "rounds": [{"round": 1, "added": [], "derivations": [], "witnesses": 0,
                                                     "complete": complete}], "known": list(known), "complete": complete}}


def evaluation(sid, complete, compatible=True):
    body = {"session": sid, "candidate": "c1", "dossier": "historicalRiggs", "rounds": [], "complete": complete,
            "closure": {"known": [], "derivations": []} if complete else None, "report": None}
    if complete:
        body["report"] = report("c1", {"evaluated": {"components": [True, compatible], "compatible": compatible,
                                                     "certificate": {"events_preserved": True, "proofs_preserved": True,
                                                                     "added_heads": ["x"], "foreign_heads": []}}})
    return {"evaluation": body}


class PolicyTests(unittest.TestCase):
    def run_policy(self, condition, lines):
        worker, _ = recorded()
        tmp = scratch(f"policy-{condition}-")
        m = write_manifest(tmp, worker, condition)
        led = ledger.Ledger(Path(tmp) / "ledger", m.episode_hash, clock=lambda: "t")
        goal = dict(m.worker["decision"]["components"][-1])
        policy = policies.BranchingPolicy(goal)
        be = backend.StubBackend([lambda e, p=p: stub_response(e["action_id"], p.get("status", "complete"), p["payload"])
                                  for p in lines])
        ctl = controller.Controller(m, led, be, run_episode.PolicyProvider(policy))
        return ctl.run(), ctl, be, policy, led

    def test_branches_on_pending_results_in_condition_c(self):
        lines = [
            {"payload": {"theory": {"module": "B", "rules": [], "seeded_verbs": [], "derived_verbs": [], "effect_verbs": [], "dossiers": []}}},
            {"payload": session("s1", "forward")},
            {"payload": forward("s1", False), "status": "step_limit"},
            {"payload": forward("s1", False), "status": "step_limit"},
            {"payload": forward("s1", True)},
            {"payload": {"candidate": {"candidate": "c1", "report": report("c1", "syntax")}}},
            {"payload": session("s2", "evaluation")},
            {"payload": evaluation("s2", False), "status": "step_limit"},
            {"payload": evaluation("s2", True, True)},
            {"payload": {"supports": {"candidate": "c1", "goal_in_universe": True, "frontier": [[]]}}},
            {"payload": session("s3", "forward")},
            {"payload": forward("s3", True, known=[])},
        ]
        outcome, ctl, be, policy, led = self.run_policy("C", lines)
        self.assertEqual(outcome, "verified_compatible_candidate")
        self.assertEqual(len(be.calls), 12)
        tools_sent = [next(iter(e["request"])) if isinstance(e["request"], dict) else e["request"] for e in be.calls]
        self.assertEqual(tools_sent.count("advance_session"), 6)
        self.assertEqual(policy.decisions, [
            "forward pending -> advance again", "forward pending -> advance again", "forward complete -> propose",
            "evaluation pending -> advance again", "compatible -> minimal_supports", "extra dossier complete -> finish",
        ])
        extra = next(e for e in be.calls if isinstance(e["request"], dict) and "begin_forward" in e["request"]
                     and e["request"]["begin_forward"]["dossier"] == "killingWithoutAcquisitionPurpose")
        self.assertEqual(extra["action_id"], "a11")
        claims = led.read_file("claims")
        self.assertEqual(claims[0]["verdict"], "match")

    def test_condition_a_requests_a_valuation_instead_of_supports(self):
        lines = [
            {"payload": {"theory": {"module": "B", "rules": [], "seeded_verbs": [], "derived_verbs": [], "effect_verbs": [], "dossiers": []}}},
            {"payload": session("s1", "forward")},
            {"payload": forward("s1", True)},
            {"payload": {"candidate": {"candidate": "c1", "report": report("c1", "syntax")}}},
            {"payload": session("s2", "evaluation")},
            {"payload": evaluation("s2", True, True)},
            {"payload": session("s3", "forward")},
            {"payload": forward("s3", True)},
        ]
        outcome, ctl, be, policy, led = self.run_policy("A", lines)
        self.assertEqual(outcome, "verified_compatible_candidate")
        self.assertIn("compatible, A -> request valuation", policy.decisions)
        self.assertEqual(len(be.calls), 8)
        self.assertEqual(led.read_file("valuations")[0]["status"], "pending")


class EntryPointTests(unittest.TestCase):
    def test_arguments_and_failures(self):
        err = []
        class Err:
            def write(self, s): err.append(s)
            def flush(self): pass
        self.assertEqual(run_episode.main([], stderr=Err()), 2)
        self.assertEqual(run_episode.main(["--manifest", "m", "--ledger", "l", "--policy", "nope",
                                           "--to-worker", "3", "--from-worker", "4"], stderr=Err()), 2)
        self.assertEqual(run_episode.main(["--manifest", "/nonexistent.json", "--ledger", "l", "--policy", "branching",
                                           "--to-worker", "3", "--from-worker", "4"], stderr=Err()), 5)
        # A lost worker before the first answer is a technical abort, exit 4,
        # recorded in the ledger; nothing normative is concluded.
        worker, _ = recorded()
        tmp = scratch("entry-")
        m = write_manifest(tmp, worker, "C")
        to_r, to_w = os.pipe()
        from_r, from_w = os.pipe()
        os.close(to_r)      # nobody will read the controller's requests
        os.close(from_w)    # and nobody will answer
        code = run_episode.main(["--manifest", str(m.source_path), "--ledger", str(Path(tmp) / "ledger"),
                                 "--policy", "branching", "--to-worker", str(to_w), "--from-worker", str(from_r)],
                                stderr=Err())
        self.assertEqual(code, 4)
        progress = ledger.Ledger(Path(tmp) / "ledger", m.episode_hash).read_file("progress")
        self.assertTrue(any(p["kind"] == "episode.aborted" and "worker lost" in p["message"] for p in progress))
        self.assertTrue(any("technical_abort" in s for s in err))


if __name__ == "__main__":
    unittest.main()
