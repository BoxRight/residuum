"""Scripted policies for the simulated provider that branch on the result
just received. They choose the next action from the visible history, so a
later action can depend on a freshly computed result; they are not a fixed
list of requests. They know nothing beyond the views their condition
delivers and cannot touch the manifest.
"""
from __future__ import annotations

PURPOSEFUL = (
    "rule purposefulImpediment(will, testator, beneficiary, object, authority, effective, act, death, t) =\n"
    "    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t\n"
    "    & purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t\n"
    "    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t\n"
)


def _act(tool, rationale, expected=None, **arguments):
    arguments.update({"rationale": rationale, "expected_observation": expected})
    return {"tool": tool, "arguments": arguments}


def _last(history):
    return history[-1] if history else None


def _payload(item):
    view = item["view"] if item else None
    if not view or "response" not in view:
        return None, None
    (kind, body), = view["response"]["payload"].items()
    return kind, body


class BranchingPolicy:
    """Decides from the last view. Phases: inspect → forward on the
    historical dossier until complete → propose → evaluate until the report
    arrives → (compatible: supports in B/C, a valuation request in A;
    incompatible or technical: a valuation request) → a dossier absent from
    any recording, advanced until complete → finish."""

    def __init__(self, goal_impediment: dict, extra_dossier: str = "killingWithoutAcquisitionPurpose"):
        self.goal = goal_impediment
        self.extra_dossier = extra_dossier
        self.phase = "inspect"
        self.session = None
        self.candidate = None
        self.eval_session = None
        self.extra_session = None
        self.decisions: list[str] = []
        self.advances = 0

    def next_action(self, history, tools, cap):
        last = _last(history)
        kind, body = _payload(last)
        last_status = last["view"]["response"]["status"] if last and "response" in (last["view"] or {}) else None
        if last and last["view"] and "protocol_error" in last["view"]:
            self.decisions.append("protocol_error -> finish")
            return _act("finish_episode", "a protocol error ended the plan", candidate_ids=[], explanation="protocol error")
        if self.phase == "inspect":
            self.phase = "open_forward"
            return _act("inspect_theory", "see the theory first")
        if self.phase == "open_forward":
            self.phase = "advance_forward"
            return _act("begin_forward", "observe the historical dossier", dossier="historicalRiggs")
        if self.phase == "advance_forward":
            if kind == "session_opened":
                self.session = body["session"]
                return _act("advance_session", "first rounds", session=self.session, max_steps=2)
            if kind == "forward" and not body["complete"]:
                self.advances += 1
                self.decisions.append("forward pending -> advance again")
                return _act("advance_session", "not yet at the fixed point", session=self.session, max_steps=4)
            if kind == "forward" and body["complete"]:
                self.decisions.append("forward complete -> propose")
                self.phase = "evaluate"
                return _act("propose_candidate", "propose the purposeful impediment", source=PURPOSEFUL,
                            legal_hypothesis={"statement": "a beneficiary who kills for the inheritance acquires no entitlement",
                                              "grounds": ["Riggs v. Palmer"], "status": "hypothesis"})
        if self.phase == "evaluate":
            if kind == "candidate":
                if body["candidate"] is None:
                    self.decisions.append("candidate rejected -> finish")
                    return _act("finish_episode", "the proposal was rejected", candidate_ids=[], explanation="rejected")
                self.candidate = body["candidate"]
                return _act("begin_candidate_evaluation", "evaluate on the historical dossier",
                            candidate=self.candidate, dossier="historicalRiggs")
            if kind == "session_opened" and body["kind"] == "evaluation":
                self.eval_session = body["session"]
                return _act("advance_session", "evaluate", session=self.eval_session, max_steps=8)
            if kind == "evaluation":
                if not body["complete"]:
                    self.decisions.append("evaluation pending -> advance again")
                    return _act("advance_session", "evaluation not complete", session=self.eval_session, max_steps=8)
                status = body["report"]["status"]
                self.phase = "extra"
                if isinstance(status, dict) and "evaluated" in status and status["evaluated"]["compatible"]:
                    if "minimal_supports" in tools:
                        self.decisions.append("compatible -> minimal_supports")
                        return _act("minimal_supports", "how much evidence the candidate needs",
                                    candidate=self.candidate, goal=self.goal,
                                    claims=[{"kind": "residuum_result", "text": "the candidate is compatible",
                                             "assertion": {"ref": last["action_id"], "attribute": "compatible", "value": "true"}}])
                    self.decisions.append("compatible, A -> request valuation")
                    return _act("request_valuation", "ask for an independent valuation",
                                case_spec={"dossier": self.extra_dossier}, expected_discrimination="purpose")
                self.decisions.append("not compatible or technical -> request valuation")
                return _act("request_valuation", "the candidate did not pass; ask for a valuation",
                            case_spec={"dossier": "historicalRiggs"}, expected_discrimination="scope of D")
        if self.phase == "extra":
            if kind in ("supports",) or (last and last["tool"] == "request_valuation"):
                self.phase = "advance_extra"
                return _act("begin_forward", "a dossier no recording covers", dossier=self.extra_dossier)
        if self.phase == "advance_extra":
            if kind == "session_opened":
                self.extra_session = body["session"]
                return _act("advance_session", "close the extra dossier", session=self.extra_session, max_steps=8)
            if kind == "forward" and not body["complete"]:
                return _act("advance_session", "still open", session=self.extra_session, max_steps=8)
            if kind == "forward" and body["complete"]:
                self.decisions.append("extra dossier complete -> finish")
                return _act("finish_episode", "done", candidate_ids=[self.candidate] if self.candidate else [],
                            explanation="historical compatibility checked; extra dossier observed")
        self.decisions.append(f"unexpected view in phase {self.phase} ({kind}, {last_status}) -> finish")
        return _act("finish_episode", "unexpected state", candidate_ids=[], explanation="unexpected state")


POLICIES = {"branching": BranchingPolicy}
