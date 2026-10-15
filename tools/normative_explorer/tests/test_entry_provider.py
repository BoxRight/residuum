"""The supervised entry point with a real provider selected by the manifest,
over real pipe descriptors, with an injected fake Anthropic client and a
stateful fake worker on a thread: the same path a pilot takes, offline.
Run individually through ``scripts/test_safe.py --python-test``."""
import json
import os
import tempfile
import threading
import unittest
from pathlib import Path

from tools.normative_explorer import ledger, manifest, run_episode
from tools.normative_explorer.provider_anthropic import AnthropicMessagesProvider
from tools.normative_explorer.provider_base import ProviderError
from tools.normative_explorer.tests.test_ledger import document
from tools.normative_explorer.tests.test_provider_anthropic import FakeClient, message, thinking, tool_use

ROOT = Path(__file__).resolve().parents[3]
ARGS = {"rationale": "r", "expected_observation": None, "claims": None, "legal_hypothesis": None}


def episode_manifest(provider="anthropic", client=True):
    (ROOT / "target" / "exploration").mkdir(parents=True, exist_ok=True)
    tmp = Path(tempfile.mkdtemp(prefix="entry-", dir=str(ROOT / "target" / "exploration")))
    doc = document("B")
    doc["controller"]["model"] = {"provider": provider, "model_id": "claude-opus-5-5",
                                  "parameters": {"output_config": {"effort": "medium"}} if provider == "anthropic" else {}}
    if client:
        doc["controller"]["model"]["client"] = {"timeout_seconds": 45}
    path = tmp / "episode.json"
    path.write_text(json.dumps(doc))
    return path, tmp


class FakeWorker(threading.Thread):
    """Answers envelopes with well-formed lines and the worker's semantics
    the controller relies on: sessions, rounds, budgets, idempotent replay."""

    ROUNDS_TO_FIXED_POINT = 4

    def __init__(self, inbound_fd, outbound_fd):
        super().__init__(daemon=True)
        self.inbound = os.fdopen(inbound_fd, "rb", buffering=0)
        self.outbound = os.fdopen(outbound_fd, "wb", buffering=0)
        self.sessions = {}
        self.done = {}
        self.actions_left = 50
        self.steps_left = 300
        self.received = []

    def response(self, action_id, status, payload, steps=0):
        return {"response": {"wire_version": "residuum-wire/1", "action_id": action_id, "status": status,
                             "steps_consumed": steps, "payload": payload,
                             "budget": {"actions_remaining": self.actions_left, "steps_remaining": self.steps_left,
                                        "sessions_remaining": 9, "candidates_remaining": 9}}}

    def answer(self, env):
        key = json.dumps(env["request"], sort_keys=True)
        if env["action_id"] in self.done and self.done[env["action_id"]][0] == key:
            return self.done[env["action_id"]][1]
        self.actions_left -= 1
        request = env["request"]
        if request == "inspect_theory":
            line = self.response(env["action_id"], "complete", {"theory": {
                "module": "Baseline", "rules": [], "seeded_verbs": ["Act"], "derived_verbs": [], "effect_verbs": [],
                "dossiers": ["d1"], "verbs": [{"name": "Act", "kind": "seeded", "args": []}], "entities": [],
                "constants": [], "rule_bodies": []}})
        elif "begin_forward" in request:
            sid = f"s{len(self.sessions) + 1}"
            self.sessions[sid] = 0
            line = self.response(env["action_id"], "complete", {"session_opened": {"session": sid, "kind": "forward"}})
        elif "advance_session" in request:
            sid, k = request["advance_session"]["session"], request["advance_session"]["max_steps"]
            start = self.sessions[sid]
            end = min(start + k, self.ROUNDS_TO_FIXED_POINT)
            self.sessions[sid] = end
            self.steps_left -= end - start
            complete = end == self.ROUNDS_TO_FIXED_POINT
            rounds = [{"round": r, "added": [], "derivations": [], "witnesses": 0, "complete": r == end and complete}
                      for r in range(start + 1, end + 1)]
            line = self.response(env["action_id"], "complete" if complete else "step_limit",
                                 {"forward": {"session": sid, "rounds": rounds, "known": [], "complete": complete}},
                                 end - start)
        else:
            line = self.response(env["action_id"], "error", {"error": {"message": "not in the fake"}})
        self.done[env["action_id"]] = (key, line)
        return line

    def run(self):
        buffer = b""
        while True:
            chunk = self.inbound.read(65536)
            if not chunk:
                break
            buffer += chunk
            while b"\n" in buffer:
                raw, buffer = buffer.split(b"\n", 1)
                env = json.loads(raw)
                self.received.append(env)
                self.outbound.write((json.dumps(self.answer(env)) + "\n").encode())
        self.outbound.close()


class Err:
    def __init__(self):
        self.chunks = []

    def write(self, s):
        self.chunks.append(s)

    def flush(self):
        pass

    @property
    def lines(self):
        return [l for l in "".join(self.chunks).splitlines() if l]


def run_with_fakes(path, tmp, replies, policy=None):
    to_r, to_w = os.pipe()
    from_r, from_w = os.pipe()
    worker = FakeWorker(to_r, from_w)
    worker.start()
    built = {}

    def factory(timeout_seconds):
        built["timeout"] = timeout_seconds
        built["client"] = FakeClient(replies)
        return built["client"]
    argv = ["--manifest", str(path), "--ledger", str(tmp / "ledger"), "--to-worker", str(to_w),
            "--from-worker", str(from_r)] + (["--policy", policy] if policy else [])
    err = Err()
    code = run_episode.main(argv, stderr=err, client_factories={"anthropic": factory})
    worker.join(5)
    return code, err, built, worker


class EntryPointWithRealProviderTests(unittest.TestCase):
    def test_anthropic_provider_runs_the_full_path_offline(self):
        path, tmp = episode_manifest()
        replies = [
            message([thinking("sig-1"), tool_use("begin_forward", dict(ARGS, dossier="d1", theory=None), "t1")]),
            message([thinking("sig-2"), tool_use("advance_session", dict(ARGS, session="s1", max_steps=8), "t2")]),
            message([tool_use("finish_episode", dict(ARGS, candidate_ids=[], explanation="observed"), "t3")]),
        ]
        code, err, built, worker = run_with_fakes(path, tmp, replies)
        summary = json.loads(err.lines[-1].split("controller: ", 1)[1])
        self.assertEqual(code, 0, summary)
        self.assertEqual(built["timeout"], 45, "deadline from the frozen manifest")
        self.assertEqual(summary["provider"], "anthropic")
        self.assertEqual(summary["outcome"], "no_verified_candidate")
        self.assertTrue(summary["idempotence_probe"]["passed"], summary["idempotence_probe"])
        bodies = built["client"].messages.bodies
        self.assertEqual(len(bodies), 3)
        self.assertEqual(bodies[0]["output_config"], {"effort": "medium"})
        opening = json.loads(bodies[0]["messages"][0]["content"])
        self.assertEqual(opening["vocabulary"]["verbs"][0]["name"], "Act", "vocabulary frozen before the first call")
        for earlier, later in zip(bodies, bodies[1:]):
            self.assertEqual(later["messages"][:len(earlier["messages"])], earlier["messages"])
        self.assertEqual(bodies[2]["messages"][3]["content"][0]["signature"], "sig-2")
        # The worker saw: vocabulary, two model actions, then the probe.
        tools_seen = [e["request"] if isinstance(e["request"], str) else next(iter(e["request"])) for e in worker.received]
        self.assertEqual(tools_seen[:3], ["inspect_theory", "begin_forward", "advance_session"])
        led = ledger.Ledger(tmp / "ledger", manifest.load(path).episode_hash)
        rebuilt = led.reconstruct()
        self.assertEqual(rebuilt["terminal"], "episode.finished")
        kinds = [r["kind"] for r in led.read_file("model-requests")]
        self.assertEqual(kinds.count("model_request_body"), 3)
        raw = [r for r in led.read_file("model-responses") if r["kind"] == "model_response_raw"]
        self.assertEqual(len(raw), 3)

    def test_provider_selection_is_explicit_and_comes_from_the_manifest(self):
        path, tmp = episode_manifest()
        code, err, built, _ = run_with_fakes(path, tmp, [], policy="branching")
        self.assertEqual(code, 2)
        self.assertIn("--policy is only for the simulated provider", "".join(err.lines))
        self.assertNotIn("client", built, "no client is built when the selection is refused")
        sim, tmp2 = episode_manifest(provider="simulated", client=False)
        code, err, _, _ = run_with_fakes(sim, tmp2, [])
        self.assertEqual(code, 2)
        self.assertIn("needs --policy", "".join(err.lines))
        oa, tmp3 = episode_manifest(provider="openai")
        code, err, _, _ = run_with_fakes(oa, tmp3, [])
        self.assertEqual(code, 2)
        self.assertIn("no live client for provider 'openai'", "".join(err.lines))
        with self.assertRaises(manifest.ManifestError):
            manifest.load(episode_manifest(provider="anthropic", client=False)[0])
        with self.assertRaises(manifest.ManifestError):
            manifest.load(episode_manifest(provider="simulated", client=True)[0])
        doc = json.loads(path.read_text())
        doc["controller"]["model"]["client"]["timeout_seconds"] = 0
        path.write_text(json.dumps(doc))
        with self.assertRaises(manifest.ManifestError):
            manifest.load(path)

    def test_a_failing_provider_is_a_recorded_model_error_not_a_crash(self):
        path, tmp = episode_manifest()
        replies = [RuntimeError("timed out")] * 3 + \
                  [message([tool_use("finish_episode", dict(ARGS, candidate_ids=[], explanation="x"), "t9")])]
        code, err, built, _ = run_with_fakes(path, tmp, replies)
        summary = json.loads(err.lines[-1].split("controller: ", 1)[1])
        self.assertEqual(code, 0)
        self.assertEqual(summary["budgets"]["usage_unknown"], 3)
        led = ledger.Ledger(tmp / "ledger", manifest.load(path).episode_hash)
        errors = [r for r in led.read_file("model-responses") if r["kind"] == "model_error"]
        self.assertEqual(len(errors), 3)
        self.assertTrue(all("timed out" in e["message"] for e in errors))


class BuildProviderTests(unittest.TestCase):
    def test_build_provider_never_builds_an_unrequested_client(self):
        path, _ = episode_manifest()
        m = manifest.load(path)
        calls = []
        provider, policy = run_episode.build_provider(m, None, {"anthropic": lambda t: calls.append(t) or FakeClient([])})
        self.assertIsInstance(provider, AnthropicMessagesProvider)
        self.assertIsNone(policy)
        self.assertEqual(calls, [45])
        with self.assertRaises(ProviderError):
            run_episode.build_provider(m, None, {})


if __name__ == "__main__":
    unittest.main()
