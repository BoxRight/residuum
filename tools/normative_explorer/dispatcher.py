"""Dispatcher: from a validated action to a worker envelope, after checking
references against what this episode has actually created, and tool
permission against the condition. The dispatcher never executes anything
and never inspects results semantically.
"""
from __future__ import annotations

from dataclasses import dataclass, field

from .manifest import PROTOCOL_VERSION, EpisodeManifest
from .schemas import ProtocolError, ValidatedAction, catalog


@dataclass
class Registry:
    """Identifiers the worker assigned, as the controller learned them from
    results. A reference to anything else is a protocol error."""
    sessions: dict[str, dict] = field(default_factory=dict)
    candidates: dict[str, dict] = field(default_factory=dict)
    admitted: set[str] = field(default_factory=set)

    def learn(self, response: dict) -> None:
        payload = response.get("payload", {})
        if "session_opened" in payload:
            p = payload["session_opened"]
            self.sessions[p["session"]] = {"kind": p["kind"]}
        if "candidate" in payload and payload["candidate"].get("candidate"):
            cid = payload["candidate"]["candidate"]
            self.candidates[cid] = {"status": list(payload["candidate"]["report"]["status"])[0]
                                    if isinstance(payload["candidate"]["report"]["status"], dict)
                                    else payload["candidate"]["report"]["status"]}
            self.admitted.add(cid)


class Dispatcher:
    def __init__(self, manifest: EpisodeManifest, registry: Registry):
        self.manifest = manifest
        self.registry = registry
        self.condition = manifest.condition
        self.tools = catalog(self.condition)

    def check_references(self, action: ValidatedAction) -> None:
        a = action.arguments
        dossier = a.get("dossier")
        if dossier is not None and dossier not in self.manifest.worker["dossiers"]:
            raise ProtocolError(f"unknown dossier {dossier!r}")
        theory = a.get("theory")
        if theory is not None and theory not in self.registry.admitted:
            raise ProtocolError(f"theory {theory!r} is not an admitted candidate of this episode")
        session = a.get("session")
        if session is not None and session not in self.registry.sessions:
            raise ProtocolError(f"unknown session {session!r}")
        candidate = a.get("candidate")
        if candidate is not None and candidate not in self.registry.admitted:
            raise ProtocolError(f"candidate {candidate!r} is not admitted in this episode")
        for cid in a.get("candidates", []) or []:
            if cid not in self.registry.admitted:
                raise ProtocolError(f"candidate {cid!r} is not admitted in this episode")
        parent = a.get("parent")
        if parent is not None and parent not in self.registry.candidates:
            raise ProtocolError(f"parent {parent!r} is not a candidate of this episode")

    def to_request(self, action: ValidatedAction) -> dict:
        """The wire request object (boundary/src/wire.rs, WireRequest)."""
        a = action.arguments
        t = action.tool
        if t == "inspect_theory":
            return {"inspect_theory": None}
        if t == "begin_forward":
            return {"begin_forward": {"dossier": a["dossier"], "theory": a.get("theory")}}
        if t in ("begin_residual", "begin_active_residual"):
            return {t: {
                "dossier": a["dossier"], "theory": a.get("theory"), "goal": a["goal"],
                "scope": a.get("scope"), "bindings": a.get("bindings") or [],
                "allowed": a.get("allowed"), "limits": a.get("limits"),
            }}
        if t == "advance_session":
            return {"advance_session": {"session": a["session"], "max_steps": a["max_steps"]}}
        if t == "propose_candidate":
            return {"propose_candidate": {"text": a["source"]}}
        if t == "begin_candidate_evaluation":
            return {"begin_candidate_evaluation": {"candidate": a["candidate"], "dossier": a["dossier"]}}
        if t == "minimal_supports":
            return {"minimal_supports": {"candidate": a["candidate"], "goal": a["goal"]}}
        if t == "compare_partition":
            return {"compare_partition": {"candidates": a["candidates"], "goal": a["goal"]}}
        raise ProtocolError(f"tool {t} is not executed by the worker")

    def envelope(self, action_id: str, action: ValidatedAction) -> dict:
        if action.tool not in self.tools:
            raise ProtocolError(f"tool {action.tool} is not in the catalogue of condition {self.condition}")
        self.check_references(action)
        return {
            "wire_version": "residuum-wire/1",
            "protocol_version": PROTOCOL_VERSION,
            "episode_id": self.manifest.episode_id,
            "action_id": action_id,
            "manifest_hash": self.manifest.worker_hash,
            "request": self.to_request(action),
        }
