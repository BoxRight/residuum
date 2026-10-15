"""Independent evaluation of an archived candidate by Residuum alone.

Stage I after pilot 003 (``docs/legal-geometry/exploration/pilot-003/
evaluation/``). No model is involved: every result comes from the worker,
driven by a fixed request script through ``scripts/test_safe.py
--worker-manifest/--worker-requests``.

Two scripts:

- ``reproduction``: the exact worker envelopes of an archived episode
  (vocabulary freeze, idempotence probe, explorer actions), rebuilt from
  its ledger, to be compared with the archived responses.
- ``evaluation``: for every dossier, the baseline closure C_T(S) by a
  forward session and the hypothetical closure C_{T+r}(S) by a candidate
  evaluation, each advanced until complete (advancing a complete session
  consumes no step and returns the full result again).

``analyse`` turns the worker's protocol output into, per dossier: whether
the observation channel holds in each closure, completeness, the
derivations that reach it, and whether the hypothetical closure restricted
to the reference vocabulary equals the baseline closure, justifications
counted with multiplicity.
"""
from __future__ import annotations

import json
import sys
from collections import Counter
from pathlib import Path

WIRE = {"wire_version": "residuum-wire/1", "protocol_version": "residuum-worker/0.1"}
STEPS = 8          # max_steps_per_action of the manifests
ADVANCES = 4       # advances per session; a complete session ignores the rest


def _envelope(worker: dict, action_id: str, request) -> dict:
    return dict(WIRE, episode_id=worker["episode_id"], action_id=action_id,
                manifest_hash=worker["manifest_hash"], request=request)


def _records(ledger: Path, name: str) -> list[dict]:
    path = ledger / f"{name}.jsonl"
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []


def reproduction_requests(ledger: Path) -> list[dict]:
    """The worker envelopes of the archived episode, in order, rebuilt from
    the ledger: the vocabulary freeze, the probe's four sends (the replay is
    the same envelope again) and every validated explorer action."""
    worker = json.loads((ledger / "worker-manifest.json").read_text())
    events = _records(ledger, "events")
    vocabulary = next(e for e in events if e["kind"] == "vocabulary.frozen")["payload"]
    probe = next(e for e in events if e["kind"] == "idempotence.probe")["payload"]
    lines = [_envelope(worker, vocabulary["action_id"], "inspect_theory")]
    dossier = probe["dossier"]
    session = "s1"
    lines.append(_envelope(worker, probe["steps"][0]["action_id"], {"begin_forward": {"dossier": dossier, "theory": None}}))
    advance = _envelope(worker, probe["steps"][1]["action_id"], {"advance_session": {"session": session, "max_steps": 2}})
    lines += [advance, dict(advance)]
    lines.append(_envelope(worker, probe["steps"][3]["action_id"], {"advance_session": {"session": session, "max_steps": 1}}))
    for record in _records(ledger, "actions"):
        if record["kind"] == "action.validated":
            lines.append(_envelope(worker, record["action_id"], record["request"]))
    return lines


def expected_responses(ledger: Path) -> list[dict]:
    """The archived response for each envelope of ``reproduction_requests``,
    in the same order. The vocabulary artefact keeps only the theory."""
    artifacts = ledger / "artifacts" / "sha256"
    events = _records(ledger, "events")
    vocabulary = next(e for e in events if e["kind"] == "vocabulary.frozen")["payload"]
    probe = next(e for e in events if e["kind"] == "idempotence.probe")["payload"]
    out = [{"theory_only": json.loads((artifacts / vocabulary["artifact"]).read_text())}]
    out += [json.loads((artifacts / step["artifact"]).read_text()) for step in probe["steps"]]
    for record in _records(ledger, "evaluator-trace"):
        if record["kind"] == "action.result":
            out.append(json.loads((artifacts / record["artifact"]).read_text()))
    return out


def compare_reproduction(got: list[dict], expected: list[dict], workdir: str, archived_workdir: str) -> list[dict]:
    """Per envelope, whether the replayed line equals the archived one. Two
    normalisations, both stated: the transport strips ``wire_version`` from
    the response before archiving it, and the worker's private workdir path
    differs between the two runs."""
    rows = []
    for g, e in zip(got, expected):
        if "theory_only" in e:
            same = g.get("response", {}).get("payload", {}).get("theory") == e["theory_only"]
        else:
            line = json.loads(json.dumps(g).replace(workdir, archived_workdir))
            line.get("response", {}).pop("wire_version", None)
            same = line == e
        rows.append({"action_id": g.get("response", {}).get("action_id"), "identical": same,
                     "workdir_in_line": workdir in json.dumps(g)})
    if len(got) != len(expected):
        rows.append({"action_id": None, "identical": False, "count": [len(got), len(expected)]})
    return rows


def evaluation_requests(worker: dict, candidate_text: str) -> list[dict]:
    """Propose the candidate once, then for every dossier (sorted) a
    baseline forward session and a candidate evaluation, each advanced
    ADVANCES times. Sessions are numbered by the worker in opening order."""
    lines, n, session = [], 0, 0

    def send(request):
        nonlocal n
        n += 1
        lines.append(_envelope(worker, f"e{n}", request))
    send({"propose_candidate": {"text": candidate_text}})
    for dossier in sorted(worker["dossiers"]):
        for opening in ({"begin_forward": {"dossier": dossier, "theory": None}},
                        {"begin_candidate_evaluation": {"candidate": "c1", "dossier": dossier}}):
            send(opening)
            session += 1
            for _ in range(ADVANCES):
                send({"advance_session": {"session": f"s{session}", "max_steps": STEPS}})
    return lines


def _event_key(event: dict) -> str:
    return json.dumps(event, sort_keys=True)


def _derivation_key(derivation: dict) -> str:
    """A justification: the event, the rule and the antecedents with the
    substitution; the step label (d1, d2, ...) is not part of it."""
    record = derivation["record"]
    return json.dumps({"event": derivation["event"], "rule": record["rule"],
                       "antecedents": record["antecedents"], "substitution": record.get("substitution")},
                      sort_keys=True)


def analyse(responses: list[dict], worker: dict, channel: str, reference_verbs: set[str]) -> dict:
    """Per dossier, from the protocol lines of an ``evaluation`` run."""
    by_id = {(line.get("response") or line.get("fault") or line).get("action_id"): line for line in responses}
    result, candidate_report = {}, None
    if "response" in by_id.get("e1", {}):
        candidate_report = by_id["e1"]["response"]["payload"]["candidate"]["report"]["id"]
    index = 2
    for dossier in sorted(worker["dossiers"]):
        entry = {}
        for kind in ("baseline", "candidate"):
            index += 1  # opening
            last = None
            derivations = Counter()
            for _ in range(ADVANCES):
                line = by_id.get(f"e{index}")
                index += 1
                if line is None or "response" not in line:
                    entry[kind] = {"error": line}
                    break
                last = line["response"]["payload"]
                body = last.get("forward") or last.get("evaluation") or last
                for round_ in body.get("rounds", []):
                    for d in round_.get("derivations", []):
                        derivations[_derivation_key(d)] += 1
            if kind in entry:
                continue
            body = last.get("forward") or last.get("evaluation")
            if "closure" in body:
                closure_derivations = Counter(_derivation_key(d) for d in body["closure"]["derivations"])
                known = body["closure"].get("known", [])
            else:
                closure_derivations = derivations
                known = body.get("known", [])
            events = [json.loads(k)["event"] for k in closure_derivations] + known
            holds = sorted({_event_key(e) for e in events if e["verb"] == channel})
            reaching = sorted(json.loads(k)["rule"] for k in closure_derivations
                              if json.loads(k)["event"]["verb"] == channel)
            entry[kind] = {"complete": bool(body.get("complete")), "channel": bool(holds),
                           "channel_events": [json.loads(h) for h in holds], "rules_reaching_channel": reaching,
                           "derivations": closure_derivations,
                           "known": sorted({_event_key(e) for e in known})}
        if "error" not in entry.get("baseline", {}) and "error" not in entry.get("candidate", {}):
            def project(c):
                return Counter({k: v for k, v in c.items() if json.loads(k)["event"]["verb"] in reference_verbs})
            def project_known(keys):
                return sorted(k for k in keys if json.loads(k)["verb"] in reference_verbs)
            entry["reference_preserved"] = (project(entry["candidate"]["derivations"]) ==
                                            project(entry["baseline"]["derivations"])
                                            and project_known(entry["candidate"]["known"]) ==
                                            project_known(entry["baseline"]["known"]))
            entry["extra_derivations"] = sorted(
                json.loads(k)["rule"] for k in entry["candidate"]["derivations"]
                if json.loads(k)["event"]["verb"] not in reference_verbs)
        result[dossier] = entry
    return {"candidate": candidate_report, "dossiers": result}


def summary(analysis: dict) -> dict:
    """The JSON-serialisable table: drops the raw derivation multisets."""
    out = {"candidate": analysis["candidate"], "dossiers": {}}
    for dossier, entry in analysis["dossiers"].items():
        row = {}
        for kind in ("baseline", "candidate"):
            e = entry[kind]
            if "error" in e:
                row[kind] = e
                continue
            row[kind] = {"complete": e["complete"], "channel": e["channel"],
                         "rules_reaching_channel": e["rules_reaching_channel"],
                         "channel_events": e["channel_events"],
                         "derivations": sum(e["derivations"].values()),
                         "distinct_justifications": len(e["derivations"])}
        row["reference_preserved"] = entry.get("reference_preserved")
        row["extra_derivations"] = entry.get("extra_derivations")
        out["dossiers"][dossier] = row
    return out


def _write_lines(path: Path, lines: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(line, ensure_ascii=False, sort_keys=True) + "\n" for line in lines))


def main(argv: list[str]) -> int:
    command = argv[0] if argv else ""
    if command == "reproduction" and len(argv) == 3:
        _write_lines(Path(argv[2]), reproduction_requests(Path(argv[1])))
        return 0
    if command == "evaluation" and len(argv) == 4:
        worker = json.loads(Path(argv[1]).read_text())
        _write_lines(Path(argv[3]), evaluation_requests(worker, Path(argv[2]).read_text()))
        return 0
    print("usage: reproduction LEDGER OUT | evaluation WORKER_MANIFEST CANDIDATE_TEXT OUT", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
