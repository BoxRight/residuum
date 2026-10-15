"""Append-only episode ledger (invocation protocol §11).

``events.jsonl`` is the single order of what happened: every entry carries
its sequence number, a timestamp, the hash of the previous entry and its
own hash, the frozen episode hash, a kind and a payload. Specialised files
(actions, model requests and responses, visible views, candidates,
partitions, valuations, progress, claims, evaluator trace) hold the full
records and are referenced from the events by sequence number. Artefacts
larger than a line go under ``artifacts/sha256/<digest>``. Nothing is ever
rewritten: a change is a new event. Hashes detect alteration of what was
kept; they do not prove truth about the world.
"""
from __future__ import annotations

import datetime as _dt
import hashlib
import json
from pathlib import Path

from .manifest import canonical, sha256_text

GENESIS = "0" * 64
SPECIALISED = (
    "actions", "model-requests", "model-responses", "visible-views", "candidates",
    "partitions", "valuations", "progress", "claims", "evaluator-trace",
)


class LedgerError(RuntimeError):
    pass


class Ledger:
    def __init__(self, directory: Path, episode_hash: str, clock=None):
        self.directory = directory
        self.episode_hash = episode_hash
        self.clock = clock or (lambda: _dt.datetime.now(_dt.timezone.utc).isoformat())
        self.directory.mkdir(parents=True, exist_ok=True)
        (self.directory / "artifacts" / "sha256").mkdir(parents=True, exist_ok=True)
        self.events_path = self.directory / "events.jsonl"
        self._seq = 0
        self._prev = GENESIS
        if self.events_path.exists():
            self._resume()
        self._counters: dict[str, int] = {}

    def _resume(self) -> None:
        for entry in self.read_events(verify=True):
            self._seq = entry["seq"]
            self._prev = entry["hash"]

    # -- identifiers ------------------------------------------------------

    def next_id(self, prefix: str) -> str:
        """Controller-assigned identifiers: a1, a2, ... per prefix, persisted
        through the events so that a resumed ledger continues the series."""
        n = self._counters.get(prefix, 0) + 1
        self._counters[prefix] = n
        return f"{prefix}{n}"

    # -- writing ----------------------------------------------------------

    def append(self, kind: str, payload: dict, file: str | None = None) -> dict:
        if file is not None and file not in SPECIALISED:
            raise LedgerError(f"unknown specialised file {file}")
        self._seq += 1
        entry = {
            "seq": self._seq,
            "at": self.clock(),
            "episode_hash": self.episode_hash,
            "prev": self._prev,
            "kind": kind,
            "payload": payload,
        }
        if file is not None:
            record = {"seq": self._seq, "kind": kind, **payload}
            with (self.directory / f"{file}.jsonl").open("a", encoding="utf-8") as handle:
                handle.write(canonical(record) + "\n")
            entry["file"] = file
        body = canonical({k: v for k, v in entry.items() if k != "hash"})
        entry["hash"] = sha256_text(body)
        with self.events_path.open("a", encoding="utf-8") as handle:
            handle.write(canonical(entry) + "\n")
        self._prev = entry["hash"]
        return entry

    def store_artifact(self, text: str) -> str:
        digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
        path = self.directory / "artifacts" / "sha256" / digest
        if not path.exists():
            path.write_text(text, encoding="utf-8")
        return digest

    def read_artifact(self, digest: str) -> str:
        return (self.directory / "artifacts" / "sha256" / digest).read_text(encoding="utf-8")

    # -- reading and reconstruction ----------------------------------------

    def read_events(self, verify: bool = True) -> list[dict]:
        events = []
        prev = GENESIS
        if not self.events_path.exists():
            return events
        for line_number, line in enumerate(self.events_path.read_text(encoding="utf-8").splitlines(), 1):
            entry = json.loads(line)
            if verify:
                expected = sha256_text(canonical({k: v for k, v in entry.items() if k != "hash"}))
                if entry["hash"] != expected:
                    raise LedgerError(f"events.jsonl line {line_number}: hash mismatch")
                if entry["prev"] != prev:
                    raise LedgerError(f"events.jsonl line {line_number}: broken chain")
                if entry["episode_hash"] != self.episode_hash:
                    raise LedgerError(f"events.jsonl line {line_number}: foreign episode")
                if entry["seq"] != len(events) + 1:
                    raise LedgerError(f"events.jsonl line {line_number}: sequence gap")
            prev = entry["hash"]
            events.append(entry)
        return events

    def read_file(self, file: str) -> list[dict]:
        path = self.directory / f"{file}.jsonl"
        if not path.exists():
            return []
        return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]

    def reconstruct(self) -> dict:
        """Rebuild the history from the ledger alone: every action with its
        request, result reference, visible view and status, in order."""
        events = self.read_events(verify=True)
        by_file = {name: {r["seq"]: r for r in self.read_file(name)} for name in SPECIALISED}
        for name, records in by_file.items():
            for seq in records:
                if seq < 1 or seq > len(events) or events[seq - 1].get("file") != name:
                    raise LedgerError(f"{name}.jsonl references seq {seq} that is not its event")
        actions: dict[str, dict] = {}
        terminal = None
        for entry in events:
            kind, payload = entry["kind"], entry["payload"]
            if kind in ("episode.finished", "episode.budget_exhausted", "episode.aborted"):
                terminal = kind
            action_id = payload.get("action_id")
            if kind == "action.validated":
                actions[action_id] = {"seq": entry["seq"], "request": payload["request"], "tool": payload["tool"]}
            elif kind == "action.result" and action_id in actions:
                actions[action_id]["status"] = payload["status"]
                actions[action_id]["result_artifact"] = payload["artifact"]
            elif kind == "view.recorded" and action_id in actions:
                actions[action_id]["view_artifact"] = payload["artifact"]
        # An episode without a terminal event was interrupted from outside
        # (worker lost, supervisor stop, kill): the supervisor's report, not
        # the ledger, states how it ended.
        return {"events": len(events), "actions": actions, "terminal": terminal,
                "interrupted": terminal is None and bool(events)}
