"""Episode controller (invocation protocol §6, §7, §12): one model action per
turn, validated and dispatched to an injectable backend; the full result is
kept, the view of the condition is delivered; candidates are tracked against
the resolved observations; the verdict comes from certificates and
observations, never from the model's own account, and no technical failure
is ever read as normative incompatibility.
"""
from __future__ import annotations

from dataclasses import dataclass, field

from . import claims as claims_mod
from .backend import Backend, BackendError
from .dispatcher import Dispatcher, Registry
from .episode import BudgetExhausted, Budgets, EpisodeState
from .ledger import Ledger
from .manifest import EpisodeManifest, ManifestError, recheck
from .projections import ProjectionError, project
from .provider_openai import ProviderError
from .schemas import ProtocolError, catalog, validate_action

OUTCOMES = ("verified_compatible_candidate", "no_verified_candidate", "awaiting_independent_valuation",
            "budget_exhausted", "technical_abort")


@dataclass
class CandidateRecord:
    candidate: str
    proposed_by: str
    admitted: bool
    # dossier → {"status": label, "compatible": bool | None, "action_id": str}
    evaluations: dict[str, dict] = field(default_factory=dict)
    sessions: dict[str, str] = field(default_factory=dict)  # session id → dossier


class Controller:
    def __init__(self, manifest: EpisodeManifest, ledger: Ledger, backend: Backend, provider):
        self.manifest = manifest
        self.ledger = ledger
        self.backend = backend
        self.provider = provider
        self.condition = manifest.condition
        self.registry = Registry()
        self.dispatcher = Dispatcher(manifest, self.registry)
        self.state = EpisodeState()
        c = manifest.controller
        self.budgets = Budgets(c["max_model_requests"], c["max_actions"], c["max_output_tokens"])
        self.history: list[dict] = []           # what the model sees, in order
        self.records: dict[str, dict] = {}       # action_id → full backend line
        self.envelopes: dict[str, dict] = {}     # action_id → envelope as sent
        self.candidates: dict[str, CandidateRecord] = {}
        self.pending_valuations: list[dict] = []
        self.observations = {o["dossier"]: o for o in c["observations"] if o["status"] == "documented"}
        self.outcome: str | None = None
        self.finish_explanation: str | None = None
        self.turns = 0

    # -- lifecycle --------------------------------------------------------

    def preflight(self) -> None:
        self.state.to("PREFLIGHT")
        try:
            recheck(self.manifest)
        except ManifestError as error:
            self._abort(f"preflight: {error}")
            return
        self.ledger.append("episode.started", {
            "episode_id": self.manifest.episode_id, "episode_hash": self.manifest.episode_hash,
            "condition": self.condition, "tools": catalog(self.condition),
            "worker_hash": self.manifest.worker_hash, "budgets": self.budgets.snapshot(),
            "observations": list(self.observations),
        })
        self.state.to("READY")

    def run(self, max_turns: int = 1000) -> str:
        if self.state.state == "CREATED":
            self.preflight()
        while not self.state.terminal and self.turns < max_turns:
            self.turn()
        if not self.state.terminal:
            self._abort("turn limit of the harness reached")
        return self.outcome or "technical_abort"

    def turn(self) -> None:
        self.turns += 1
        try:
            recheck(self.manifest)
        except ManifestError as error:
            self._abort(f"turn {self.turns}: {error}")
            return
        try:
            self.budgets.reserve_request()
            cap = self.budgets.output_cap()
        except BudgetExhausted as error:
            self._exhaust(str(error))
            return
        tools = catalog(self.condition)
        self.ledger.append("model_request_prepared", {
            "turn": self.turns, "history_items": len(self.history), "tools": tools, "output_cap": cap,
        }, file="model-requests")
        self.state.to("MODEL_REQUESTED")
        try:
            action, usage = self.provider.next_action(list(self.history), tools, cap)
        except ProviderError as error:
            # Transport or protocol failure of the model: recorded, the request
            # budget stays charged, nothing executes, no hidden retry.
            self.ledger.append("model_error", {"turn": self.turns, "message": str(error)}, file="model-responses")
            self.state.to("READY")
            return
        self.state.to("MODEL_RECEIVED")
        self.budgets.charge_usage(usage)
        self.ledger.append("model_response", {"turn": self.turns, "action": action, "usage": usage},
                           file="model-responses")
        try:
            validated = validate_action(action, self.condition)
            if validated.worker_side:
                envelope_probe = self.dispatcher.envelope("probe", validated)
            else:
                envelope_probe = None
        except ProtocolError as error:
            self._protocol_error(str(error), action)
            return
        self.state.to("ACTION_VALIDATED")
        if validated.tool == "finish_episode":
            self._finish(validated)
            return
        if validated.tool == "request_valuation":
            self._request_valuation(validated)
            return
        try:
            self.budgets.charge_action()
        except BudgetExhausted as error:
            self.state.to("READY")
            self._exhaust(str(error))
            return
        action_id = self.ledger.next_id("a")
        envelope = dict(envelope_probe, action_id=action_id)
        self.ledger.append("action.validated", {
            "action_id": action_id, "tool": validated.tool, "request": envelope["request"],
            "rationale": validated.rationale, "expected_observation": validated.expected_observation,
            "legal_hypothesis": validated.legal_hypothesis,
        }, file="actions")
        self.envelopes[action_id] = envelope
        self.state.to("TOOL_RUNNING")
        try:
            line = self.backend.handle(envelope)
        except BackendError as error:
            self.ledger.append("action.lost", {"action_id": action_id, "error": str(error)})
            self._abort(f"backend: {error}")
            return
        self.records[action_id] = line
        artifact = self.ledger.store_artifact(_dumps(line))
        status = line["response"]["status"] if "response" in line else f"fault:{line['fault']['class']}"
        self.ledger.append("action.result", {"action_id": action_id, "status": status, "artifact": artifact},
                           file="evaluator-trace")
        self.state.to("RESULT_RECORDED")
        if "response" in line:
            self.registry.learn(line["response"])
            self._track(validated, action_id, line["response"])
        verdicts = [claims_mod.check(c, self.records, self.condition) for c in validated.claims]
        for v in verdicts:
            self.ledger.append("claim.checked", {"action_id": action_id, **v}, file="claims")
        try:
            view = project(line, self.condition)
        except ProjectionError as error:
            self._abort(f"projection: {error}")
            return
        view_artifact = self.ledger.store_artifact(_dumps(view))
        self.ledger.append("view.recorded", {"action_id": action_id, "condition": self.condition,
                                             "artifact": view_artifact}, file="visible-views")
        self.state.to("VIEW_RECORDED")
        self.history.append({"action_id": action_id, "tool": validated.tool, "arguments": validated.arguments,
                             "view": view})
        self.state.to("READY")

    def resend(self, action_id: str) -> dict:
        """Idempotence probe: the same action id and content again, against
        the same worker. Records whether the answer is identical and whether
        the worker's reported budget is unchanged; charges nothing here."""
        if action_id not in self.envelopes:
            raise ProtocolError(f"{action_id} was never sent")
        before = self.budgets.snapshot()
        try:
            line = self.backend.handle(dict(self.envelopes[action_id]))
        except BackendError as error:
            record = {"action_id": action_id, "identical": False, "budget_unchanged": None, "error": str(error)}
            self.ledger.append("action.resent", record, file="evaluator-trace")
            return record
        original = self.records[action_id]
        identical = line == original
        budget_unchanged = ("response" in line and "response" in original
                            and line["response"]["budget"] == original["response"]["budget"]
                            and line["response"]["steps_consumed"] == original["response"]["steps_consumed"])
        record = {"action_id": action_id, "identical": identical, "budget_unchanged": budget_unchanged,
                  "controller_budget_unchanged": self.budgets.snapshot() == before,
                  "artifact": self.ledger.store_artifact(_dumps(line))}
        self.ledger.append("action.resent", record, file="evaluator-trace")
        return record

    def idempotence_probe(self, dossier: str) -> dict:
        """Technical acceptance on mutable state, against the live worker,
        outside the model loop: open a forward session, advance it two
        rounds (a pending result), resend that exact envelope, then advance
        one more round with a fresh action. Passes only if the resend answers
        identically, the fresh advance starts at the next round (the session
        did not advance twice), and the worker's reported actions and steps
        moved by exactly the fresh action. Recorded in evaluator-trace; a
        failure makes the technical acceptance fail without touching the
        normative outcome already recorded."""
        from .schemas import validate_action
        record = {"dossier": dossier, "steps": [], "passed": False, "reason": None}

        def send(tool, **arguments):
            arguments.setdefault("rationale", "idempotence probe")
            arguments.setdefault("expected_observation", None)
            validated = validate_action({"tool": tool, "arguments": arguments}, "C")
            action_id = self.ledger.next_id("p")
            envelope = self.dispatcher.envelope(action_id, validated)
            line = self.backend.handle(envelope)
            if "response" in line:
                self.registry.learn(line["response"])
            record["steps"].append({"action_id": action_id, "tool": tool,
                                    "artifact": self.ledger.store_artifact(_dumps(line))})
            return envelope, line
        try:
            _, opened = send("begin_forward", dossier=dossier)
            session = opened["response"]["payload"]["session_opened"]["session"]
            first_env, first = send("advance_session", session=session, max_steps=2)
            if first["response"]["status"] != "step_limit":
                record["reason"] = f"probe needs a pending session; got {first['response']['status']}"
                return self._finish_probe(record)
            replay = self.backend.handle(dict(first_env))
            record["steps"].append({"action_id": first_env["action_id"], "tool": "replay",
                                    "artifact": self.ledger.store_artifact(_dumps(replay))})
            _, fresh = send("advance_session", session=session, max_steps=1)
            f, r, n = first["response"], replay["response"], fresh["response"]
            last_round = f["payload"]["forward"]["rounds"][-1]["round"]
            next_round = n["payload"]["forward"]["rounds"][0]["round"] if n["payload"]["forward"]["rounds"] else None
            checks = {
                "replay_identical": replay == first,
                "session_not_advanced_twice": next_round == last_round + 1,
                "no_extra_action_charged": n["budget"]["actions_remaining"] == r["budget"]["actions_remaining"] - 1,
                "no_extra_steps_charged": n["budget"]["steps_remaining"] == r["budget"]["steps_remaining"] - n["steps_consumed"]
                                          and n["steps_consumed"] == 1,
            }
            record["checks"] = checks
            record["rounds"] = {"first_last": last_round, "fresh_first": next_round}
            record["passed"] = all(checks.values())
            if not record["passed"]:
                record["reason"] = "failed: " + ", ".join(k for k, v in checks.items() if not v)
        except (BackendError, KeyError, IndexError, TypeError, ProtocolError) as error:
            record["reason"] = f"probe could not complete: {error}"
        return self._finish_probe(record)

    def _finish_probe(self, record: dict) -> dict:
        self.ledger.append("idempotence.probe", record, file="evaluator-trace")
        return record

    # -- tracking ---------------------------------------------------------

    def _track(self, validated, action_id: str, response: dict) -> None:
        (kind, body), = response["payload"].items()
        if kind == "candidate":
            cid = body["candidate"]
            report = body["report"]
            status_key = next(iter(report["status"])) if isinstance(report["status"], dict) else report["status"]
            if cid:
                self.candidates[cid] = CandidateRecord(cid, action_id, True)
                self.ledger.append("candidate.generated", {
                    "candidate": cid, "action_id": action_id, "stages": report["stages"],
                    "rules": [r["name"] for r in report["rules"]],
                }, file="candidates")
            else:
                self.ledger.append("candidate.rejected", {
                    "action_id": action_id, "status": status_key, "stages": report["stages"],
                    "original_text": report["original_text"],
                }, file="candidates")
        elif kind == "session_opened" and body["kind"] == "evaluation":
            cid = validated.arguments["candidate"]
            self.candidates[cid].sessions[body["session"]] = validated.arguments["dossier"]
        elif kind == "evaluation":
            cid, dossier = body["candidate"], body["dossier"]
            record = self.candidates.get(cid)
            if record is None:
                return
            if not body["complete"] or body["report"] is None:
                record.evaluations[dossier] = {"status": "pending", "compatible": None, "action_id": action_id}
                return
            report = body["report"]
            status_key = next(iter(report["status"]))
            if status_key == "evaluated":
                compatible = report["status"]["evaluated"]["compatible"]
                record.evaluations[dossier] = {"status": "evaluated", "compatible": compatible, "action_id": action_id}
                observation = self.observations.get(dossier)
                if observation is not None:
                    satisfied = compatible == observation["required"]
                    self.ledger.append("filter.applied", {
                        "candidate": cid, "dossier": dossier, "observation_source": observation["source"],
                        "compatible": compatible, "required": observation["required"],
                        "satisfied": satisfied, "action_id": action_id,
                    }, file="progress")
            else:
                # certificate_failed, evaluation_pending, out_of_certificate_scope:
                # technical statuses, never incompatibility.
                record.evaluations[dossier] = {"status": status_key, "compatible": None, "action_id": action_id}
                self.ledger.append("evaluation.technical", {"candidate": cid, "dossier": dossier,
                                                            "status": status_key, "action_id": action_id},
                                   file="progress")
        elif kind == "partition":
            self._observe_partition(action_id, body)

    def _observe_partition(self, action_id: str, body: dict) -> None:
        """Every partition result is observed; it is a refinement only when
        it has the same carrier in the same order as the previous observed
        partition, is finer, and actually differs. A partition over new or
        reordered candidates changes the universe and is recorded as such.
        The kernels themselves come from the worker; this is bookkeeping."""
        hypotheses = list(body["partition"]["hypotheses"])
        kernel = list(body["partition"]["kernel"])
        observed = {
            "action_id": action_id, "hypotheses": hypotheses, "kernel": kernel,
            "classes": len(set(kernel)), "discriminants": len(body["discriminants"]),
        }
        self.ledger.append("partition.observed", observed, file="partitions")
        previous = getattr(self, "_last_partition", None)
        self._last_partition = observed
        if previous is None:
            return
        if previous["hypotheses"] != hypotheses:
            self.ledger.append("partition.carrier_changed", {
                "action_id": action_id, "previous_action": previous["action_id"],
                "previous_hypotheses": previous["hypotheses"], "hypotheses": hypotheses,
            }, file="partitions")
            return
        same = all((kernel[i] == kernel[j]) == (previous["kernel"][i] == previous["kernel"][j])
                   for i in range(len(kernel)) for j in range(len(kernel)))
        finer = all(kernel[i] != kernel[j] or previous["kernel"][i] == previous["kernel"][j]
                    for i in range(len(kernel)) for j in range(len(kernel)))
        if same:
            self.ledger.append("partition.unchanged", {"action_id": action_id,
                                                       "previous_action": previous["action_id"]},
                               file="partitions")
        elif finer:
            self.ledger.append("partition.refined", {
                "action_id": action_id, "previous_action": previous["action_id"],
                "classes_before": previous["classes"], "classes_after": observed["classes"],
            }, file="partitions")
        else:
            self.ledger.append("partition.not_a_refinement", {
                "action_id": action_id, "previous_action": previous["action_id"],
            }, file="partitions")

    def _request_valuation(self, validated) -> None:
        vid = self.ledger.next_id("v")
        entry = {"valuation_id": vid, "case_spec": validated.arguments["case_spec"],
                 "expected_discrimination": validated.arguments["expected_discrimination"],
                 "status": "pending", "source": None}
        self.pending_valuations.append(entry)
        self.ledger.append("valuation.requested", entry, file="valuations")
        self.state.to("TOOL_RUNNING"); self.state.to("RESULT_RECORDED"); self.state.to("VIEW_RECORDED")
        self.history.append({"action_id": vid, "tool": "request_valuation", "arguments": validated.arguments,
                             "view": {"valuation": {"id": vid, "status": "pending"}}})
        self.state.to("READY")
        self.state.to("WAITING_VALUATION")
        self.state.to("READY")

    def _protocol_error(self, message: str, action) -> None:
        try:
            self.budgets.charge_action()
        except BudgetExhausted:
            pass
        self.ledger.append("protocol_error", {"turn": self.turns, "message": message, "action": action},
                           file="actions")
        self.history.append({"action_id": None, "tool": None, "arguments": None,
                             "view": {"protocol_error": message}})
        self.state.to("READY")

    # -- end of episode ---------------------------------------------------

    def classify_candidate(self, cid: str) -> str:
        record = self.candidates.get(cid)
        if record is None or not record.admitted:
            return "unknown"
        if not self.observations:
            return "no_resolved_observation"
        verdict = "verified"
        for dossier, observation in self.observations.items():
            ev = record.evaluations.get(dossier)
            if ev is None or ev["status"] != "evaluated":
                verdict = "pending"
                continue
            if ev["compatible"] != observation["required"]:
                return "incompatible"
        return verdict

    def _finish(self, validated) -> None:
        self.finish_explanation = validated.arguments["explanation"]
        referenced = validated.arguments["candidate_ids"]
        classes = {cid: self.classify_candidate(cid) for cid in referenced}
        if any(v == "verified" for v in classes.values()):
            outcome = "verified_compatible_candidate"
        elif any(v == "pending" for v in classes.values()) or self.pending_valuations:
            outcome = "awaiting_independent_valuation"
        else:
            outcome = "no_verified_candidate"
        self.outcome = outcome
        self.ledger.append("episode.finished", {
            "outcome": outcome, "referenced": classes, "explanation": self.finish_explanation,
            "candidates": {cid: {"evaluations": r.evaluations} for cid, r in self.candidates.items()},
            "pending_valuations": len(self.pending_valuations), "budgets": self.budgets.snapshot(),
            "scope": "compatibility with the declared D and resolved observations; not a legal approval",
        }, file="progress")
        self.state.to("TOOL_RUNNING"); self.state.to("RESULT_RECORDED"); self.state.to("VIEW_RECORDED")
        self.state.to("READY"); self.state.to("FINISHED")

    def _exhaust(self, message: str) -> None:
        self.outcome = "budget_exhausted"
        self.ledger.append("episode.budget_exhausted", {"message": message, "budgets": self.budgets.snapshot()},
                           file="progress")
        self.state.to("BUDGET_EXHAUSTED")

    def _abort(self, message: str) -> None:
        self.outcome = "technical_abort"
        self.ledger.append("episode.aborted", {"message": message}, file="progress")
        if self.state.state in ("CREATED",):
            self.state.to("PREFLIGHT")
        if self.state.state in ("MODEL_REQUESTED", "MODEL_RECEIVED", "TOOL_RUNNING", "PREFLIGHT", "READY"):
            self.state.to("ABORTED")
        else:
            self.state.state = "ABORTED"
            self.state.history.append("ABORTED")


def _dumps(value) -> str:
    import json
    return json.dumps(value, sort_keys=True, ensure_ascii=False)
