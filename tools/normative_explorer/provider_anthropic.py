"""Adapter for Claude through the Anthropic Messages API, with the client
injected and no network in this module (invocation protocol §3, §4).

The client is the official SDK's ``anthropic.Anthropic`` built by
``make_client`` (never in tests); the adapter only calls
``client.messages.create(**body)``. It builds the request the protocol
fixes, keeps the conversation strictly append-only, and parses exactly one
``tool_use`` block per answer.

Choices forced by the API and by the experiment:

- **Append-only history, blocks replayed verbatim.** Claude Opus 5.5 and
  Claude Fable 5.1 bind each thinking block's signature to the conversation
  prefix that produced it; editing earlier turns invalidates them and, on
  accounts created on or after 2026-08-31, the request fails. Every request
  is the previous one plus the stored assistant content (thinking blocks
  included, unchanged) and one ``tool_result``. ``system`` and ``tools`` are
  frozen for the episode.
- **No forced tool use.** These models reject ``tool_choice`` ``any`` and
  ``tool``; the request uses ``auto`` with ``disable_parallel_tool_use`` and
  the instructions say to answer with one tool call. An answer without
  exactly one ``tool_use`` is a ``ProviderError``; it is not stored, so the
  next request repeats the same prefix.
- **Strict only within the documented limits.** Across all strict schemas
  of one request the API accepts at most 20 strict tools, 24 optional
  parameters and 16 parameters with unions (``anyOf`` or type arrays).
  Pilot 001 was rejected for exceeding the last. Each tool's cost is a
  conservative count (every ``$ref`` expanded at each use, every union node
  and every non-required property counted); the cheapest tools are strict
  while the totals fit, the others are sent non-strict with the same full
  schema, so no term, binding or legal configuration is removed. Local and
  Rust validation run on every action either way. The API also has an
  internal complexity limit that cannot be checked offline.
- **Non-recursive strict schemas.** Strict tool use does not accept
  recursive schemas or numeric and length constraints, so terms, times and
  types are unrolled to the depths the P1 profile allows (``after`` nested at
  most twice, record fields one level) and limits stay with the local
  validator, which runs on every action anyway.
- **No server-side fallback.** The recommended refusal fallback would let
  another model answer; the protocol forbids a silent model substitution in
  a measured episode. A ``refusal`` stop is a ``ProviderError`` carrying its
  category and usage.
- **No retries.** ``make_client`` sets ``max_retries=0``; every failure is
  recorded by the controller and charged.
"""
from __future__ import annotations

import json
import os

from .manifest import EpisodeManifest
from .provider_base import ProviderError, initial_context
from .schemas import CLAIM_ATTRIBUTES, CLAIM_KINDS, COMMON, TOOLS, catalog

ADAPTER_VERSION = "provider-anthropic/0.1"

# Fields the protocol fixes; a manifest parameter may never touch them.
CONTROLLED = {"model", "system", "messages", "tools", "tool_choice", "max_tokens", "stream",
              "cache_control", "metadata", "stop_sequences", "betas", "fallbacks"}
# Additional parameters a manifest may set, each with the keys it may carry.
ALLOWED_PARAMETERS = {
    "output_config": {"effort"},
    "thinking": {"type", "display"},
    "inference_geo": None,
}
EFFORTS = {"low", "medium", "high", "xhigh", "max"}
# Non-streaming requests stay under the SDK's HTTP timeout at this size.
MAX_TOKENS_PER_REQUEST = 16000
# Margin for what the API adds to the prompt around tool definitions; the
# input bound is the serialized body's UTF-8 bytes (a token never covers
# less than one byte) plus this margin.
INPUT_BOUND_MARGIN_TOKENS = 4000

INSTRUCTION_SUFFIX = (
    "\n\nAnswer every turn with exactly one tool call and no other tool call. "
    "A turn without a tool call is not executed."
)


def make_client(timeout_seconds: float, workspace_id: str | None = None):
    """The real client, for a supervised pilot only; tests never call it.
    The key comes from ANTHROPIC_API_KEY as the SDK resolves it; the
    workspace from the argument or ANTHROPIC_WORKSPACE_ID, sent as a header
    and never written to the manifest or the ledger. The deadline comes from
    the frozen manifest; retries are always zero, so the SDK never repeats a
    request the controller did not record."""
    try:
        import anthropic  # imported here so the controller runs without the SDK
    except ImportError as error:
        raise ProviderError(f"the anthropic SDK is not installed: {error}") from error

    workspace = workspace_id or os.environ.get("ANTHROPIC_WORKSPACE_ID")
    headers = {"anthropic-workspace-id": workspace} if workspace else None
    return anthropic.Anthropic(max_retries=0, timeout=float(timeout_seconds), default_headers=headers)


# -- non-recursive strict schemas ---------------------------------------------

def _obj(properties: dict) -> dict:
    return {"type": "object", "properties": properties, "required": list(properties),
            "additionalProperties": False}


def _one_of(key: str, schema: dict) -> dict:
    return _obj({key: schema})


STRING = {"type": "string"}
TERM0 = {"anyOf": [_one_of("var", STRING), _one_of("const", STRING), {"type": "string", "enum": ["none"]}]}
TERM1 = {"anyOf": TERM0["anyOf"] + [_one_of("after", {"$ref": "#/$defs/term0"})]}
TERM2 = {"anyOf": TERM0["anyOf"] + [
    _one_of("after", {"$ref": "#/$defs/term1"}),
    _one_of("record", _obj({"entity": STRING, "fields": {
        "type": "array", "items": _obj({"field": STRING, "value": {"$ref": "#/$defs/term1"}})}})),
]}
TIME0 = {"anyOf": [_one_of("at", STRING), _one_of("var", STRING)]}
TIME1 = {"anyOf": TIME0["anyOf"] + [_one_of("after", {"$ref": "#/$defs/time0"})]}
TIME2 = {"anyOf": TIME0["anyOf"] + [_one_of("after", {"$ref": "#/$defs/time1"})]}
TYPE0 = {"anyOf": [_one_of("entity", STRING), {"type": "string", "enum": ["proposition_time", "derivation_time"]}]}
TYPE1 = {"anyOf": TYPE0["anyOf"] + [_one_of("set", {"$ref": "#/$defs/type0"}),
                                     _one_of("optional", {"$ref": "#/$defs/type0"})]}
TYPE2 = {"anyOf": TYPE0["anyOf"] + [_one_of("set", {"$ref": "#/$defs/type1"}),
                                     _one_of("optional", {"$ref": "#/$defs/type1"})]}
EVENT = _obj({
    "polarity": {"type": "string", "enum": ["positive", "evidential_not"]},
    "verb": STRING,
    "args": {"type": "array", "items": {"$ref": "#/$defs/term2"}},
    "kind": {"type": "string", "enum": ["prop", "derived", "seeded", "effect"]},
    "time": {"$ref": "#/$defs/time2"},
    "transition": {"type": "null"},
})
BINDING = _obj({
    "scope": {"anyOf": [_one_of("rule", STRING), _one_of("program", STRING)]},
    "parameter": STRING,
    "ty": {"$ref": "#/$defs/type2"},
    "value": {"anyOf": [_one_of("term", {"$ref": "#/$defs/term2"}), _one_of("time", {"$ref": "#/$defs/time2"})]},
})
ABDUCIBLE = _obj({"verb": STRING, "polarity": {"type": "string", "enum": ["positive", "evidential_not"]}})
# Ordered so that every reference points to an earlier definition: no cycle.
DEFS = {"term0": TERM0, "term1": TERM1, "term2": TERM2, "time0": TIME0, "time1": TIME1, "time2": TIME2,
        "type0": TYPE0, "type1": TYPE1, "type2": TYPE2, "event": EVENT, "binding": BINDING,
        "abducible": ABDUCIBLE}


def _nullable(schema: dict) -> dict:
    return {"anyOf": [schema, {"type": "null"}]}


def _schema(tag: str) -> dict:
    base = tag.rstrip("?")
    if base == "ident" or base in ("text", "source"):
        schema = STRING
    elif base in ("idents", "ident_list"):
        schema = {"type": "array", "items": STRING}
    elif base == "steps":
        schema = {"type": "integer"}
    elif base == "event":
        schema = {"$ref": "#/$defs/event"}
    elif base == "scope":
        schema = {"anyOf": [_one_of("rules", {"type": "array", "items": STRING}), _one_of("program", STRING)]}
    elif base == "bindings":
        schema = {"type": "array", "items": {"$ref": "#/$defs/binding"}}
    elif base == "abducibles":
        schema = {"type": "array", "items": {"$ref": "#/$defs/abducible"}}
    elif base == "limits":
        schema = _obj({k: {"type": "integer"} for k in ("max_depth", "max_nodes", "max_branches")})
    elif base == "object":
        schema = _obj({"dossier": STRING, "note": STRING})
    elif base == "claims":
        schema = {"type": "array", "items": _obj({
            "kind": {"type": "string", "enum": list(CLAIM_KINDS)},
            "assertion": _nullable(_obj({"ref": STRING, "attribute": {"type": "string", "enum": list(CLAIM_ATTRIBUTES)},
                                         "value": STRING})),
            "text": STRING,
        })}
    elif base == "hypothesis":
        schema = _obj({"statement": STRING, "grounds": {"type": "array", "items": STRING},
                       "status": {"type": "string", "enum": ["hypothesis"]}})
    else:
        raise ValueError(f"no schema for tag {tag}")
    return _nullable(schema) if tag.endswith("?") else schema


def _reachable(schema, defs: dict) -> dict:
    """Only the definitions a tool's properties reach, in DEFS order, so
    that no tool carries definitions it never uses."""
    seen, pending = set(), [schema]
    while pending:
        node = pending.pop()
        if isinstance(node, dict):
            ref = node.get("$ref")
            if ref:
                name = ref.split("/")[-1]
                if name not in seen:
                    seen.add(name)
                    pending.append(defs[name])
            pending.extend(node.values())
        elif isinstance(node, list):
            pending.extend(node)
    return {name: definition for name, definition in defs.items() if name in seen}


# Documented limits across all strict schemas of one request.
STRICT_LIMITS = {"strict_tools": 20, "optional_parameters": 24, "union_parameters": 16}


def strict_cost(schema: dict) -> dict:
    """Conservative count of what a schema adds to the strict limits: each
    ``$ref`` is expanded at every use, every node with ``anyOf`` or a type
    array counts as a union parameter, every property missing from its
    object's ``required`` counts as optional. An upper bound on the API's
    own count, not that count."""
    defs = schema.get("$defs", {})
    unions = optional = 0
    pending = [schema]
    while pending:
        node = pending.pop()
        if isinstance(node, list):
            pending.extend(node)
            continue
        if not isinstance(node, dict):
            continue
        ref = node.get("$ref")
        if ref:
            pending.append(defs[ref.split("/")[-1]])
        if "anyOf" in node or isinstance(node.get("type"), list):
            unions += 1
        if isinstance(node.get("properties"), dict):
            optional += len(set(node["properties"]) - set(node.get("required", [])))
        pending.extend(v for k, v in node.items() if k != "$defs")
    return {"union_parameters": unions, "optional_parameters": optional}


def strict_totals(tools: list[dict]) -> dict:
    totals = {"strict_tools": 0, "optional_parameters": 0, "union_parameters": 0}
    for tool in tools:
        if tool.get("strict"):
            totals["strict_tools"] += 1
            for key, value in strict_cost(tool["input_schema"]).items():
                totals[key] += value
    return totals


def check_strict_limits(tools: list[dict]) -> dict:
    """Preflight of the whole catalogue against the documented limits, by
    the conservative count. Raises a non-recoverable ProviderError."""
    totals = strict_totals(tools)
    over = {k: (v, STRICT_LIMITS[k]) for k, v in totals.items() if v > STRICT_LIMITS[k]}
    if over:
        raise ProviderError(f"strict tool schemas exceed the documented limits (conservative count): {over}",
                            fatal=True)
    return totals


def tool_definitions(condition: str) -> list[dict]:
    """The condition's catalogue, in catalogue order. Every property is
    required, optional ones nullable; each tool carries only the definitions
    it reaches. Tools are made strict from the cheapest (ties in catalogue
    order) while the conservative totals fit the documented limits; the
    rest are sent non-strict with the same schema."""
    definitions = []
    for name in catalog(condition):
        arguments, _, _ = TOOLS[name]
        properties = {}
        for field, (tag, required) in {**arguments, **COMMON}.items():
            properties[field] = _schema(tag if (required or tag.endswith("?")) else tag + "?")
        definitions.append({
            "name": name,
            "description": f"{name} under the episode contract; arguments are validated locally before anything runs.",
            "strict": False,
            "input_schema": {"type": "object", "properties": properties, "required": list(properties),
                             "additionalProperties": False, "$defs": _reachable(properties, DEFS)},
        })
    totals = {key: 0 for key in STRICT_LIMITS}
    order = sorted(range(len(definitions)),
                   key=lambda i: (sum(strict_cost(definitions[i]["input_schema"]).values()), i))
    for i in order:
        cost = dict(strict_cost(definitions[i]["input_schema"]), strict_tools=1)
        if all(totals[k] + cost[k] <= STRICT_LIMITS[k] for k in STRICT_LIMITS):
            definitions[i]["strict"] = True
            for k in STRICT_LIMITS:
                totals[k] += cost[k]
    check_strict_limits(definitions)
    return definitions


# Error types (and, without a type, HTTP statuses) that the same request
# cannot recover from.
NON_RECOVERABLE_TYPES = {"invalid_request_error", "authentication_error", "permission_error",
                         "not_found_error", "request_too_large"}
NON_RECOVERABLE_STATUSES = {400, 401, 403, 404, 413}


def error_details(error: Exception) -> dict:
    """Structured facts of an SDK error, read by attribute so that any
    client exposing them works: HTTP status, error type and message,
    request id."""
    body = getattr(error, "body", None)
    inner = body.get("error") if isinstance(body, dict) and isinstance(body.get("error"), dict) else {}
    request_id = getattr(error, "request_id", None)
    if request_id is None and isinstance(body, dict):
        request_id = body.get("request_id")
    return {"exception": type(error).__name__, "status_code": getattr(error, "status_code", None),
            "error_type": inner.get("type"), "error_message": inner.get("message"), "request_id": request_id}


def is_non_recoverable(details: dict) -> bool:
    if details["error_type"] is not None:
        return details["error_type"] in NON_RECOVERABLE_TYPES
    return details["status_code"] in NON_RECOVERABLE_STATUSES


# -- the adapter --------------------------------------------------------------

def _dump(value) -> dict:
    """The full response as JSON data, nulls included, for the raw record."""
    if hasattr(value, "model_dump"):
        return value.model_dump(mode="json")
    if isinstance(value, dict):
        return json.loads(json.dumps(value))
    raise ProviderError(f"response of type {type(value).__name__} is not a mapping")


def _replay_blocks(content: list) -> list:
    """Content blocks as sent back: only each block's own null fields are
    dropped (e.g. an absent ``citations`` or ``caller``); a tool's ``input``
    and every other value stay exactly as received, nulls included."""
    return [{k: v for k, v in block.items() if v is not None} for block in content]


def _validate_parameters(parameters: dict) -> None:
    for key, value in parameters.items():
        if key in CONTROLLED:
            raise ProviderError(f"manifest parameter {key!r} would override a controlled field")
        if key not in ALLOWED_PARAMETERS:
            raise ProviderError(f"manifest parameter {key!r} is not authorised")
        allowed = ALLOWED_PARAMETERS[key]
        if allowed is not None:
            if not isinstance(value, dict) or set(value) - allowed:
                raise ProviderError(f"manifest parameter {key!r} may only carry {sorted(allowed)}")
    if "output_config" in parameters and parameters["output_config"].get("effort") not in EFFORTS:
        raise ProviderError(f"effort must be one of {sorted(EFFORTS)}")
    if "thinking" in parameters and parameters["thinking"].get("type") != "adaptive":
        raise ProviderError("thinking may only be adaptive: disabling or budgeting it is rejected by current models")


class AnthropicMessagesProvider:
    def __init__(self, client, manifest: EpisodeManifest):
        model = manifest.controller["model"]
        if model["provider"] != "anthropic":
            raise ProviderError(f"manifest provider is {model['provider']!r}, not anthropic")
        self.client = client
        self.manifest = manifest
        self.model_id = model["model_id"]
        self.parameters = dict(model.get("parameters") or {})
        _validate_parameters(self.parameters)
        self.system = manifest.controller["explorer_instructions"] + INSTRUCTION_SUFFIX
        self.tools = tool_definitions(manifest.condition)
        # One entry per accepted answer: its content blocks as replayed
        # (thinking blocks included, unchanged) and its tool_use id.
        self.emitted: list[tuple[list, str]] = []
        self.vocabulary: dict | None = None
        self._opening: str | None = None   # frozen on the first request
        self.recorder = None
        self.last_request: dict | None = None
        self.last_response: dict | None = None

    def build_request(self, visible_history: list, tools: list[str], output_cap: int) -> dict:
        if list(tools) != [t["name"] for t in self.tools]:
            raise ProviderError("tool catalogue differs from the manifest's condition")
        if len(visible_history) != len(self.emitted):
            raise ProviderError(f"conversation desynchronised: {len(visible_history)} results for "
                                f"{len(self.emitted)} model calls")
        if self._opening is None:
            self._opening = initial_context(self.manifest, self.vocabulary)
        messages = [{"role": "user", "content": self._opening}]
        for (content, tool_use_id), item in zip(self.emitted, visible_history):
            view = item["view"]
            messages.append({"role": "assistant", "content": json.loads(json.dumps(content))})
            result = {"type": "tool_result", "tool_use_id": tool_use_id,
                      "content": json.dumps(view, ensure_ascii=False, sort_keys=True)}
            if isinstance(view, dict) and "protocol_error" in view:
                result["is_error"] = True
            messages.append({"role": "user", "content": [result]})
        body = {
            "model": self.model_id,
            "max_tokens": int(min(output_cap, MAX_TOKENS_PER_REQUEST)),
            "system": self.system,
            "messages": messages,
            "tools": self.tools,
            "tool_choice": {"type": "auto", "disable_parallel_tool_use": True},
            "cache_control": {"type": "ephemeral"},
        }
        body.update(json.loads(json.dumps(self.parameters)))
        return body

    def request_bounds(self, visible_history: list, tools: list[str], output_cap: int) -> tuple[int, int]:
        """Upper bounds for the next request, computed from the request that
        would be sent: input tokens at most the body's UTF-8 bytes plus a
        fixed margin; output tokens at most its max_tokens."""
        body = self.build_request(visible_history, tools, output_cap)
        size = len(json.dumps(body, ensure_ascii=False).encode("utf-8"))
        return size + INPUT_BOUND_MARGIN_TOKENS, int(body["max_tokens"])

    @staticmethod
    def extract_usage(data: dict) -> dict | None:
        usage = data.get("usage")
        if not isinstance(usage, dict):
            return None

        def count(key):
            # A declared count is a non-negative integer; absent, null or
            # malformed is unknown, never zero.
            value = usage.get(key)
            if isinstance(value, int) and not isinstance(value, bool) and value >= 0:
                return value
            return None
        output = count("output_tokens")
        if output is None:
            return None
        result = {"output_tokens": output}
        parts = {"uncached_input_tokens": count("input_tokens"),
                 "cache_creation_input_tokens": count("cache_creation_input_tokens"),
                 "cache_read_input_tokens": count("cache_read_input_tokens")}
        result.update({k: v for k, v in parts.items() if v is not None})
        # The input total is reported only when every part is declared.
        if None not in parts.values():
            result["input_tokens"] = sum(parts.values())
        return result

    @staticmethod
    def parse_response(data: dict) -> tuple[list, str, dict]:
        """Exactly one tool_use block → (content for replay, tool_use id, action)."""
        usage = AnthropicMessagesProvider.extract_usage(data)
        stop = data.get("stop_reason")
        if stop == "refusal":
            details = data.get("stop_details") or {}
            raise ProviderError(f"refusal: category {details.get('category')!r}", usage)
        if stop == "max_tokens":
            raise ProviderError("model_incomplete: stop_reason max_tokens", usage)
        content = _replay_blocks(data.get("content") or [])
        calls = [block for block in content if block.get("type") == "tool_use"]
        if len(calls) != 1:
            raise ProviderError(f"protocol: exactly one tool call is required, got {len(calls)} "
                                f"(stop_reason {stop!r})", usage)
        call = calls[0]
        if not isinstance(call.get("input"), dict):
            raise ProviderError("protocol: tool input must be an object", usage)
        return content, call["id"], {"tool": call["name"], "arguments": call["input"]}

    def _record(self, kind: str, payload) -> None:
        if self.recorder is not None:
            self.recorder(kind, payload)

    def next_action(self, visible_history: list, tools: list[str], output_cap: int):
        body = self.build_request(visible_history, tools, output_cap)
        self.last_request = body
        self._record("model_request_body", body)
        try:
            response = self.client.messages.create(**body)
        except Exception as error:  # every failure is recorded; no retry here
            details = error_details(error)
            self._record("model_transport_error", {"error": f"{type(error).__name__}: {error}", "details": details})
            raise ProviderError(f"model_error: {type(error).__name__}: {error}", details=details,
                                fatal=is_non_recoverable(details)) from error
        raw = _dump(response)
        self.last_response = raw
        self._record("model_response_raw", raw)
        content, tool_use_id, action = self.parse_response(raw)
        self.emitted.append((content, tool_use_id))
        return action, self.extract_usage(raw)
