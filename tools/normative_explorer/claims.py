"""Deterministic checking of structured claims against the full record
(explanation proposal §4). Only the assertion is checked, never the prose.
"""
from __future__ import annotations

VERDICTS = ("match", "mismatch", "unreferenced", "out_of_view", "unchecked")
RULE_VERSION = "claims/1"


def _payload(line):
    if "response" not in line:
        return None, None
    (kind, body), = line["response"]["payload"].items()
    return kind, body


def check(claim: dict, records: dict[str, dict], condition: str) -> dict:
    """records: action_id → full backend line of that action."""
    assertion = claim.get("assertion")
    if assertion is None:
        return _verdict(claim, "unchecked", "no assertion")
    ref = assertion["ref"]
    if ref not in records:
        return _verdict(claim, "unreferenced", f"{ref} is not an action of this episode")
    line = records[ref]
    attribute, value = assertion["attribute"], assertion["value"]
    kind, body = _payload(line)
    provenance = condition in ("B", "C")
    if attribute == "status":
        actual = line["response"]["status"] if "response" in line else line["fault"]["class"]
        return _compare(claim, actual == value, actual)
    if attribute == "compatible":
        report = _report(kind, body)
        if report is None or "evaluated" not in report["status"]:
            return _verdict(claim, "unchecked", "no evaluated report at that reference")
        actual = report["status"]["evaluated"]["compatible"]
        return _compare(claim, str(actual).lower() == value.lower(), actual)
    if attribute == "component":
        report = _report(kind, body)
        if report is None or "evaluated" not in report["status"]:
            return _verdict(claim, "unchecked", "no evaluated report at that reference")
        try:
            index, expected = value.split("=")
            actual = report["status"]["evaluated"]["components"][int(index)]
        except (ValueError, IndexError):
            return _verdict(claim, "mismatch", "component index not present")
        return _compare(claim, str(actual).lower() == expected.lower(), actual)
    if attribute == "contains_event":
        if kind == "forward":
            events = body["known"]
        elif kind == "evaluation":
            if not provenance:
                return _verdict(claim, "out_of_view", "the evaluation closure is not delivered in this condition")
            events = (body.get("closure") or {}).get("known", [])
        else:
            return _verdict(claim, "unchecked", "reference carries no event set")
        actual = any(e["verb"] == value for e in events)
        return _compare(claim, actual, actual)
    if attribute == "classes":
        if kind != "partition":
            return _verdict(claim, "unchecked", "reference is not a partition")
        if condition != "C":
            return _verdict(claim, "out_of_view", "partitions are not delivered in this condition")
        actual = len(set(body["partition"]["kernel"]))
        return _compare(claim, str(actual) == value, actual)
    if attribute == "differs_from":
        if value not in records:
            return _verdict(claim, "unreferenced", f"{value} is not an action of this episode")
        other = records[value]
        same = _payload(line) == _payload(other) and _status(line) == _status(other)
        return _compare(claim, not same, "differs" if not same else "equal")
    return _verdict(claim, "unchecked", f"attribute {attribute} has no rule in {RULE_VERSION}")


def _status(line):
    return line["response"]["status"] if "response" in line else line["fault"]["class"]


def _report(kind, body):
    if kind == "candidate":
        return body["report"]
    if kind == "evaluation":
        return body.get("report")
    return None


def _compare(claim, ok: bool, actual):
    return _verdict(claim, "match" if ok else "mismatch", f"recorded {actual!r}")


def _verdict(claim, verdict, detail):
    return {"kind": claim["kind"], "assertion": claim.get("assertion"), "text": claim["text"],
            "verdict": verdict, "detail": detail, "rule_version": RULE_VERSION}
