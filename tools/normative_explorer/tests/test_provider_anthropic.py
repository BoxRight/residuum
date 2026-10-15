"""Anthropic Messages adapter with an injected fake client, no network:
strict non-recursive schemas that real calls satisfy end to end and that
dispatch to the same envelopes as the other adapter; the request body and
its parameter allow-list; strictly append-only replay with thinking blocks
unchanged; parsing of stop reasons; and the controller's accounting of
rejected answers. Run individually through ``scripts/test_safe.py
--python-test``."""
import json
import unittest
from pathlib import Path

from tools.normative_explorer import controller, dispatcher, ledger, provider_anthropic, provider_openai, schemas
from tools.normative_explorer.backend import StubBackend
from tools.normative_explorer.provider_base import ProviderError
from tools.normative_explorer.tests import jsonschema_subset
from tools.normative_explorer.tests.test_controller import stub_response
from tools.normative_explorer.tests.test_provider_openai import SAMPLES, completed, call, manifest_on_disk as _openai_manifest

import tempfile

ROOT = Path(__file__).resolve().parents[3]


def manifest_on_disk(condition, parameters=None, model_id="claude-opus-5-5"):
    from tools.normative_explorer import manifest
    from tools.normative_explorer.tests.test_ledger import document
    (ROOT / "target" / "exploration").mkdir(parents=True, exist_ok=True)
    tmp = tempfile.mkdtemp(prefix="anthropic-", dir=str(ROOT / "target" / "exploration"))
    doc = document(condition)
    doc["controller"]["model"] = {"provider": "anthropic", "model_id": model_id, "parameters": parameters or {}}
    path = Path(tmp) / "episode.json"
    path.write_text(json.dumps(doc))
    return manifest.load(path), Path(tmp)


class FakeMessages:
    def __init__(self, replies, on_create=None):
        self.replies = list(replies)
        self.bodies = []
        self.on_create = on_create

    def create(self, **body):
        if self.on_create:
            self.on_create(body)
        self.bodies.append(json.loads(json.dumps(body)))
        reply = self.replies.pop(0)
        if isinstance(reply, Exception):
            raise reply
        return reply


class FakeClient:
    def __init__(self, replies, on_create=None):
        self.messages = FakeMessages(replies, on_create)


def message(content, stop_reason="tool_use", output_tokens=30, input_tokens=1000, cache_read=0, stop_details=None):
    return {"id": "msg_1", "type": "message", "role": "assistant", "model": "claude-opus-5-5",
            "content": content, "stop_reason": stop_reason, "stop_details": stop_details,
            "usage": {"input_tokens": input_tokens, "output_tokens": output_tokens,
                      "cache_creation_input_tokens": 0, "cache_read_input_tokens": cache_read}}


def thinking(signature="sig-1"):
    return {"type": "thinking", "thinking": "", "signature": signature}


def tool_use(name, args, block_id="toolu_1"):
    return {"type": "tool_use", "id": block_id, "name": name, "input": args, "caller": None}


def refs(node, out):
    if isinstance(node, dict):
        for k, v in node.items():
            if k == "$ref":
                out.append(v.split("/")[-1])
            refs(v, out)
    elif isinstance(node, list):
        for v in node:
            refs(v, out)
    return out


def keys(node, out):
    if isinstance(node, dict):
        out.update(node)
        for v in node.values():
            keys(v, out)
    elif isinstance(node, list):
        for v in node:
            keys(v, out)
    return out


class SchemaTests(unittest.TestCase):
    def test_strict_schemas_are_acyclic_and_carry_no_unsupported_constraints(self):
        order = list(provider_anthropic.DEFS)
        for name, definition in provider_anthropic.DEFS.items():
            for target in refs(definition, []):
                self.assertLess(order.index(target), order.index(name), f"{name} -> {target} is not acyclic")
        for condition in "ABC":
            tools = provider_anthropic.tool_definitions(condition)
            self.assertEqual([t["name"] for t in tools], schemas.catalog(condition))
            for t in tools:
                self.assertTrue(t["strict"])
                s = t["input_schema"]
                self.assertFalse(s["additionalProperties"])
                self.assertEqual(set(s["required"]), set(s["properties"]))
                self.assertFalse(keys(s, set()) & {"minimum", "maximum", "maxLength", "minLength", "maxItems"})

    def test_conforming_calls_dispatch_to_the_same_envelopes_as_the_other_adapter(self):
        m, _ = manifest_on_disk("C")
        mo, _ = _openai_manifest("C")
        tools = {t["name"]: t for t in provider_anthropic.tool_definitions("C")}
        self.assertEqual(set(tools), set(SAMPLES))

        def registry():
            r = dispatcher.Registry()
            r.sessions["s1"] = {"kind": "forward"}
            r.candidates.update({"c1": {}, "c2": {}})
            r.admitted.update({"c1", "c2"})
            return r
        da, do = dispatcher.Dispatcher(m, registry()), dispatcher.Dispatcher(mo, registry())
        for i, (name, args) in enumerate(SAMPLES.items()):
            s = tools[name]["input_schema"]
            jsonschema_subset.validate(args, s, s["$defs"], name)
            _, _, action = provider_anthropic.AnthropicMessagesProvider.parse_response(
                message([tool_use(name, args)]))
            validated = schemas.validate_action(action, "C")
            _, _, other = provider_openai.OpenAIResponsesProvider.parse_response(completed([call(name, args)]))
            other_validated = schemas.validate_action(other, "C")
            if validated.worker_side:
                env = da.envelope(f"a{i + 1}", validated)
                env_o = do.envelope(f"a{i + 1}", other_validated)
                self.assertEqual(env["request"], env_o["request"], name)
        # Depth beyond what P1 admits is outside the schema.
        deep = dict(SAMPLES["minimal_supports"])
        deep["goal"] = dict(deep["goal"], time={"after": {"after": {"after": {"at": "t"}}}})
        s = tools["minimal_supports"]["input_schema"]
        with self.assertRaises(AssertionError):
            jsonschema_subset.validate(deep, s, s["$defs"])


class AdapterTests(unittest.TestCase):
    def test_request_body_and_parameter_allow_list(self):
        for bad in ({"temperature": 0}, {"model": "x"}, {"system": "x"}, {"tools": []}, {"max_tokens": 1},
                    {"fallbacks": "default"}, {"thinking": {"type": "disabled"}},
                    {"thinking": {"type": "enabled", "budget_tokens": 2000}},
                    {"output_config": {"effort": "huge"}}, {"output_config": {"effort": "low", "format": {}}}):
            m, _ = manifest_on_disk("B", bad)
            with self.assertRaises(ProviderError, msg=str(bad)):
                provider_anthropic.AnthropicMessagesProvider(FakeClient([]), m)
        m, _ = manifest_on_disk("B", {"output_config": {"effort": "high"},
                                      "thinking": {"type": "adaptive", "display": "summarized"}})
        client = FakeClient([message([thinking(), tool_use("inspect_theory", SAMPLES["inspect_theory"])])])
        p = provider_anthropic.AnthropicMessagesProvider(client, m)
        action, usage = p.next_action([], schemas.catalog("B"), 100000)
        self.assertEqual(action["tool"], "inspect_theory")
        body = client.messages.bodies[0]
        self.assertEqual(body["model"], "claude-opus-5-5")
        self.assertEqual(body["max_tokens"], provider_anthropic.MAX_TOKENS_PER_REQUEST)
        self.assertEqual(body["tool_choice"], {"type": "auto", "disable_parallel_tool_use": True})
        self.assertEqual(body["output_config"], {"effort": "high"})
        self.assertEqual(body["cache_control"], {"type": "ephemeral"})
        self.assertIn("exactly one tool call", body["system"])
        self.assertNotIn("anthropic-workspace-id", json.dumps(body))
        self.assertNotIn("fallbacks", body)
        self.assertEqual(usage, {"output_tokens": 30, "input_tokens": 1000, "cache_read_input_tokens": 0})
        with self.assertRaises(ProviderError):
            provider_anthropic.AnthropicMessagesProvider(FakeClient([]), _openai_manifest("B")[0])

    def test_history_is_append_only_with_thinking_blocks_unchanged(self):
        m, _ = manifest_on_disk("A")
        first = [thinking("sig-A"), {"type": "text", "text": "Looking first.", "citations": None},
                 tool_use("inspect_theory", SAMPLES["inspect_theory"], "toolu_A")]
        second = [thinking("sig-B"), tool_use("begin_forward", SAMPLES["begin_forward"], "toolu_B")]
        client = FakeClient([message(first), message(second),
                             message([tool_use("finish_episode", SAMPLES["finish_episode"], "toolu_C")])])
        p = provider_anthropic.AnthropicMessagesProvider(client, m)
        p.vocabulary = {"verbs": []}
        p.next_action([], schemas.catalog("A"), 500)
        h1 = [{"action_id": "a1", "tool": "inspect_theory", "arguments": {}, "view": {"response": {"status": "complete"}}}]
        p.next_action(h1, schemas.catalog("A"), 500)
        h2 = h1 + [{"action_id": None, "tool": None, "arguments": None, "view": {"protocol_error": "unknown dossier"}}]
        p.vocabulary = {"verbs": ["changed later"]}   # must not alter the frozen opening
        p.next_action(h2, schemas.catalog("A"), 500)
        b1, b2, b3 = client.messages.bodies
        for earlier, later in ((b1, b2), (b2, b3)):
            self.assertEqual(later["messages"][:len(earlier["messages"])], earlier["messages"], "append-only")
            self.assertEqual(later["system"], earlier["system"])
            self.assertEqual(later["tools"], earlier["tools"])
        replayed = b2["messages"][1]["content"]
        self.assertEqual(replayed[0], {"type": "thinking", "thinking": "", "signature": "sig-A"})
        self.assertEqual(replayed[1], {"type": "text", "text": "Looking first."})
        self.assertNotIn("caller", replayed[2])
        self.assertIsNone(replayed[2]["input"]["expected_observation"], "nulls inside a tool input are kept")
        self.assertEqual(b2["messages"][2]["content"][0]["tool_use_id"], "toolu_A")
        error_result = b3["messages"][4]["content"][0]
        self.assertEqual(error_result["tool_use_id"], "toolu_B")
        self.assertTrue(error_result["is_error"])
        self.assertEqual(json.loads(b1["messages"][0]["content"])["vocabulary"], {"verbs": []})

    def test_stop_reasons_and_tool_call_count(self):
        parse = provider_anthropic.AnthropicMessagesProvider.parse_response
        cases = [
            (message([{"type": "text", "text": "I would inspect"}], stop_reason="end_turn"), "exactly one"),
            (message([tool_use("a", {}), tool_use("b", {}, "t2")]), "exactly one"),
            (message([tool_use("a", {})], stop_reason="max_tokens"), "model_incomplete"),
            (message([], stop_reason="refusal", stop_details={"type": "refusal", "category": "cyber"}), "refusal"),
        ]
        for reply, pattern in cases:
            with self.assertRaisesRegex(ProviderError, pattern):
                parse(reply)
        try:
            parse(message([], stop_reason="refusal", output_tokens=4, input_tokens=50, cache_read=10,
                          stop_details={"category": "bio"}))
        except ProviderError as error:
            self.assertIn("'bio'", str(error))
            self.assertEqual(error.usage, {"output_tokens": 4, "input_tokens": 60, "cache_read_input_tokens": 10})

    def test_controller_records_and_charges_rejected_answers_without_retry(self):
        m, tmp = manifest_on_disk("A")
        led = ledger.Ledger(tmp / "ledger", m.episode_hash, clock=lambda: "t")
        bodies_seen = []

        def on_create(body):
            bodies_seen.append(sum(1 for r in led.read_file("model-requests") if r["kind"] == "model_request_body"))
        client = FakeClient([
            message([{"type": "text", "text": "no tool"}], stop_reason="end_turn", output_tokens=11, input_tokens=100),
            RuntimeError("overloaded"),
            message([], stop_reason="refusal", output_tokens=2, input_tokens=120, stop_details={"category": None}),
            message([tool_use("inspect_theory", SAMPLES["inspect_theory"])], output_tokens=9, input_tokens=130),
            message([tool_use("finish_episode", SAMPLES["finish_episode"], "toolu_9")], output_tokens=5, input_tokens=140),
        ], on_create)
        p = provider_anthropic.AnthropicMessagesProvider(client, m)
        be = StubBackend([lambda e: stub_response(e["action_id"], "complete", {"theory": {
            "module": "B", "rules": [], "seeded_verbs": [], "derived_verbs": [], "effect_verbs": [], "dossiers": ["d1"]}})])
        ctl = controller.Controller(m, led, be, p)
        self.assertEqual(ctl.run(), "no_verified_candidate")
        self.assertEqual(bodies_seen, [1, 2, 3, 4, 5], "each body is in the ledger before the call")
        kinds = [r["kind"] for r in led.read_file("model-responses")]
        self.assertEqual(kinds.count("model_error"), 3)
        snap = ctl.budgets.snapshot()
        self.assertEqual(snap["output_tokens"][0], 11 + 2 + 9 + 5)
        self.assertEqual(snap["input_tokens"], 100 + 120 + 130 + 140)
        self.assertEqual(snap["usage_unknown"], 1)
        self.assertEqual(len(client.messages.bodies), 5, "no hidden retry")
        # Rejected answers are not stored: the requests after them repeat the same prefix.
        b = client.messages.bodies
        self.assertEqual(b[0]["messages"], b[1]["messages"])
        self.assertEqual(b[1]["messages"], b[2]["messages"])
        self.assertEqual(b[2]["messages"], b[3]["messages"])
        self.assertEqual(len(b[4]["messages"]), 3)


if __name__ == "__main__":
    unittest.main()
