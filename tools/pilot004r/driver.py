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
ATTRIBUTION_RETURNS = ("Returns certificate status, its baseline (LegalGeneral plus the other committed "
                       "hypotheses), the events it adds to that baseline, the justifications it contributes "
                       "(each with its rule and the hypothesis that owns it, also on events the baseline "
                       "already derives) and the positions that depend on its rules; never valuations or loss.")


def attribution_tools(tools: list[dict]) -> list[dict]:
    """Run-011: evaluate_rule describes its result against the stated baseline."""
    out = json.loads(json.dumps(tools))
    for tool in out:
        if tool["name"] == "evaluate_rule":
            head, sep, _ = tool["description"].partition("Returns ")
            if not sep:
                raise SystemExit("attribution: unexpected evaluate_rule description")
            tool["description"] = head + ATTRIBUTION_RETURNS
    return out


def auto_evaluation_tools(tools: list[dict]) -> list[dict]:
    """Run-010: the propose_rule description says that an admitted draft is evaluated at once."""
    out = json.loads(json.dumps(tools))
    for tool in out:
        if tool["name"] == "propose_rule":
            tool["description"] += AUTO_EVALUATION_TOOL
    return out


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


INVESTIGATION_PARAGRAPH = """
  During explore, judge hypotheses by how much they can tell you about formal consequences and differences.
  The normative loss does not apply to drafts or to their evaluations. Decide separately, at the end of the
  phase, whether to commit any hypothesis; you may commit none. Uncertainty about whether a hypothesis is
  correct is a reason to test it, not a reason to skip the test."""

DRAFT_EXPLORE_END = "you are never required to produce a rule."

AUTO_EVALUATION_SENTENCE = ("evaluate_rule tests a hypothesis hypothetically on a presented dossier, without "
                            "penalty.")
AUTO_EVALUATION_PARAGRAPH = """
  Each draft that Residuum admits is evaluated at once, automatically, on the current dossier (one
  verified evaluation from the budget), and that evaluation is returned with the admission."""
AUTO_EVALUATION_TOOL = (" When the draft is admitted, Residuum evaluates it at once on the current dossier "
                        "(one verified evaluation from the budget) and returns that evaluation.")
ADMITTED = re.compile(r"^(h\d+): admitted$")


def instructions(condition: str, budgets: dict, n_dossiers: int, question_first: bool = False,
                 draft_contract: bool = False, findings_report: bool = False,
                 investigation_split: bool = False, auto_evaluate: bool = False) -> str:
    text = _contract(condition, budgets, n_dossiers, question_first, draft_contract)
    if auto_evaluate:
        # Run-010: every admitted draft is evaluated by the harness; only with the draft contract.
        if not draft_contract or " ".join(text.split()).count(AUTO_EVALUATION_SENTENCE) != 1:
            raise SystemExit("auto_evaluate needs the draft contract")
        head, tail = text.split("evaluate_rule tests a hypothesis", 1)
        end = tail.index("without penalty.") + len("without penalty.")
        text = head + "evaluate_rule tests a hypothesis" + tail[:end] + AUTO_EVALUATION_PARAGRAPH + tail[end:]
    if investigation_split:
        # Run-009: investigating and committing are separate decisions; only with the draft contract.
        if not draft_contract or text.count(DRAFT_EXPLORE_END) != 1:
            raise SystemExit("investigation_split needs the draft contract")
        text = text.replace(DRAFT_EXPLORE_END, DRAFT_EXPLORE_END + INVESTIGATION_PARAGRAPH)
    return text + FINDINGS_PARAGRAPH if findings_report else text


def _contract(condition: str, budgets: dict, n_dossiers: int, question_first: bool,
              draft_contract: bool) -> str:
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

    def _call(self, extra: dict | None = None) -> dict:
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
        body.update(extra or {})
        headers = {"x-api-key": self.key, "anthropic-version": "2023-06-01", "content-type": "application/json"}
        # Keys not scoped to a workspace must name one.
        workspace = os.environ.get("ANTHROPIC_WORKSPACE_ID", "").strip()
        if workspace:
            headers["anthropic-workspace-id"] = workspace
        request = urllib.request.Request(API_URL, data=json.dumps(body).encode(), method="POST", headers=headers)
        self.requests += 1
        entry = {"n": self.requests, "new_message": self.messages[-1]}
        if extra:
            entry["request_options"] = extra
        self.ledger.append("request", entry)
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

    def plan(self, view_text: str, pending_tool_ids: list[str]) -> tuple[str, dict]:
        """Planning step: the same view, one planning request appended, tool calls disabled. The plan stays in
        the conversation, so the execution turns that follow receive it."""
        content = self._content(view_text, pending_tool_ids) + [{"type": "text", "text": PLAN_REQUEST}]
        self.messages.append({"role": "user", "content": content})
        response = self._call({"tool_choice": {"type": "none"}})
        self.messages.append({"role": "assistant", "content": response["content"]})
        u = response.get("usage", {})
        usage = {"calls": 1, "input_tokens": u.get("input_tokens", 0) + u.get("cache_creation_input_tokens", 0)
                 + u.get("cache_read_input_tokens", 0), "output_tokens": u.get("output_tokens", 0),
                 "cost_microusd": round(response["_cost"] * 1e6)}
        text = "\n".join(b.get("text", "") for b in response["content"] if b.get("type") == "text")
        return text, usage

    @staticmethod
    def _content(view_text: str, pending_tool_ids: list[str]) -> list[dict]:
        if pending_tool_ids:
            content = [{"type": "tool_result", "tool_use_id": pending_tool_ids[0], "content": view_text}]
            content += [{"type": "tool_result", "tool_use_id": i,
                         "content": "Not executed: one tool call per turn."} for i in pending_tool_ids[1:]]
            return content
        return [{"type": "text", "text": view_text}]

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

    def plan(self, view_text: str, pending_tool_ids: list[str]) -> tuple[str, dict]:
        self.requests += 1
        entry = self.script.pop(0) if self.script and "plan" in self.script[0] else {"plan": {}}
        text = json.dumps(entry["plan"])
        self.ledger.append("response", {"n": self.requests, "scripted_plan": entry["plan"]})
        return text, {"calls": 1, "input_tokens": 0, "output_tokens": 0, "cost_microusd": 0}

    def turn(self, view_text: str, pending_tool_ids: list[str]):
        self.requests += 1
        action = self.script.pop(0) if self.script else {"tool": "end_phase", "arguments": {}}
        action = dict(action, _ids=[f"s{self.requests}"])
        self.ledger.append("response", {"n": self.requests, "scripted": action})
        return action, {"calls": 1, "input_tokens": 0, "output_tokens": 0, "cost_microusd": 0}


PLAN_REQUEST = (
    "Planning step: no action is taken now. Before this explore phase, write a provisional investigation plan "
    "from the information in the current view. Reply with one JSON object with these fields: "
    "\"uncertainties\" (the uncertainties or relations you consider investigable now), "
    "\"operations\" (a list of {\"tool\", \"purpose\"} with the available tools that could inform them), "
    "\"distinguishing_results\" (results that would tell alternatives apart), and "
    "\"conditions\" (when you would continue, change the plan or abstain). The plan does not change the "
    "committed set, is not scored and does not oblige you to commit anything. Do not call any tool now.")

EXECUTE_REQUEST = ("Your plan is recorded. Now carry out the explore phase with the tools. You may follow the "
                   "plan, change it or abandon it in light of the results.")


def parse_plan(text: str):
    """The first JSON object in the plan text, or None; the raw text is always kept."""
    start, end = text.find("{"), text.rfind("}")
    if start < 0 or end <= start:
        return None
    try:
        value = json.loads(text[start:end + 1])
    except json.JSONDecodeError:
        return None
    return value if isinstance(value, dict) else None


# The harness's evaluate result: "<hypothesis> on <dossier>: <status>".
EVALUATION_RESULT = re.compile(r"^h\d+ on \S+: (.*)$")


class PlanTrace:
    """Planned against executed actions of one explore phase. A deviation is an executed tool absent from the
    plan's operations; what preceded it (the last result and navigation reference in the view) is recorded as
    an observable antecedent, not as a motive. A consultation has a result when it is an evaluation or a
    navigation response other than an opened session or an error."""

    def __init__(self, dossier: str, plan: dict | None):
        self.dossier = dossier
        operations = (plan or {}).get("operations") or []
        self.planned = [o.get("tool") for o in operations if isinstance(o, dict) and o.get("tool")]
        self.executed: list[str] = []
        self.deviations: list[dict] = []
        self.consultations_with_result = 0
        self.consultations_without_result = 0
        self.automatic_evaluations = 0

    def action(self, tool: str, view: dict) -> None:
        self.executed.append(tool)
        if tool not in self.planned and tool != "end_phase":
            nav = view.get("last_navigation") or {}
            self.deviations.append({"tool": tool, "preceded_by_result": view.get("last_result"),
                                    "preceded_by_navigation": nav.get("action_id")})

    def result(self, view: dict) -> None:
        nav = view.get("last_navigation")
        if nav is not None:
            kind = next(iter(nav.get("payload") or {}), None)
            if kind in ("session_opened", "error") or nav.get("status") == "error":
                self.consultations_without_result += 1
            else:
                self.consultations_with_result += 1
            return
        evaluated = EVALUATION_RESULT.match(view.get("last_result") or "")
        if evaluated:
            if evaluated.group(1).startswith("pending"):
                self.consultations_without_result += 1
            else:
                self.consultations_with_result += 1

    def summary(self) -> dict:
        return {"dossier": self.dossier, "planned_tools": self.planned, "executed_tools": self.executed,
                "planned_not_executed": [t for t in self.planned if t not in self.executed],
                "executed_not_planned": sorted({t for t in self.executed if t not in self.planned} - {"end_phase"}),
                "deviations": self.deviations,
                "consultations_with_result": self.consultations_with_result,
                "consultations_without_result": self.consultations_without_result,
                **({"automatic_evaluations": self.automatic_evaluations} if self.automatic_evaluations else {})}


FINDINGS_PARAGRAPH = """

Findings: when you end a phase you may report findings about the space of hypotheses (end_phase, field
findings): what you established about relations, consequences or differences between hypotheses. Each
finding may carry an assertion that cites a record by its reference (n... for a navigation result, e... for
an evaluation) and is checked against that record by Residuum's existing claims verifier. Findings are
kept with their verdicts and shown in later views. They are separate from drafts and from the committed
set, they are never scored, and an empty report is valid."""


def findings_tools(tools: list[dict]) -> list[dict]:
    """end_phase with an optional findings report in the claims format of tools/normative_explorer."""
    from tools.normative_explorer.schemas import CLAIM_ATTRIBUTES, CLAIM_KINDS, CLAIMS_MAX
    out = json.loads(json.dumps(tools))
    for tool in out:
        if tool["name"] == "end_phase":
            tool["input_schema"]["properties"]["findings"] = {
                "type": "array", "maxItems": CLAIMS_MAX,
                "description": "Optional report of findings (it may be empty). Each item: kind, text and "
                               "assertion (null, or {ref, attribute, value}).",
                "items": {"type": "object", "properties": {
                    "kind": {"type": "string", "enum": list(CLAIM_KINDS)},
                    "text": {"type": "string"},
                    "assertion": {"type": ["object", "null"], "properties": {
                        "ref": {"type": "string", "description": "n... or e... reference of a record."},
                        "attribute": {"type": "string", "enum": list(CLAIM_ATTRIBUTES)},
                        "value": {"type": "string"}}}},
                    "required": ["kind", "text", "assertion"]},
            }
    return out


class Findings:
    """Records seen so far and the per-phase context needed to check a report with the existing verifier
    (tools/normative_explorer/claims.py, condition C: provenance and partitions are delivered here). Novelty is
    a diagnostic, never a certified property."""

    def __init__(self):
        self.records: dict[str, dict] = {}
        self.meta: dict[str, dict] = {}
        self.sessions: dict[str, str] = {}
        self.phase_key = None
        self.phase_start_verbs: dict[str, set] = {}
        self.last_action: dict | None = None

    def observe(self, message: dict) -> None:
        view = message["view"]
        key = (view["dossier"], view["phase"])
        if key != self.phase_key:
            self.phase_key = key
            self.phase_start_verbs = {d["id"]: {e.split("(", 1)[0].lstrip("~")
                                                for e in d["closure_of_O_plus_committed"]}
                                      for d in view["dossiers"]}
        args = (self.last_action or {}).get("arguments", {})
        for record in message.get("records", []):
            ref, line = record["ref"], record["line"]
            dossier = record.get("dossier")
            payload = (line.get("response") or {}).get("payload") or {}
            if dossier is None:
                if "session_opened" in payload:
                    dossier = args.get("dossier")
                    self.sessions[payload["session_opened"]["session"]] = dossier
                elif args.get("session"):
                    dossier = self.sessions.get(args["session"])
                else:
                    dossier = args.get("dossier")
            self.records[ref] = line
            self.meta[ref] = {"dossier": dossier, "phase": list(key)}

    def check(self, raw) -> list[dict]:
        from tools.normative_explorer import claims
        from tools.normative_explorer.schemas import ProtocolError, _check
        if raw is None:
            raw = []
        try:
            _check("claims", raw, "findings")
        except ProtocolError as e:
            return [{"kind": "invalid", "text": json.dumps(raw, ensure_ascii=False)[:300], "reference": None,
                     "assertion": None, "verdict": "malformed", "detail": str(e), "novelty": "not_assessed"}]
        out = []
        for claim in raw:
            verdict = claims.check(claim, self.records, "C")
            assertion = claim.get("assertion")
            ref = assertion["ref"] if assertion else None
            out.append({"kind": claim["kind"], "text": claim["text"], "reference": ref,
                        "assertion": f"{assertion['attribute']}={assertion['value']}" if assertion else None,
                        "verdict": verdict["verdict"], "detail": verdict["detail"],
                        "novelty": self.novelty(assertion, verdict["verdict"])})
        return out

    def novelty(self, assertion, verdict: str) -> str:
        if not assertion or assertion["ref"] not in self.meta:
            return "not_assessed"
        meta = self.meta[assertion["ref"]]
        when = "reference_from_this_phase" if tuple(meta["phase"]) == self.phase_key else "reference_from_earlier"
        if assertion["attribute"] == "contains_event" and verdict == "match" and meta["dossier"] is not None:
            shown = self.phase_start_verbs.get(meta["dossier"])
            if shown is not None:
                seen = assertion["value"] in shown
                return f"{when}; content_{'already_in' if seen else 'not_in'}_view_at_phase_start"
        return f"{when}; content_not_assessed"


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
    findings_report = bool(t.get("findings_report", False))
    investigation_split = bool(t.get("investigation_split", False))
    auto_evaluate = bool(t.get("auto_evaluate", False))
    attribution = bool(t.get("attribution", False))
    system = instructions(t["condition"], t["budgets"], len(t["order"]), question_first, draft_contract,
                          findings_report, investigation_split, auto_evaluate)
    first = opening(manifest, base)
    base_tools = draft_tools(TOOLS) if draft_contract else TOOLS
    if auto_evaluate:
        base_tools = auto_evaluation_tools(base_tools)
    if attribution:
        base_tools = attribution_tools(base_tools)
    if findings_report:
        base_tools = findings_tools(base_tools)
    tools = base_tools + ([QUESTION_TOOL] if question_first or draft_contract else []) + \
        (NAVIGATION_TOOLS if t["budgets"].get("max_navigation_actions", 0) > 0 else [])
    memory = Findings() if findings_report else None
    planning = bool(t.get("planning", False))
    planned_phase = None
    trace: PlanTrace | None = None
    carried_usage = None
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
    last_tool: str | None = None
    admission: str | None = None
    for line in process.stdout:
        message = json.loads(line)
        if "report" in message:
            report = message["report"]
            break
        view = message["view"]
        ledger.append("view", view)
        if memory is not None:
            if message.get("records"):
                ledger.append("records", message["records"])
            memory.observe(message)
        last = view.get("last_result")
        if admission is not None:
            # The view after the harness's own evaluation: report admission and evaluation together.
            last = f"{admission}\nAutomatic evaluation: {last}"
            ledger.append("automatic_result", {"result": last})
            admission = None
            automatic_result = True
        else:
            automatic_result = False
        if (auto_evaluate and ceiling is None and last_tool == "propose_rule"
                and ADMITTED.match(last or "")):
            # Run-010: evaluate the admitted draft at once on the current dossier, with no model call.
            admission = last
            last_tool = None
            action = {"tool": "evaluate_rule",
                      "arguments": {"hypothesis": ADMITTED.match(last).group(1), "dossier": view["dossier"]}}
            if memory is not None:
                memory.last_action = action
            ledger.append("automatic_action", action)
            process.stdin.write(json.dumps({"action": action, "usage": {
                "calls": 0, "input_tokens": 0, "output_tokens": 0, "cost_microusd": 0}}) + "\n")
            process.stdin.flush()
            continue
        text = (f"Result of your last action: {last}\n\n" if last else "") + \
            "Current view:\n" + json.dumps(view, ensure_ascii=False)
        if not opened:
            text = first + text
            opened = True
        key = (view["dossier"], view["phase"])
        if trace is not None:
            if key == (trace.dossier, "explore"):
                if automatic_result:
                    trace.automatic_evaluations += 1
                else:
                    trace.result(view)
            else:
                ledger.append("plan_trace", trace.summary())
                trace = None
        if planning and view["phase"] == "explore" and key != planned_phase and ceiling is None:
            # Planning step before the explore phase: the same view, no action, not scored.
            planned_phase = key
            try:
                plan_text, carried_usage = provider.plan(text, pending)
                pending = []
                plan = parse_plan(plan_text)
                ledger.append("plan", {"dossier": view["dossier"], "text": plan_text, "parsed": plan})
                trace = PlanTrace(view["dossier"], plan)
                text = EXECUTE_REQUEST
            except CeilingReached as e:
                ceiling = str(e)
                ledger.append("ceiling", {"reason": ceiling})
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
        if carried_usage is not None:
            # The planning call is charged with the first execution turn of the phase.
            usage = {k: usage.get(k, 0) + carried_usage.get(k, 0) for k in usage}
            carried_usage = None
        if trace is not None:
            trace.action(action.get("tool", ""), view)
        pending = action.pop("_ids", [])
        last_tool = action.get("tool")
        if memory is not None and action.get("tool") == "end_phase":
            arguments = dict(action.get("arguments") or {})
            if "findings" in arguments:
                # An explicit report, possibly empty; an omitted field is no report.
                checked = memory.check(arguments.pop("findings"))
                arguments["findings_checked"] = checked
                ledger.append("findings", {"phase": list(memory.phase_key), "checked": checked})
            else:
                ledger.append("findings", {"phase": list(memory.phase_key), "report": "absent"})
            action = {"tool": "end_phase", "arguments": arguments}
        if memory is not None:
            memory.last_action = action
        ledger.append("action", action)
        process.stdin.write(json.dumps({"action": action, "usage": usage}) + "\n")
        process.stdin.flush()
    if trace is not None:
        ledger.append("plan_trace", trace.summary())
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
