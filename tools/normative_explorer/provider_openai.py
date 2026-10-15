"""Adapter for the real model provider (invocation protocol §3, §4), with
the client injected and no network in this module. It builds the exact
request body the protocol fixes, keeps the client-managed conversation
(every answer of the model is replayed as a function call with its call id
and the result the controller delivered as its output), and parses exactly
one function call from the response. Anything else is a ProviderError that
the controller records as model_error; nothing is retried here.

The first real call still needs a decision outside this module: the exact
model, what is sent, and the budget. Credentials belong to the client's
local configuration and never appear in the body or the logs.
"""
from __future__ import annotations

import json

from .manifest import EpisodeManifest
from .schemas import (CLAIM_ATTRIBUTES, CLAIM_KINDS, CLAIM_TEXT_MAX, CLAIMS_MAX, COMMON, IDENT_MAX,
                      SOURCE_MAX, STEPS_MAX, TEXT_MAX, TOOLS, catalog)

ADAPTER_VERSION = "provider-openai/0.1"

# Fields the protocol fixes; a manifest parameter may never touch them.
CONTROLLED = {"model", "instructions", "input", "tools", "tool_choice", "parallel_tool_calls",
              "max_output_tokens", "stream", "store", "previous_response_id"}
# Additional parameters a manifest may set, and nothing else.
ALLOWED_PARAMETERS = {"temperature", "top_p", "seed", "reasoning"}

FORMAL_SYNTAX = """Proposals are Formal rule declarations only (no module, import, verb, const or
experiment). One rule:

  rule <name>(<param>, ...) =
      <atom> [& <atom>]...      (or I for an empty body)
      <= <head atom>

An atom is <verb>(<arg>, ...) @ <time>; a negative seeded atom is ~<verb>(...) @ <time>.
Arguments are parameters (variables) or declared constants; `none` fills an optional
position. Times are a parameter, a constant, or `after <param>`. Every parameter used in
the head must occur in the body. The head verb must be one of the profile's heads; body
verbs must be in the profile's vocabulary; at most the profile's atoms and binders.
Rules are evaluated by the worker; proposing one does not make it part of the theory."""


class ProviderError(RuntimeError):
    """The model's output or the transport is not usable; no action executes.
    ``usage`` carries what the provider reported for the attempt, so the
    cost is charged even when the answer is rejected."""

    def __init__(self, message: str, usage: dict | None = None):
        super().__init__(message)
        self.usage = usage


# -- strict JSON schemas derived from the tool catalogue ---------------------

def _string(max_length):
    return {"type": "string", "maxLength": max_length}


TERM = {"anyOf": [
    {"type": "object", "properties": {"var": _string(IDENT_MAX)}, "required": ["var"], "additionalProperties": False},
    {"type": "object", "properties": {"const": _string(IDENT_MAX)}, "required": ["const"], "additionalProperties": False},
    {"type": "string", "enum": ["none"]},
    {"type": "object", "properties": {"after": {"$ref": "#/$defs/term"}}, "required": ["after"], "additionalProperties": False},
    {"type": "object", "properties": {"record": {"type": "object", "properties": {
        "entity": _string(IDENT_MAX),
        "fields": {"type": "array", "items": {"type": "object", "properties": {
            "field": _string(IDENT_MAX), "value": {"$ref": "#/$defs/term"}},
            "required": ["field", "value"], "additionalProperties": False}}},
        "required": ["entity", "fields"], "additionalProperties": False}},
     "required": ["record"], "additionalProperties": False},
]}
TYPE = {"anyOf": [
    {"type": "object", "properties": {"entity": _string(IDENT_MAX)}, "required": ["entity"], "additionalProperties": False},
    {"type": "object", "properties": {"set": {"$ref": "#/$defs/type"}}, "required": ["set"], "additionalProperties": False},
    {"type": "object", "properties": {"optional": {"$ref": "#/$defs/type"}}, "required": ["optional"], "additionalProperties": False},
    {"type": "string", "enum": ["proposition_time", "derivation_time"]},
]}
TIME = {"anyOf": [
    {"type": "object", "properties": {"at": _string(IDENT_MAX)}, "required": ["at"], "additionalProperties": False},
    {"type": "object", "properties": {"var": _string(IDENT_MAX)}, "required": ["var"], "additionalProperties": False},
    {"type": "object", "properties": {"after": {"$ref": "#/$defs/time"}}, "required": ["after"], "additionalProperties": False},
]}
EVENT = {
    "type": "object",
    "properties": {
        "polarity": {"type": "string", "enum": ["positive", "evidential_not"]},
        "verb": _string(IDENT_MAX),
        "args": {"type": "array", "items": {"$ref": "#/$defs/term"}},
        "kind": {"type": "string", "enum": ["prop", "derived", "seeded", "effect"]},
        "time": {"$ref": "#/$defs/time"},
        "transition": {"type": "null"},
    },
    "required": ["polarity", "verb", "args", "kind", "time", "transition"],
    "additionalProperties": False,
}
BINDING = {"type": "object", "properties": {
    "scope": {"anyOf": [
        {"type": "object", "properties": {"rule": _string(IDENT_MAX)}, "required": ["rule"], "additionalProperties": False},
        {"type": "object", "properties": {"program": _string(IDENT_MAX)}, "required": ["program"], "additionalProperties": False},
    ]},
    "parameter": _string(IDENT_MAX),
    "ty": {"$ref": "#/$defs/type"},
    "value": {"anyOf": [
        {"type": "object", "properties": {"term": {"$ref": "#/$defs/term"}}, "required": ["term"], "additionalProperties": False},
        {"type": "object", "properties": {"time": {"$ref": "#/$defs/time"}}, "required": ["time"], "additionalProperties": False},
    ]},
}, "required": ["scope", "parameter", "ty", "value"], "additionalProperties": False}
ABDUCIBLE = {"type": "object", "properties": {
    "verb": _string(IDENT_MAX), "polarity": {"type": "string", "enum": ["positive", "evidential_not"]}},
    "required": ["verb", "polarity"], "additionalProperties": False}
DEFS = {"term": TERM, "time": TIME, "type": TYPE, "event": EVENT, "binding": BINDING, "abducible": ABDUCIBLE}


def _schema(tag: str) -> dict:
    base = tag.rstrip("?")
    optional = tag.endswith("?")
    if base == "ident":
        schema = _string(IDENT_MAX)
    elif base in ("idents", "ident_list"):
        schema = {"type": "array", "items": _string(IDENT_MAX)}
    elif base == "text":
        schema = _string(TEXT_MAX)
    elif base == "source":
        schema = _string(SOURCE_MAX)
    elif base == "steps":
        schema = {"type": "integer", "minimum": 1, "maximum": STEPS_MAX}
    elif base == "event":
        schema = {"$ref": "#/$defs/event"}
    elif base == "scope":
        schema = {"anyOf": [
            {"type": "object", "properties": {"rules": {"type": "array", "items": _string(IDENT_MAX)}},
             "required": ["rules"], "additionalProperties": False},
            {"type": "object", "properties": {"program": _string(IDENT_MAX)}, "required": ["program"],
             "additionalProperties": False},
        ]}
    elif base == "bindings":
        schema = {"type": "array", "items": {"$ref": "#/$defs/binding"}}
    elif base == "abducibles":
        schema = {"type": "array", "items": {"$ref": "#/$defs/abducible"}}
    elif base == "limits":
        schema = {"type": "object", "properties": {k: {"type": "integer", "minimum": 1} for k in ("max_depth", "max_nodes", "max_branches")},
                  "required": ["max_depth", "max_nodes", "max_branches"], "additionalProperties": False}
    elif base == "object":
        schema = {"type": "object", "properties": {"dossier": _string(IDENT_MAX), "note": _string(TEXT_MAX)},
                  "required": ["dossier", "note"], "additionalProperties": False}
    elif base == "claims":
        schema = {"type": "array", "maxItems": CLAIMS_MAX, "items": {"type": "object", "properties": {
            "kind": {"type": "string", "enum": list(CLAIM_KINDS)},
            "assertion": {"anyOf": [{"type": "null"}, {"type": "object", "properties": {
                "ref": _string(IDENT_MAX), "attribute": {"type": "string", "enum": list(CLAIM_ATTRIBUTES)},
                "value": _string(120)}, "required": ["ref", "attribute", "value"], "additionalProperties": False}]},
            "text": _string(CLAIM_TEXT_MAX),
        }, "required": ["kind", "assertion", "text"], "additionalProperties": False}}
    elif base == "hypothesis":
        schema = {"type": "object", "properties": {
            "statement": _string(TEXT_MAX),
            "grounds": {"type": "array", "maxItems": 4, "items": _string(200)},
            "status": {"type": "string", "enum": ["hypothesis"]},
        }, "required": ["statement", "grounds", "status"], "additionalProperties": False}
    else:
        raise ValueError(f"no schema for tag {tag}")
    if optional:
        schema = {"anyOf": [schema, {"type": "null"}]}
    return schema


def tool_definitions(condition: str) -> list[dict]:
    """Strict function tools: every property required (nullable when
    optional), additionalProperties false, shared $defs for events."""
    definitions = []
    for name in catalog(condition):
        arguments, _, _ = TOOLS[name]
        properties = {}
        for field, (tag, required) in {**arguments, **COMMON}.items():
            schema = _schema(tag if (required or tag.endswith("?")) else tag + "?")
            properties[field] = schema
        definitions.append({
            "type": "function",
            "name": name,
            "description": f"{name} under the episode contract; arguments are validated locally.",
            "strict": True,
            "parameters": {"type": "object", "properties": properties, "required": list(properties),
                           "additionalProperties": False, "$defs": DEFS},
        })
    return definitions


# -- the adapter ------------------------------------------------------------

def initial_context(manifest: EpisodeManifest, vocabulary: dict | None = None) -> str:
    """What the model is told at the start: the public facts of the episode
    and, when the controller froze it, the vocabulary package (signatures,
    entities, constants, rules of the baseline) with the Formal syntax the
    proposal path accepts. Identical across conditions. No repository
    documents, no reserved artefacts, no catalogue."""
    c = manifest.controller
    facts = {
        "vocabulary": vocabulary,
        "syntax": FORMAL_SYNTAX,
        "dossier_seeds": manifest.worker["dossiers"],
        "episode_id": manifest.episode_id,
        "condition": manifest.condition,
        "dossiers": manifest.dossiers,
        "decision": manifest.worker["decision"],
        "profile": manifest.worker["profile"],
        "budgets": {"actions": c["max_actions"], "model_requests": c["max_model_requests"]},
        "observations": [{"dossier": o["dossier"], "required": o["required"], "status": o["status"]}
                         for o in c["observations"]],
        "tools": catalog(manifest.condition),
    }
    return json.dumps(facts, ensure_ascii=False, sort_keys=True)


def _as_dict(value):
    if hasattr(value, "model_dump"):
        return value.model_dump()
    if isinstance(value, dict):
        return value
    raise ProviderError(f"response of type {type(value).__name__} is not a mapping")


class OpenAIResponsesProvider:
    """``client`` is injected and must expose ``responses.create(**body)``.
    The adapter never constructs a client, never reads credentials and never
    retries."""

    def __init__(self, client, manifest: EpisodeManifest):
        model = manifest.controller["model"]
        if model["provider"] != "openai":
            raise ProviderError(f"manifest provider is {model['provider']!r}, not openai")
        self.client = client
        self.manifest = manifest
        self.model_id = model["model_id"]
        self.parameters = dict(model.get("parameters") or {})
        collisions = set(self.parameters) & CONTROLLED
        unknown = set(self.parameters) - ALLOWED_PARAMETERS
        if collisions:
            raise ProviderError(f"manifest parameters would override controlled fields {sorted(collisions)}")
        if unknown:
            raise ProviderError(f"manifest parameters not authorised: {sorted(unknown)}")
        self.instructions = manifest.controller["explorer_instructions"]
        self.tools = tool_definitions(manifest.condition)
        # One entry per turn: every output item the model produced (function
        # call, reasoning or other continuation items, kept verbatim) and the
        # call id of its single function call.
        self.emitted: list[tuple[list, str]] = []
        self.vocabulary: dict | None = None      # frozen by the controller
        self.recorder = None                      # set by the controller: (kind, payload) -> None
        self.last_request: dict | None = None
        self.last_response: dict | None = None

    def build_request(self, visible_history: list, tools: list[str], output_cap: int) -> dict:
        if list(tools) != [t["name"] for t in self.tools]:
            raise ProviderError("tool catalogue differs from the manifest's condition")
        if len(visible_history) != len(self.emitted):
            raise ProviderError(f"conversation desynchronised: {len(visible_history)} results for "
                                f"{len(self.emitted)} model calls")
        items = [{"role": "user", "content": initial_context(self.manifest, self.vocabulary)}]
        for (output_items, call_id), item in zip(self.emitted, visible_history):
            # Every item of that answer is replayed as received, in order, so
            # that reasoning or continuation items stay attached to their call.
            items.extend(json.loads(json.dumps(output_items)))
            items.append({"type": "function_call_output", "call_id": call_id,
                          "output": json.dumps(item["view"], ensure_ascii=False, sort_keys=True)})
        body = {
            "model": self.model_id,
            "instructions": self.instructions,
            "input": items,
            "tools": self.tools,
            "tool_choice": "required",
            "parallel_tool_calls": False,
            "max_output_tokens": int(output_cap),
            "stream": False,
            "store": False,
        }
        body.update(self.parameters)
        return body

    @staticmethod
    def extract_usage(data: dict) -> dict | None:
        """Reception accounting, independent of whether the answer is usable."""
        usage = data.get("usage")
        if not isinstance(usage, dict) or "output_tokens" not in usage:
            return None
        out = {"output_tokens": int(usage["output_tokens"])}
        if "input_tokens" in usage:
            out["input_tokens"] = int(usage["input_tokens"])
        return out

    @staticmethod
    def parse_response(response) -> tuple[list, str, dict]:
        """Exactly one function call → (all output items, call_id, action)."""
        data = _as_dict(response)
        usage = OpenAIResponsesProvider.extract_usage(data)
        if data.get("status") != "completed":
            raise ProviderError(f"model_incomplete: status {data.get('status')!r}", usage)
        output = [_as_dict(item) for item in data.get("output", [])]
        calls = [item for item in output if item.get("type") == "function_call"]
        if len(calls) != 1:
            raise ProviderError(f"protocol: exactly one tool call is required, got {len(calls)}", usage)
        call = calls[0]
        try:
            arguments = json.loads(call["arguments"])
        except (KeyError, TypeError, json.JSONDecodeError) as error:
            raise ProviderError(f"protocol: tool arguments are not JSON: {error}", usage) from error
        if not isinstance(arguments, dict):
            raise ProviderError("protocol: tool arguments must be an object", usage)
        return output, call["call_id"], {"tool": call["name"], "arguments": arguments}

    def _record(self, kind: str, payload) -> None:
        if self.recorder is not None:
            self.recorder(kind, payload)

    def next_action(self, visible_history: list, tools: list[str], output_cap: int):
        body = self.build_request(visible_history, tools, output_cap)
        self.last_request = body
        # The exact body is persisted before any network; the raw answer
        # before any interpretation.
        self._record("model_request_body", body)
        try:
            response = self.client.responses.create(**body)
        except Exception as error:  # transport failures are reported, never retried here
            self._record("model_transport_error", {"error": f"{type(error).__name__}: {error}"})
            raise ProviderError(f"model_error: {type(error).__name__}: {error}") from error
        data = _as_dict(response)
        self.last_response = data
        self._record("model_response_raw", data)
        output, call_id, action = self.parse_response(data)
        self.emitted.append((output, call_id))
        return action, self.extract_usage(data)
