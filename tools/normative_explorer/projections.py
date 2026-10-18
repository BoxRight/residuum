"""Views A/B/C as explicit field allow-lists over the full audit record
(contract §4.1, protocol §10). The full line from the backend is kept in the
ledger; the explorer receives only what its condition authorises. Nothing
here hides by instruction or by omission in a prompt: a field absent from
the allow-list of a condition is never emitted.

A: queries, results and states. B: adds provenance (justifications, proofs,
hypothetical closures). C: adds coverage, partitions and discriminants.
Paths of the auditor's artefacts and the composed private module source
never reach any condition.
"""
from __future__ import annotations

from .schemas import ProtocolError

NEVER = {"file", "source"}


class ProjectionError(RuntimeError):
    pass


def _events(xs):
    return list(xs)


def _rounds(rounds, with_provenance: bool):
    out = []
    for r in rounds:
        item = {"round": r["round"], "added": _events(r["added"]), "complete": r["complete"]}
        if with_provenance:
            item["derivations"] = r["derivations"]
            item["witnesses"] = r["witnesses"]
        out.append(item)
    return out


def _residual_rounds(rounds, with_provenance: bool):
    out = []
    for r in rounds:
        item = {"round": r["round"], "pending": _events(r["pending"]), "metrics": r["metrics"],
                "complete": r["complete"]}
        if with_provenance:
            item["branches"] = r["branches"]
        out.append(item)
    return out


def _certificate(cert, with_provenance: bool):
    if with_provenance:
        return dict(cert)
    return {"holds": bool(cert["events_preserved"] and cert["proofs_preserved"] and not cert["foreign_heads"])}


def _report(report, with_provenance: bool):
    status = report["status"]
    if isinstance(status, dict):
        (kind, body), = status.items()
        if kind == "evaluated":
            body = {"components": body["components"], "compatible": body["compatible"],
                    "certificate": _certificate(body["certificate"], with_provenance)}
        elif kind == "certificate_failed":
            body = _certificate(body, with_provenance)
        status = {kind: body}
    return {
        "id": report["id"], "module_name": report["module_name"],
        "original_text": report["original_text"], "rules": report["rules"],
        "stages": report["stages"], "status": status,
    }


def _result(result, with_provenance: bool):
    if result is None:
        return None
    (kind, q), = result.items()
    out = {"goal": q["goal"], "required": q["required"], "metrics": q["metrics"],
           "failure": q["failure"], "explanations_count": len(q["explanations"])}
    if with_provenance:
        out["explanations"] = q["explanations"]
    return {kind: out}


def project(line: dict, condition: str) -> dict:
    """Project one backend line ({"response": ...} or {"fault": ...})."""
    if condition not in ("A", "B", "C"):
        raise ProjectionError(f"unknown condition {condition}")
    provenance = condition in ("B", "C")
    geometry = condition == "C"
    supports = condition in ("B", "C")
    if "fault" in line:
        f = line["fault"]
        return {"fault": {"class": f["class"], "action_id": f["action_id"], "message": f["message"]}}
    r = line["response"]
    (kind, p), = r["payload"].items()
    if kind == "theory":
        body = dict(p)
    elif kind == "session_opened":
        body = dict(p)
    elif kind == "forward":
        body = {"session": p["session"], "rounds": _rounds(p["rounds"], provenance),
                "known": _events(p["known"]), "complete": p["complete"]}
    elif kind == "residual":
        body = {"session": p["session"], "kind": p["kind"], "complete": p["complete"],
                "rounds": _residual_rounds(p["rounds"], provenance),
                "result": _result(p["result"], provenance), "error": p["error"]}
    elif kind == "candidate":
        body = {"candidate": p["candidate"], "report": _report(p["report"], provenance)}
    elif kind == "evaluation":
        body = {"session": p["session"], "candidate": p["candidate"], "dossier": p["dossier"],
                "rounds": _rounds(p["rounds"], provenance), "complete": p["complete"],
                "report": _report(p["report"], provenance) if p["report"] else None}
        if provenance:
            body["closure"] = p["closure"]
    elif kind == "supports":
        if not supports:
            raise ProjectionError("minimal supports are not visible in condition A")
        body = dict(p)
    elif kind == "partition":
        if not geometry:
            raise ProjectionError("partitions are not visible outside condition C")
        body = dict(p)
    elif kind == "error":
        body = dict(p)
    else:
        raise ProjectionError(f"unknown payload kind {kind}")
    _assert_clean(body)
    return {"response": {"action_id": r["action_id"], "status": r["status"],
                         "steps_consumed": r["steps_consumed"], "budget": r["budget"],
                         "payload": {kind: body}}}


def _assert_clean(value) -> None:
    if isinstance(value, dict):
        for k, v in value.items():
            if k in NEVER:
                raise ProjectionError(f"reserved field {k} would be emitted")
            _assert_clean(v)
    elif isinstance(value, list):
        for v in value:
            _assert_clean(v)


RESERVED_A = {"derivations", "witnesses", "branches", "explanations", "hypothetical", "closure",
              "frontier", "partition", "signatures", "discriminants", "covers", "pool",
              "events_preserved", "proofs_preserved", "added_heads", "foreign_heads"} | NEVER
# B receives minimal supports (frontiers); only C receives partitions.
RESERVED_B = {"partition", "signatures", "discriminants", "covers", "pool"} | NEVER


def keys_of(value, out=None):
    out = set() if out is None else out
    if isinstance(value, dict):
        for k, v in value.items():
            out.add(k)
            keys_of(v, out)
    elif isinstance(value, list):
        for v in value:
            keys_of(v, out)
    return out


__all__ = ["project", "ProjectionError", "ProtocolError", "RESERVED_A", "RESERVED_B", "keys_of"]
