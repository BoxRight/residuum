"""Real-provider adapter with an injected fake client, no network: request
body, parameter allow-list, strict schemas that real tool calls satisfy end
to end (schema → local validation → dispatcher → envelope written for the
Rust decoder), replay of every output item including opaque continuation
items, persistence of the exact body before the network and the raw answer
before parsing, and usage charged even when the answer is rejected. Run
individually through ``scripts/test_safe.py --python-test``."""
import json
import tempfile
import unittest
from pathlib import Path

from tools.normative_explorer import controller, dispatcher, ledger, manifest, provider_openai, schemas
from tools.normative_explorer.backend import StubBackend
from tools.normative_explorer.tests import jsonschema_subset
from tools.normative_explorer.tests.test_controller import stub_response
from tools.normative_explorer.tests.test_ledger import document

ROOT = Path(__file__).resolve().parents[3]
ROUNDTRIP = ROOT / "target" / "exploration" / "schema-roundtrip"


class FakeResponses:
    def __init__(self, replies, on_create=None):
        self.replies = list(replies)
        self.bodies = []
        self.on_create = on_create

    def create(self, **body):
        if self.on_create:
            self.on_create(body)
        self.bodies.append(body)
        reply = self.replies.pop(0)
        if isinstance(reply, Exception):
            raise reply
        return reply


class FakeClient:
    def __init__(self, replies, on_create=None):
        self.responses = FakeResponses(replies, on_create)


def call(name, args, call_id="call_0"):
    return {"type": "function_call", "call_id": call_id, "name": name, "arguments": json.dumps(args)}


def completed(items, output_tokens=17, input_tokens=900, status="completed"):
    return {"id": "resp_1", "status": status, "output": items,
            "usage": {"output_tokens": output_tokens, "input_tokens": input_tokens}}


def full_args(**fields):
    base = {"rationale": "r", "expected_observation": None, "claims": None, "legal_hypothesis": None}
    base.update(fields)
    return base


def manifest_on_disk(condition, parameters=None):
    ROUNDTRIP.parent.mkdir(parents=True, exist_ok=True)
    tmp = tempfile.mkdtemp(prefix="openai-", dir=str(ROUNDTRIP.parent))
    doc = document(condition)
    doc["controller"]["model"] = {"provider": "openai", "model_id": "fixed-model-2026-10",
                                  "parameters": parameters or {}}
    path = Path(tmp) / "episode.json"
    path.write_text(json.dumps(doc))
    return manifest.load(path), Path(tmp)


EVENT = {"polarity": "positive", "verb": "testamentaryEntitlement",
         "args": [{"const": "palmerWill"}, {"var": "testator"}, "none",
                  {"after": {"var": "t"}},
                  {"record": {"entity": "Holding", "fields": [{"field": "owner", "value": {"const": "elmerPalmer"}}]}}],
         "kind": "derived", "time": {"after": {"at": "dossierTime"}}, "transition": None}

SAMPLES = {
    "inspect_theory": full_args(),
    "begin_forward": full_args(dossier="d1", theory=None),
    "begin_residual": full_args(
        dossier="d1", theory=None, goal=EVENT, scope={"rules": ["literalTestamentaryEntitlement"]},
        bindings=[{"scope": {"rule": "literalTestamentaryEntitlement"}, "parameter": "death",
                   "ty": {"entity": "Fact"}, "value": {"term": {"const": "palmerDeath"}}},
                  {"scope": {"program": "p1"}, "parameter": "when",
                   "ty": {"optional": {"set": "proposition_time"}}, "value": {"time": {"at": "death1882"}}}],
        allowed=[{"verb": "Act", "polarity": "evidential_not"}],
        limits={"max_depth": 3, "max_nodes": 10, "max_branches": 11}),
    "begin_active_residual": full_args(dossier="d1", theory=None, goal=EVENT, scope=None, bindings=None,
                                       allowed=None, limits=None),
    "advance_session": full_args(session="s1", max_steps=3,
                                 claims=[{"kind": "residuum_result", "text": "t",
                                          "assertion": {"ref": "a1", "attribute": "status", "value": "complete"}}]),
    "propose_candidate": full_args(source="rule r(x) = a(x) @ t <= b(x) @ t", parent=None, move=None,
                                   legal_hypothesis={"statement": "s", "grounds": ["g"], "status": "hypothesis"}),
    "begin_candidate_evaluation": full_args(candidate="c1", dossier="d1"),
    "request_valuation": full_args(case_spec={"dossier": "d1", "note": "n"}, expected_discrimination="x"),
    "minimal_supports": full_args(candidate="c1", goal=EVENT),
    "compare_partition": full_args(candidates=["c1", "c2"], goal=EVENT),
    "finish_episode": full_args(candidate_ids=[], explanation="done"),
}


class SchemaRoundTripTests(unittest.TestCase):
    def test_conforming_tool_calls_validate_dispatch_and_reach_the_wire(self):
        m, _ = manifest_on_disk("C")
        registry = dispatcher.Registry()
        registry.sessions["s1"] = {"kind": "forward"}
        registry.candidates.update({"c1": {}, "c2": {}})
        registry.admitted.update({"c1", "c2"})
        d = dispatcher.Dispatcher(m, registry)
        tools = {t["name"]: t for t in provider_openai.tool_definitions("C")}
        self.assertEqual(set(tools), set(SAMPLES))
        envelopes = []
        for i, (name, args) in enumerate(SAMPLES.items()):
            parameters = tools[name]["parameters"]
            jsonschema_subset.validate(args, parameters, parameters["$defs"], name)
            output, call_id, action = provider_openai.OpenAIResponsesProvider.parse_response(
                completed([call(name, args)]))
            validated = schemas.validate_action(action, "C")
            if validated.worker_side:
                envelopes.append(d.envelope(f"a{i + 1}", validated))
        # Fields the schema would reject.
        bad = dict(SAMPLES["begin_residual"], allowed=[{"verb": "Act", "polarity": "negative"}])
        with self.assertRaises(AssertionError):
            jsonschema_subset.validate(bad, tools["begin_residual"]["parameters"],
                                       tools["begin_residual"]["parameters"]["$defs"])
        # Written for the Rust decoder: boundary test schema_roundtrip_tests.
        ROUNDTRIP.mkdir(parents=True, exist_ok=True)
        (ROUNDTRIP / "envelopes.jsonl").write_text(
            "".join(json.dumps(e, ensure_ascii=False) + "\n" for e in envelopes))
        self.assertEqual(len(envelopes), 9)


class AdapterTests(unittest.TestCase):
    def test_parameters_are_allow_listed_and_never_override_the_contract(self):
        for bad in ({"store": True}, {"model": "other"}, {"tools": []}, {"max_output_tokens": 10**9},
                    {"input": []}, {"frequency_penalty": 1}):
            m, _ = manifest_on_disk("B", bad)
            with self.assertRaises(provider_openai.ProviderError, msg=str(bad)):
                provider_openai.OpenAIResponsesProvider(FakeClient([]), m)
        m, _ = manifest_on_disk("B", {"temperature": 0, "reasoning": {"effort": "low"}})
        p = provider_openai.OpenAIResponsesProvider(FakeClient([completed([call("inspect_theory", full_args())])]), m)
        p.next_action([], schemas.catalog("B"), 333)
        body = p.client.responses.bodies[0]
        self.assertEqual((body["model"], body["store"], body["max_output_tokens"]), ("fixed-model-2026-10", False, 333))
        self.assertEqual(body["reasoning"], {"effort": "low"})

    def test_every_output_item_is_replayed_including_opaque_ones(self):
        m, _ = manifest_on_disk("B")
        reasoning = {"type": "reasoning", "id": "rs_1", "encrypted_content": "opaque==", "summary": []}
        client = FakeClient([completed([reasoning, call("inspect_theory", full_args())]),
                             completed([call("finish_episode", full_args(candidate_ids=[], explanation="x"), "call_1")])])
        p = provider_openai.OpenAIResponsesProvider(client, m)
        p.vocabulary = {"verbs": [{"name": "v", "kind": "derived", "args": []}]}
        p.next_action([], schemas.catalog("B"), 100)
        history = [{"action_id": "a1", "tool": "inspect_theory", "arguments": {}, "view": {"response": {"status": "complete"}}}]
        p.next_action(history, schemas.catalog("B"), 100)
        items = client.responses.bodies[1]["input"]
        self.assertEqual(items[1], reasoning, "opaque item replayed verbatim, in order")
        self.assertEqual(items[2]["type"], "function_call")
        self.assertEqual(items[3]["type"], "function_call_output")
        facts = json.loads(items[0]["content"])
        self.assertEqual(facts["vocabulary"], p.vocabulary)
        self.assertIn("rule <name>", facts["syntax"])
        self.assertIn("d1", facts["dossier_seeds"])

    def test_vocabulary_package_is_identical_across_conditions(self):
        contexts = []
        for condition in "ABC":
            m, _ = manifest_on_disk(condition)
            ctx = json.loads(provider_openai.initial_context(m, {"verbs": []}))
            ctx.pop("condition"); ctx.pop("tools"); ctx.pop("episode_id")
            contexts.append(ctx)
        self.assertEqual(contexts[0], contexts[1])
        self.assertEqual(contexts[1], contexts[2])

    def test_body_persisted_before_network_raw_answer_before_parsing_usage_always_charged(self):
        m, tmp = manifest_on_disk("A")
        led = ledger.Ledger(tmp / "ledger", m.episode_hash, clock=lambda: "t")
        seen_before_create = []

        def on_create(body):
            seen_before_create.append([r["kind"] for r in led.read_file("model-requests")])
        truncated = completed([call("inspect_theory", full_args())], output_tokens=40, input_tokens=1000, status="incomplete")
        bad_json = completed([{"type": "function_call", "call_id": "c", "name": "inspect_theory", "arguments": "{nope"}],
                             output_tokens=7, input_tokens=1100)
        no_usage = {"status": "completed", "output": [], }
        client = FakeClient([truncated, bad_json, no_usage, RuntimeError("reset"),
                             completed([call("finish_episode", full_args(candidate_ids=[], explanation="x"))],
                                       output_tokens=5, input_tokens=1200)], on_create)
        p = provider_openai.OpenAIResponsesProvider(client, m)
        be = StubBackend([])
        ctl = controller.Controller(m, led, be, p)
        self.assertEqual(ctl.run(), "no_verified_candidate")
        self.assertTrue(all("model_request_body" in kinds for kinds in seen_before_create),
                        "each body is in the ledger before responses.create")
        responses = led.read_file("model-responses")
        kinds = [r["kind"] for r in responses]
        self.assertEqual(kinds.count("model_response_raw"), 4)
        self.assertEqual(kinds.count("model_transport_error"), 1)
        self.assertEqual(kinds.count("model_error"), 4)
        raw = [r for r in responses if r["kind"] == "model_response_raw"]
        self.assertEqual(json.loads(led.read_artifact(raw[0]["artifact"]))["status"], "incomplete")
        body0 = [r for r in led.read_file("model-requests") if r["kind"] == "model_request_body"][0]
        self.assertEqual(json.loads(led.read_artifact(body0["artifact"]))["model"], "fixed-model-2026-10")
        snap = ctl.budgets.snapshot()
        self.assertEqual(snap["output_tokens"][0], 40 + 7 + 5)
        self.assertEqual(snap["input_tokens"], 1000 + 1100 + 1200)
        self.assertEqual(snap["usage_unknown"], 2, "no usage reported and transport failure")
        self.assertEqual(ctl.budgets.model_requests, 5)
        self.assertEqual(len(client.responses.bodies), 5, "no hidden retry")

    def test_parsing_rejects_everything_but_one_call(self):
        parse = provider_openai.OpenAIResponsesProvider.parse_response
        for reply, pattern in [(completed([]), "exactly one"),
                               (completed([call("a", {}), call("b", {}, "c2")]), "exactly one"),
                               (completed([call("a", {})], status="incomplete"), "model_incomplete"),
                               ({"status": "completed", "output": [{"type": "message", "content": "x"}]}, "exactly one")]:
            with self.assertRaisesRegex(provider_openai.ProviderError, pattern):
                parse(reply)
        try:
            parse(completed([call("a", {})], status="incomplete", output_tokens=9))
        except provider_openai.ProviderError as error:
            self.assertEqual(error.usage, {"output_tokens": 9, "input_tokens": 900})
        with self.assertRaises(provider_openai.ProviderError):
            provider_openai.OpenAIResponsesProvider(FakeClient([]), manifest.validate(document("A"), Path("m")))

    def test_tools_are_strict_and_follow_the_catalogue(self):
        for condition in "ABC":
            tools = provider_openai.tool_definitions(condition)
            self.assertEqual([t["name"] for t in tools], schemas.catalog(condition))
            for t in tools:
                p = t["parameters"]
                self.assertTrue(t["strict"])
                self.assertFalse(p["additionalProperties"])
                self.assertEqual(set(p["required"]), set(p["properties"]))


if __name__ == "__main__":
    unittest.main()
