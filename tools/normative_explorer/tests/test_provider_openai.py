"""Real-provider adapter with an injected fake client: request body per the
protocol, strict tool schemas from the catalogue, conversation replay by
call id, exactly-one-call parsing, and model errors recorded by the
controller without retry. No network. Run individually through
``scripts/test_safe.py --python-test``."""
import json
import unittest
from pathlib import Path

from tools.normative_explorer import controller, ledger, manifest, provider_openai, provider_sim, schemas
from tools.normative_explorer.tests.test_ledger import document


class FakeResponses:
    def __init__(self, replies):
        self.replies = list(replies)
        self.bodies = []

    def create(self, **body):
        self.bodies.append(body)
        reply = self.replies.pop(0)
        if isinstance(reply, Exception):
            raise reply
        return reply


class FakeClient:
    def __init__(self, replies):
        self.responses = FakeResponses(replies)


def completed(calls, output_tokens=17, status="completed"):
    return {"id": "resp_1", "status": status,
            "output": [{"type": "function_call", "call_id": f"call_{i}", "name": name, "arguments": json.dumps(args)}
                       for i, (name, args) in enumerate(calls)],
            "usage": {"output_tokens": output_tokens}}


def openai_manifest(condition="B"):
    doc = document(condition)
    doc["controller"]["model"] = {"provider": "openai", "model_id": "fixed-model-2026-10", "parameters": {"temperature": 0}}
    return manifest.validate(doc, Path("m.json"))


class SchemaTests(unittest.TestCase):
    def test_tools_are_strict_and_follow_the_catalogue(self):
        for condition in "ABC":
            tools = provider_openai.tool_definitions(condition)
            self.assertEqual([t["name"] for t in tools], schemas.catalog(condition))
            for t in tools:
                self.assertTrue(t["strict"])
                p = t["parameters"]
                self.assertFalse(p["additionalProperties"])
                self.assertEqual(set(p["required"]), set(p["properties"]))
                self.assertIn("rationale", p["properties"])
                self.assertIn("claims", p["properties"])
                self.assertIn("legal_hypothesis", p["properties"])
                self.assertTrue(self._strict(p))

    def _strict(self, node):
        if isinstance(node, dict):
            if node.get("type") == "object" and "properties" in node:
                if node.get("additionalProperties") is not False or set(node.get("required", [])) != set(node["properties"]):
                    return False
            return all(self._strict(v) for v in node.values())
        if isinstance(node, list):
            return all(self._strict(v) for v in node)
        return True


class AdapterTests(unittest.TestCase):
    def test_request_body_and_conversation_replay(self):
        m = openai_manifest("B")
        client = FakeClient([
            completed([("inspect_theory", {"rationale": "look", "expected_observation": None, "claims": [], "legal_hypothesis": None})]),
            completed([("finish_episode", {"candidate_ids": [], "explanation": "done", "rationale": "x",
                                           "expected_observation": None, "claims": [], "legal_hypothesis": None})], output_tokens=5),
        ])
        provider = provider_openai.OpenAIResponsesProvider(client, m)
        action, usage = provider.next_action([], schemas.catalog("B"), 333)
        self.assertEqual(action["tool"], "inspect_theory")
        self.assertEqual(usage, {"output_tokens": 17})
        body = client.responses.bodies[0]
        self.assertEqual(body["model"], "fixed-model-2026-10")
        self.assertEqual(body["tool_choice"], "required")
        self.assertFalse(body["parallel_tool_calls"])
        self.assertFalse(body["store"])
        self.assertFalse(body["stream"])
        self.assertEqual(body["max_output_tokens"], 333)
        self.assertEqual(body["temperature"], 0)
        self.assertEqual([t["name"] for t in body["tools"]], schemas.catalog("B"))
        self.assertEqual(body["input"][0]["role"], "user")
        facts = json.loads(body["input"][0]["content"])
        self.assertEqual(facts["condition"], "B")
        self.assertEqual(facts["tools"], schemas.catalog("B"))
        self.assertNotIn("api_key", json.dumps(body))
        self.assertNotIn("Authorization", json.dumps(body))
        # Second turn: the first call and the result the controller delivered are replayed.
        history = [{"action_id": "a1", "tool": "inspect_theory", "arguments": {}, "view": {"response": {"status": "complete"}}}]
        provider.next_action(history, schemas.catalog("B"), 100)
        items = client.responses.bodies[1]["input"]
        self.assertEqual(items[1]["type"], "function_call")
        self.assertEqual(items[1]["call_id"], "call_0")
        self.assertEqual(items[1]["name"], "inspect_theory")
        self.assertEqual(items[2], {"type": "function_call_output", "call_id": "call_0",
                                    "output": json.dumps({"response": {"status": "complete"}}, sort_keys=True)})
        # Desynchronised history and foreign catalogue are refused before any call.
        with self.assertRaises(provider_openai.ProviderError):
            provider.build_request([], schemas.catalog("B"), 10)
        with self.assertRaises(provider_openai.ProviderError):
            provider.build_request(history + history, schemas.catalog("C"), 10)

    def test_parsing_rejects_everything_but_one_call(self):
        parse = provider_openai.OpenAIResponsesProvider.parse_response
        with self.assertRaisesRegex(provider_openai.ProviderError, "exactly one"):
            parse(completed([]))
        with self.assertRaisesRegex(provider_openai.ProviderError, "exactly one"):
            parse(completed([("inspect_theory", {}), ("inspect_theory", {})]))
        with self.assertRaisesRegex(provider_openai.ProviderError, "model_incomplete"):
            parse(completed([("inspect_theory", {})], status="incomplete"))
        bad = completed([("inspect_theory", {})])
        bad["output"][0]["arguments"] = "{not json"
        with self.assertRaisesRegex(provider_openai.ProviderError, "not JSON"):
            parse(bad)
        text_only = {"status": "completed", "output": [{"type": "message", "content": "please run it"}]}
        with self.assertRaises(provider_openai.ProviderError):
            parse(text_only)
        no_usage = completed([("inspect_theory", {"a": 1})])
        del no_usage["usage"]
        call_id, action, _, usage = parse(no_usage)
        self.assertEqual((call_id, action["tool"], usage), ("call_0", "inspect_theory", None))
        with self.assertRaises(provider_openai.ProviderError):
            provider_openai.OpenAIResponsesProvider(FakeClient([]), manifest.validate(document("A"), Path("m")))

    def test_controller_records_model_errors_without_retry(self):
        import tempfile
        tmp = tempfile.mkdtemp(prefix="openai-", dir=str(Path(__file__).resolve().parents[3] / "target" / "exploration"))
        doc = document("A")
        doc["controller"]["model"] = {"provider": "openai", "model_id": "fixed-model-2026-10", "parameters": {}}
        path = Path(tmp) / "episode.json"
        path.write_text(json.dumps(doc))
        m = manifest.load(path)
        client = FakeClient([
            RuntimeError("connection reset"),
            completed([("inspect_theory", {"rationale": "x"})]),  # arguments incomplete for strict schema: still parsed, validated by the controller
            completed([("finish_episode", {"candidate_ids": [], "explanation": "done", "rationale": "x",
                                           "expected_observation": None})]),
        ])
        provider = provider_openai.OpenAIResponsesProvider(client, m)
        if True:
            led = ledger.Ledger(Path(tmp) / "ledger", m.episode_hash, clock=lambda: "t")
            from tools.normative_explorer.backend import StubBackend
            from tools.normative_explorer.tests.test_controller import stub_response
            be = StubBackend([lambda e: stub_response(e["action_id"], "complete", {"theory": {
                "module": "B", "rules": [], "seeded_verbs": [], "derived_verbs": [], "effect_verbs": [], "dossiers": ["d1"]}})])
            ctl = controller.Controller(m, led, be, provider)
            outcome = ctl.run()
            self.assertEqual(outcome, "no_verified_candidate")
            responses = led.read_file("model-responses")
            self.assertEqual([r["kind"] for r in responses], ["model_error", "model_response", "model_response"])
            self.assertIn("connection reset", responses[0]["message"])
            self.assertEqual(ctl.budgets.model_requests, 3, "the failed request stays charged")
            self.assertEqual(len(client.responses.bodies), 3, "no hidden retry")
            # The inspect call was validated by the controller (missing expected_observation is required by the schema).
            self.assertTrue(any(a["kind"] == "protocol_error" for a in led.read_file("actions")))


if __name__ == "__main__":
    unittest.main()
