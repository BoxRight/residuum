"""Pilot 004-R driver: connects one model provider to the trajectory process.

The trajectory process (``residuum-trajectory``) holds the theory, the
dossiers and the reserved valuations; this driver never reads the reserved
file. Each turn the process emits a view; the driver forwards it to the model,
turns the model's single tool call into an action, and returns it with the
usage of the call. Every request, response, view and action is appended to the
ledger. A cost ceiling and a request ceiling stop the model calls; once a
ceiling is reached every remaining turn ends its phase without a call.

Providers: ``anthropic`` (Messages API over the standard library; the key is
read from ANTHROPIC_API_KEY or from the file named by
RESIDUUM_ANTHROPIC_KEY_FILE, default ~/.config/residuum/anthropic_api_key) and
``scripted`` (fixed tool calls, no network) for local checks.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

API_URL = "https://api.anthropic.com/v1/messages"

TOOLS = [
    {
        "name": "propose_rule",
        "description": "Submit Formal rule declarations as a new hypothesis. Residuum parses, elaborates and "
                       "admits it or reports why not. Proposing does not commit it.",
        "input_schema": {
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "One or more Formal rule declarations."},
                "parent": {"type": ["string", "null"], "description": "Id of a hypothesis this one revises, if any."},
                "rationale": {"type": "string"},
            },
            "required": ["text", "rationale"],
        },
    },
    {
        "name": "evaluate_rule",
        "description": "Run the closure of LegalGeneral plus the committed hypotheses plus this hypothesis on an "
                       "already presented dossier, with Residuum's independent certificate. Uses one verified "
                       "evaluation from the budget. Returns certificate status, new events and positions the "
                       "hypothesis contributes. It never returns valuations or loss.",
        "input_schema": {
            "type": "object",
            "properties": {"hypothesis": {"type": "string"}, "dossier": {"type": "string"}},
            "required": ["hypothesis", "dossier"],
        },
    },
    {
        "name": "commit",
        "description": "Replace the committed set of hypotheses. The new theory must certify on every presented "
                       "dossier. The committed set is what is scored.",
        "input_schema": {
            "type": "object",
            "properties": {"hypotheses": {"type": "array", "items": {"type": "string"}}},
            "required": ["hypotheses"],
        },
    },
    {
        "name": "end_phase",
        "description": "End the current phase. Ending an explore phase triggers the scoring of the current dossier "
                       "with the committed set. An explore phase ends only after at least one proposal in that "
                       "phase or with a stated abstention.",
        "input_schema": {
            "type": "object",
            "properties": {"abstention": {"type": "string",
                                          "description": "Why you end this explore phase without proposing."}},
        },
    },
]

GOAL = {"type": "string", "description": "One event in Formal, e.g. verb(arg, ...) @ time; ~Act(...) @ time for a negative Act."}
HYPOTHESIS = {"type": ["string", "null"], "description": "Id of an admitted hypothesis: the theory is LegalGeneral plus "
                                                         "that hypothesis. Null or absent: LegalGeneral alone."}
DOSSIER = {"type": "string", "description": "Id of a presented dossier."}
def draft_tools(tools: list[dict]) -> list[dict]:
    """The run-006 catalogue: same tools, described as drafting, hypothetical testing and commitment, plus
    discard and a reason on commit (both recorded only)."""
    descriptions = {
        "propose_rule": "Create a draft hypothesis from Formal rule declarations. Residuum parses, elaborates and "
                        "admits it or reports why not. A draft has no effect on the committed set R_k and is "
                        "never scored.",
        "evaluate_rule": "Test a hypothesis hypothetically: the closure of LegalGeneral plus the committed "
                         "hypotheses plus this one on a presented dossier, with Residuum's independent "
                         "certificate. No penalty; uses one verified evaluation from the budget. Returns "
                         "certificate status, new events and positions the hypothesis contributes; never "
                         "valuations or loss.",
        "commit": "Decide which hypotheses form the committed set R_k (it replaces the current one). Committed "
                  "hypotheses are scored and contrasted with later dossiers. The new theory must certify on "
                  "every presented dossier.",
    }
    out = []
    for tool in tools:
        tool = json.loads(json.dumps(tool))
        if tool["name"] in descriptions:
            tool["description"] = descriptions[tool["name"]]
        if tool["name"] == "commit":
            tool["input_schema"]["properties"]["reason"] = {
                "type": "string", "description": "Why you commit this set (recorded only)."}
        out.append(tool)
    out.append({
        "name": "discard",
        "description": "Record that you set a hypothesis aside and why. It executes nothing and does not change "
                       "the committed set.",
        "input_schema": {"type": "object", "properties": {
            "hypothesis": {"type": "string"}, "reason": {"type": "string"}},
            "required": ["hypothesis", "reason"]},
    })
    return out


QUESTION_TOOL = {
    "name": "pose_question",
    "description": "Record an investigable question about the presented dossiers and the theory, and the tool "
                   "you will use to answer it. It executes nothing.",
    "input_schema": {"type": "object", "properties": {
        "question": {"type": "string"},
        "tool": {"type": "string", "description": "Name of the tool you will consult to answer it."}},
        "required": ["question", "tool"]},
}

NAVIGATION_TOOLS = [
    {
        "name": "inspect_theory",
        "description": "Inventory of the theory: rules with their bodies, verbs with signatures and kinds, entities, "
                       "constants and the presented dossiers.",
        "input_schema": {"type": "object", "properties": {}},
    },
    {
        "name": "begin_forward",
        "description": "Open a forward session: the closure of a dossier under a theory, advanced round by round "
                       "with advance_session (each round lists added events and their derivations).",
        "input_schema": {"type": "object", "properties": {"dossier": DOSSIER, "hypothesis": HYPOTHESIS},
                         "required": ["dossier"]},
    },
    {
        "name": "begin_residual",
        "description": "Open a residual (backward) query session for a goal on a dossier under a theory: what is "
                       "missing for the goal to hold, as search trees and residual conditions. Advance it with "
                       "advance_session.",
        "input_schema": {
            "type": "object",
            "properties": {
                "dossier": DOSSIER, "hypothesis": HYPOTHESIS, "goal": GOAL,
                "scope": {"type": ["object", "null"],
                          "description": "Optional: {\"rules\": [rule names]} or {\"program\": name}."},
                "allowed": {"type": ["array", "null"],
                            "description": "Optional abducibles: [{\"verb\": name, \"polarity\": "
                                           "\"positive\" | \"evidential_not\"}]."},
                "limits": {"type": ["object", "null"],
                           "description": "Optional: {\"max_depth\", \"max_nodes\", \"max_branches\"}."},
                "bindings": {"type": ["array", "null"],
                             "description": "Optional typed parameter bindings in the worker wire format."},
            },
            "required": ["dossier", "goal"],
        },
    },
    {
        "name": "begin_active_residual",
        "description": "As begin_residual, over the active (program) residual search.",
        "input_schema": {
            "type": "object",
            "properties": {
                "dossier": DOSSIER, "hypothesis": HYPOTHESIS, "goal": GOAL,
                "scope": {"type": ["object", "null"]}, "allowed": {"type": ["array", "null"]},
                "limits": {"type": ["object", "null"]}, "bindings": {"type": ["array", "null"]},
            },
            "required": ["dossier", "goal"],
        },
    },
    {
        "name": "advance_session",
        "description": "Advance an open forward or residual session by at most max_steps engine steps.",
        "input_schema": {"type": "object", "properties": {
            "session": {"type": "string"}, "max_steps": {"type": "integer", "minimum": 1}},
            "required": ["session", "max_steps"]},
    },
    {
        "name": "minimal_supports",
        "description": "Minimal sets of seeded facts, over the presented dossiers, from which LegalGeneral plus "
                       "the hypothesis derives the goal.",
        "input_schema": {"type": "object", "properties": {
            "hypothesis": {"type": "string"}, "goal": GOAL}, "required": ["hypothesis", "goal"]},
    },
    {
        "name": "compare_partition",
        "description": "Compare two or more hypotheses on a goal: the pool of minimal supports, which hypotheses "
                       "derive the goal from each (signatures), the supports that discriminate them, and "
                       "inclusion between their regions.",
        "input_schema": {"type": "object", "properties": {
            "hypotheses": {"type": "array", "items": {"type": "string"}, "minItems": 2}, "goal": GOAL},
            "required": ["hypotheses", "goal"]},
    },
]


SYNTAX = """Formal rule syntax accepted by propose_rule (rules only: no module, import, verb, const,
entity or experiment declarations):

  rule <name>(<param>, ...) =
      <atom> [& <atom>]...
      <= <head atom>

An atom is <verb>(<arg>, ...) @ <time>; a negative atom is ~Act(...) @ <time> (only Act may be
negated). Arguments are parameters or declared constants. Times are a parameter, a constant or
`after <param>`. Every parameter of the head must occur in the body. Rule names must be new.
Head verbs allowed: {heads}. Body verbs allowed: {body}. At most {atoms} body atoms and
{clauses} rules per proposal. Constants must be declared in LegalGeneral or occur in a dossier
already presented."""


DRAFT_EXPLORE = """- explore: you see the dossier and the closure Residuum computes for LegalGeneral plus your committed
  hypotheses, with justifications. This phase is for investigating the dossier: which relations it contains,
  what consequences possible generalisations would have, and how alternative hypotheses differ.
  propose_rule creates a draft hypothesis: a draft has no effect on your committed set R_k and is never
  scored. evaluate_rule tests a hypothesis hypothetically on a presented dossier, without penalty. You may
  draft, test, compare and discard as many alternatives as the budgets allow; discard records that you set a
  draft aside and why, and pose_question records a question if that helps you. commit is the decision to
  incorporate hypotheses into R_k so that they are contrasted with later experience; give its reason. When
  you end the phase, the committed set is scored on this dossier: loss = required positions not derived +
  excluded positions derived. Drafts you did not commit play no part in it. That prequential score is
  final and is never revised. The phase ends only after at least one draft in it or with a stated
  abstention; you are never required to produce a rule."""

DRAFT_REVISE = """- revise: {feedback} You may revise, keep or discard hypotheses (draft, test, discard and commit again),
  also when no dossiers remain. The changes apply to the following dossiers; every presented dossier is also
  rescored with the new committed set, recorded separately as a retrospective adjustment."""

DRAFT_TASK = """Your task: build and test hypotheses that could explain how the dossiers are treated, and commit those
you have reason to expect to hold. The evaluator contrasts your committed hypotheses with the documented
positions through the loss."""


def instructions(condition: str, budgets: dict, n_dossiers: int, question_first: bool = False,
                 draft_contract: bool = False) -> str:
    feedback = ("After each scoring you receive the list of discrepancies for that dossier: documented "
                "positions your committed theory did not derive, and excluded positions it derived with the "
                "hypotheses responsible." if condition == "feedback" else
                "After each scoring you are only told that the dossier was evaluated.")
    question = ""
    if question_first:
        question = ("\n  In explore, first pose an investigable question (pose_question) and name the tool that "
                    "will answer it,\n  then consult Residuum with a navigation tool or evaluate_rule; in light "
                    "of the result, propose a\n  hypothesis or state an abstention. Concluding that the evidence "
                    "is insufficient is a valid outcome.")
    navigation = ""
    if budgets.get("max_navigation_actions", 0) > 0:
        navigation = (f"\n\nNavigation tools give access to Residuum's own operations on the presented dossiers and "
                      f"your admitted hypotheses (theory inventory, forward and residual sessions, minimal supports, "
                      f"partition comparison). They never return valuations or loss. Budget: "
                      f"{budgets['max_navigation_actions']} navigation calls and "
                      f"{budgets['max_navigation_steps']} engine steps for the whole trajectory. Sessions do not "
                      f"survive the presentation of a new dossier.")
    text = _instructions(condition, budgets, n_dossiers, feedback, question, navigation)
    if not draft_contract:
        return text
    # Draft contract (run-006): explore, revise and task paragraphs replaced.
    start = text.index("- explore:")
    budgets_at = text.index("Budgets for the whole trajectory:")
    return (text[:start] + DRAFT_EXPLORE + "\n" + DRAFT_REVISE.format(feedback=feedback) + "\n\n"
            + DRAFT_TASK + "\n\n" + text[budgets_at:])


def _instructions(condition: str, budgets: dict, n_dossiers: int, feedback: str, question: str,
                  navigation: str) -> str:
    return f"""You are the learner in an experiment run with a formal evaluator, Residuum.

The theory LegalGeneral (Formal source below) is fixed; you cannot change it. You may add rules over its
existing verbs and constants. {n_dossiers} dossiers are presented one at a time in a fixed order. A dossier is
a set of established facts (seeds). For each dossier the evaluator holds documented normative positions
(obligation, permission, prohibition, omissionProhibition or right over subject, conduct, object and
beneficiary), marked as required or excluded. They are not shown to you.

Each dossier has two phases.
- explore: you see the dossier and the closure Residuum computes for LegalGeneral plus your committed
  hypotheses, with justifications. Investigate the relations you can observe there and propose provisional
  generalisations about them as rules, then verify their consequences by evaluating them. Do this even when
  LegalGeneral shows no discrepancy yet: the task is to explore hypotheses and test what they entail, not to
  wait for a signal of error. You may propose rules, evaluate them on presented dossiers and commit a set of
  hypotheses. When you end the phase, the committed set is scored on this dossier: loss = required
  positions not derived + excluded positions derived. That prequential score is final and is never revised.
  The phase ends only after at least one proposal in it or with a stated abstention.{question}
- revise: {feedback} You may revise, keep or discard hypotheses (propose, evaluate and commit again), also
  when no dossiers remain. The changes apply to the following dossiers; every presented dossier is also
  rescored with the new committed set, recorded separately as a retrospective adjustment.

Your task: explore and test generalisations, and keep the loss low on the dossiers you have not seen yet.

Budgets for the whole trajectory: {budgets['max_verified_evaluations']} verified evaluations, {budgets['max_proposals']}
proposals, {budgets['max_turns_per_phase']} turns per phase. Every turn is exactly one tool call.{navigation}"""


class CeilingReached(Exception):
    pass


class Ledger:
    def __init__(self, directory: Path):
        self.path = directory / "turns.jsonl"
        self.seq = 0

    def append(self, kind: str, payload) -> None:
        self.seq += 1
        with self.path.open("a", encoding="utf-8") as f:
            f.write(json.dumps({"seq": self.seq, "time": time.time(), "kind": kind, "payload": payload},
                               ensure_ascii=False) + "\n")


def api_key() -> str:
    key = os.environ.get("ANTHROPIC_API_KEY")
    if key:
        return key.strip()
    path = Path(os.environ.get("RESIDUUM_ANTHROPIC_KEY_FILE",
                               str(Path.home() / ".config" / "residuum" / "anthropic_api_key")))
    if not path.is_file():
        raise SystemExit(f"no ANTHROPIC_API_KEY and no key file at {path}")
    return path.read_text(encoding="utf-8").strip()


class AnthropicProvider:
    def __init__(self, cfg: dict, system: str, ledger: Ledger, tools: list[dict]):
        self.cfg = cfg
        self.system = system
        self.ledger = ledger
        self.tools = tools
        self.key = api_key()
        self.messages: list[dict] = []
        self.cost = 0.0
        self.requests = 0

    def _price(self, usage: dict) -> float:
        p = self.cfg["pricing"]
        return (usage.get("input_tokens", 0) * p["input_per_mtok"]
                + usage.get("cache_creation_input_tokens", 0) * p["input_per_mtok"] * p["cache_write_multiplier"]
                + usage.get("cache_read_input_tokens", 0) * p["cache_read_per_mtok"]
                + usage.get("output_tokens", 0) * p["output_per_mtok"]) / 1e6

    def _call(self) -> dict:
        if self.requests >= self.cfg["max_model_requests"] or self.cost >= self.cfg["max_cost_usd"]:
            raise CeilingReached(f"requests={self.requests} cost={self.cost:.4f}")
        # One rolling cache breakpoint on the newest message keeps the shared
        # prefix cached between turns.
        messages = json.loads(json.dumps(self.messages))
        last = messages[-1]["content"]
        if isinstance(last, list) and last:
            last[-1]["cache_control"] = {"type": "ephemeral"}
        body = {"model": self.cfg["model_id"], "max_tokens": self.cfg["max_tokens"],
                "system": [{"type": "text", "text": self.system, "cache_control": {"type": "ephemeral"}}],
                "tools": self.tools, "messages": messages}
        body.update(self.cfg.get("parameters") or {})
        headers = {"x-api-key": self.key, "anthropic-version": "2023-06-01", "content-type": "application/json"}
        # Keys not scoped to a workspace must name one.
        workspace = os.environ.get("ANTHROPIC_WORKSPACE_ID", "").strip()
        if workspace:
            headers["anthropic-workspace-id"] = workspace
        request = urllib.request.Request(API_URL, data=json.dumps(body).encode(), method="POST", headers=headers)
        self.requests += 1
        self.ledger.append("request", {"n": self.requests, "new_message": self.messages[-1]})
        try:
            with urllib.request.urlopen(request, timeout=self.cfg.get("timeout_seconds", 180)) as r:
                response = json.loads(r.read())
        except urllib.error.HTTPError as e:
            detail = e.read().decode(errors="replace")[:4000]
            self.ledger.append("http_error", {"status": e.code, "body": detail})
            raise CeilingReached(f"http {e.code}: {detail[:300]}")
        usage = response.get("usage", {})
        cost = self._price(usage)
        self.cost += cost
        self.ledger.append("response", {"n": self.requests, "response": response, "cost_usd": cost,
                                        "cumulative_cost_usd": self.cost})
        response["_cost"] = cost
        return response

    def turn(self, view_text: str, pending_tool_ids: list[str]) -> tuple[dict | None, dict]:
        if pending_tool_ids:
            content = [{"type": "tool_result", "tool_use_id": pending_tool_ids[0], "content": view_text}]
            content += [{"type": "tool_result", "tool_use_id": i,
                         "content": "Not executed: one tool call per turn."} for i in pending_tool_ids[1:]]
        else:
            content = [{"type": "text", "text": view_text}]
        self.messages.append({"role": "user", "content": content})
        usage = {"calls": 0, "input_tokens": 0, "output_tokens": 0, "cost_microusd": 0}
        for attempt in range(2):
            response = self._call()
            u = response.get("usage", {})
            usage["calls"] += 1
            usage["input_tokens"] += (u.get("input_tokens", 0) + u.get("cache_creation_input_tokens", 0)
                                      + u.get("cache_read_input_tokens", 0))
            usage["output_tokens"] += u.get("output_tokens", 0)
            usage["cost_microusd"] += round(response["_cost"] * 1e6)
            self.messages.append({"role": "assistant", "content": response["content"]})
            uses = [b for b in response["content"] if b.get("type") == "tool_use"]
            if uses:
                return {"tool": uses[0]["name"], "arguments": uses[0]["input"],
                        "_ids": [b["id"] for b in uses]}, usage
            if attempt == 0:
                self.messages.append({"role": "user", "content": [
                    {"type": "text", "text": "Protocol: every turn is exactly one tool call."}]})
        return None, usage


class ScriptedProvider:
    """Fixed tool calls in order, then end_phase forever. No network."""

    def __init__(self, script: list[dict], ledger: Ledger):
        self.script = list(script)
        self.ledger = ledger
        self.cost = 0.0
        self.requests = 0

    def turn(self, view_text: str, pending_tool_ids: list[str]):
        self.requests += 1
        action = self.script.pop(0) if self.script else {"tool": "end_phase", "arguments": {}}
        action = dict(action, _ids=[f"s{self.requests}"])
        self.ledger.append("response", {"n": self.requests, "scripted": action})
        return action, {"calls": 1, "input_tokens": 0, "output_tokens": 0, "cost_microusd": 0}


def strip_comments(source: str) -> str:
    return "\n".join(line for line in (re.sub(r"//.*$", "", l).rstrip() for l in source.splitlines()) if line)


def opening(manifest: dict, base: Path) -> str:
    d = manifest["driver"]
    ontology = (base / d["ontology"]).read_text(encoding="utf-8")
    vocabulary = strip_comments((base / d["vocabulary"]).read_text(encoding="utf-8"))
    profile = d["profile"]
    syntax = SYNTAX.format(heads=", ".join(profile["heads"]), body=", ".join(profile["body_verbs"]),
                           atoms=profile["max_atoms"], clauses=manifest["trajectory"]["budgets"]["max_clauses"])
    return (f"LegalGeneral (fixed theory):\n```\n{ontology}\n```\n\n"
            f"Instance constants available to the dossiers:\n```\n{vocabulary}\n```\n\n{syntax}\n\n")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args(argv)
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    base = args.manifest.parent
    d = manifest["driver"]
    t = manifest["trajectory"]
    args.ledger.mkdir(parents=True, exist_ok=True)
    ledger = Ledger(args.ledger)
    question_first = bool(t.get("question_first", False))
    draft_contract = bool(t.get("draft_contract", False))
    system = instructions(t["condition"], t["budgets"], len(t["order"]), question_first, draft_contract)
    first = opening(manifest, base)
    base_tools = draft_tools(TOOLS) if draft_contract else TOOLS
    tools = base_tools + ([QUESTION_TOOL] if question_first or draft_contract else []) + \
        (NAVIGATION_TOOLS if t["budgets"].get("max_navigation_actions", 0) > 0 else [])
    ledger.append("start", {"manifest": manifest, "system": system, "opening": first, "tools": tools})
    if d["provider"] == "anthropic":
        provider = AnthropicProvider(d["model"], system, ledger, tools)
    elif d["provider"] == "scripted":
        provider = ScriptedProvider(d["script"], ledger)
    else:
        raise SystemExit(f"unknown provider {d['provider']!r}")
    process = subprocess.Popen([str(args.binary), "--manifest", str(args.manifest), "--ledger", str(args.ledger)],
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    pending: list[str] = []
    ceiling: str | None = None
    report = None
    opened = False
    for line in process.stdout:
        message = json.loads(line)
        if "report" in message:
            report = message["report"]
            break
        view = message["view"]
        ledger.append("view", view)
        last = view.get("last_result")
        text = (f"Result of your last action: {last}\n\n" if last else "") + \
            "Current view:\n" + json.dumps(view, ensure_ascii=False)
        if not opened:
            text = first + text
            opened = True
        action, usage = None, {"calls": 0, "input_tokens": 0, "output_tokens": 0, "cost_microusd": 0}
        if ceiling is None:
            try:
                action, usage = provider.turn(text, pending)
            except CeilingReached as e:
                ceiling = str(e)
                ledger.append("ceiling", {"reason": ceiling})
        if action is None:
            action = {"tool": "end_phase", "arguments": {}, "_ids": []}
            ledger.append("forced_end_phase", {"reason": ceiling or "no tool call"})
        pending = action.pop("_ids", [])
        ledger.append("action", action)
        process.stdin.write(json.dumps({"action": action, "usage": usage}) + "\n")
        process.stdin.flush()
    process.stdin.close()
    code = process.wait()
    summary = {"exit": code, "ceiling": ceiling, "model_requests": provider.requests,
               "cost_usd": round(provider.cost, 6), "report_received": report is not None}
    ledger.append("end", summary)
    (args.ledger / "driver-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary))
    return 0 if code == 0 and report is not None else 4


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
