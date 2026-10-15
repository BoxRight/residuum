"""Tool catalogue and action validation (invocation protocol §5, §10).

Every tool has a closed argument schema plus the common explanation fields.
Validation here is structural and strict: unknown keys, wrong types, bad
identifiers and out-of-range numbers are protocol errors. Deep validation of
events, bindings and references against the theory stays in the worker.
"""
from __future__ import annotations

from dataclasses import dataclass

IDENT_MAX = 128
TEXT_MAX = 400
CLAIM_TEXT_MAX = 300
CLAIMS_MAX = 8
SOURCE_MAX = 64 * 1024
STEPS_MAX = 4096

CLAIM_KINDS = ("residuum_result", "hypothesis_change", "investigation_reason")
CLAIM_ATTRIBUTES = ("status", "component", "contains_event", "classes", "compatible", "differs_from")

# Tool → (argument schema, conditions that offer it, executed by the worker?)
# Schema entries: name → (type tag, required).
TOOLS: dict[str, tuple[dict[str, tuple[str, bool]], tuple[str, ...], bool]] = {
    "inspect_theory": ({}, ("A", "B", "C"), True),
    "begin_forward": ({"dossier": ("ident", True), "theory": ("ident?", False)}, ("A", "B", "C"), True),
    "begin_residual": ({
        "dossier": ("ident", True), "theory": ("ident?", False), "goal": ("event", True),
        "scope": ("scope?", False), "bindings": ("list", False), "allowed": ("list?", False),
        "limits": ("limits?", False),
    }, ("A", "B", "C"), True),
    "begin_active_residual": ({
        "dossier": ("ident", True), "theory": ("ident?", False), "goal": ("event", True),
        "scope": ("scope?", False), "bindings": ("list", False), "allowed": ("list?", False),
        "limits": ("limits?", False),
    }, ("A", "B", "C"), True),
    "advance_session": ({"session": ("ident", True), "max_steps": ("steps", True)}, ("A", "B", "C"), True),
    "propose_candidate": ({"source": ("source", True), "parent": ("ident?", False), "move": ("text?", False)},
                          ("A", "B", "C"), True),
    "begin_candidate_evaluation": ({"candidate": ("ident", True), "dossier": ("ident", True)}, ("A", "B", "C"), True),
    "request_valuation": ({"case_spec": ("object", True), "expected_discrimination": ("text", True)},
                          ("A", "B", "C"), False),
    "minimal_supports": ({"candidate": ("ident", True), "goal": ("event", True)}, ("B", "C"), True),
    "compare_partition": ({"candidates": ("idents", True), "goal": ("event", True)}, ("C",), True),
    # Finishing with no candidate is legitimate (no_verified_candidate).
    "finish_episode": ({"candidate_ids": ("ident_list", True), "explanation": ("text", True)}, ("A", "B", "C"), False),
}
COMMON = {"rationale": ("text", True), "expected_observation": ("text?", True),
          "claims": ("claims", False), "legal_hypothesis": ("hypothesis?", False)}


class ProtocolError(ValueError):
    """The model's output is not an acceptable action; nothing executes."""


@dataclass(frozen=True)
class ValidatedAction:
    tool: str
    arguments: dict
    rationale: str
    expected_observation: str | None
    claims: list
    legal_hypothesis: dict | None
    worker_side: bool


def catalog(condition: str) -> list[str]:
    return [name for name, (_, conditions, _) in TOOLS.items() if condition in conditions]


def is_ident(value) -> bool:
    return (isinstance(value, str) and 0 < len(value) <= IDENT_MAX
            and not any(c.isspace() or ord(c) < 32 for c in value))


def _check(tag: str, value, where: str) -> None:
    optional = tag.endswith("?")
    base = tag.rstrip("?")
    if value is None:
        if optional:
            return
        raise ProtocolError(f"{where}: must not be null")
    if base == "ident" and not is_ident(value):
        raise ProtocolError(f"{where}: not an identifier")
    elif base in ("idents", "ident_list"):
        if not isinstance(value, list) or not all(is_ident(v) for v in value):
            raise ProtocolError(f"{where}: must be a list of identifiers")
        if base == "idents" and not value:
            raise ProtocolError(f"{where}: must not be empty")
        if len(set(value)) != len(value):
            raise ProtocolError(f"{where}: duplicate identifiers")
    elif base == "text":
        if not isinstance(value, str) or len(value) > TEXT_MAX:
            raise ProtocolError(f"{where}: text up to {TEXT_MAX} characters")
    elif base == "source":
        if not isinstance(value, str) or not value.strip() or len(value) > SOURCE_MAX:
            raise ProtocolError(f"{where}: non-empty source up to {SOURCE_MAX} bytes")
    elif base == "steps":
        if not isinstance(value, int) or isinstance(value, bool) or not 1 <= value <= STEPS_MAX:
            raise ProtocolError(f"{where}: integer in 1..{STEPS_MAX}")
    elif base == "event":
        if not isinstance(value, dict) or set(value) != {"polarity", "verb", "args", "kind", "time", "transition"}:
            raise ProtocolError(f"{where}: event with exactly polarity, verb, args, kind, time, transition")
        if not is_ident(value["verb"]) or not isinstance(value["args"], list):
            raise ProtocolError(f"{where}: event verb and args")
    elif base == "scope":
        if not isinstance(value, dict) or len(value) != 1 or next(iter(value)) not in ("rules", "program"):
            raise ProtocolError(f"{where}: scope is {{rules: [...]}} or {{program: name}}")
    elif base == "list":
        if not isinstance(value, list):
            raise ProtocolError(f"{where}: list")
    elif base == "limits":
        if not isinstance(value, dict) or set(value) != {"max_depth", "max_nodes", "max_branches"}:
            raise ProtocolError(f"{where}: limits with max_depth, max_nodes, max_branches")
        if not all(isinstance(v, int) and not isinstance(v, bool) and v > 0 for v in value.values()):
            raise ProtocolError(f"{where}: limits must be positive integers")
    elif base == "object":
        if not isinstance(value, dict):
            raise ProtocolError(f"{where}: object")
    elif base == "claims":
        if not isinstance(value, list) or len(value) > CLAIMS_MAX:
            raise ProtocolError(f"{where}: at most {CLAIMS_MAX} claims")
        for i, claim in enumerate(value):
            if not isinstance(claim, dict) or set(claim) != {"kind", "assertion", "text"}:
                raise ProtocolError(f"{where}[{i}]: claim with kind, assertion, text")
            if claim["kind"] not in CLAIM_KINDS or not isinstance(claim["text"], str) or len(claim["text"]) > CLAIM_TEXT_MAX:
                raise ProtocolError(f"{where}[{i}]: kind or text")
            a = claim["assertion"]
            if a is not None:
                if not isinstance(a, dict) or set(a) != {"ref", "attribute", "value"}:
                    raise ProtocolError(f"{where}[{i}]: assertion with ref, attribute, value")
                if not is_ident(a["ref"]) or a["attribute"] not in CLAIM_ATTRIBUTES or not isinstance(a["value"], str):
                    raise ProtocolError(f"{where}[{i}]: assertion fields")
    elif base == "hypothesis":
        if not isinstance(value, dict) or set(value) != {"statement", "grounds", "status"}:
            raise ProtocolError(f"{where}: hypothesis with statement, grounds, status")
        if value["status"] != "hypothesis" or not isinstance(value["statement"], str) or len(value["statement"]) > TEXT_MAX:
            raise ProtocolError(f"{where}: hypothesis status and statement")
        if not isinstance(value["grounds"], list) or len(value["grounds"]) > 4 or not all(
                isinstance(g, str) and len(g) <= 200 for g in value["grounds"]):
            raise ProtocolError(f"{where}: up to four textual grounds")


def validate_action(action, condition: str) -> ValidatedAction:
    """One tool call: {"tool": name, "arguments": {...common fields...}}."""
    if not isinstance(action, dict) or set(action) != {"tool", "arguments"}:
        raise ProtocolError("action must be an object with exactly tool and arguments")
    tool = action["tool"]
    if tool not in TOOLS:
        raise ProtocolError(f"unknown tool {tool!r}")
    schema, conditions, worker_side = TOOLS[tool]
    if condition not in conditions:
        raise ProtocolError(f"tool {tool} is not offered in condition {condition}")
    arguments = action["arguments"]
    if not isinstance(arguments, dict):
        raise ProtocolError("arguments must be an object")
    allowed = set(schema) | set(COMMON)
    unknown = set(arguments) - allowed
    if unknown:
        raise ProtocolError(f"unknown arguments {sorted(unknown)}")
    for name, (tag, required) in {**schema, **COMMON}.items():
        if name not in arguments:
            if required:
                raise ProtocolError(f"missing argument {name}")
            continue
        _check(tag, arguments[name], name)
    return ValidatedAction(
        tool=tool,
        arguments={k: v for k, v in arguments.items() if k in schema},
        rationale=arguments["rationale"],
        expected_observation=arguments.get("expected_observation"),
        claims=arguments.get("claims") or [],
        legal_hypothesis=arguments.get("legal_hypothesis"),
        worker_side=worker_side,
    )
