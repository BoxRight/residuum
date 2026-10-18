"""Injectable backend interface: whatever answers a worker envelope with one
protocol line. The controller never launches the binary itself; the
interactive bounded mode is a later runner change. ``RecordedBackend``
replays the lines the Rust boundary generator recorded, matched by request
content, so controller tests use real worker output without a process.
"""
from __future__ import annotations

import json
from pathlib import Path

from .manifest import canonical


class BackendError(RuntimeError):
    pass


class Backend:
    def handle(self, envelope: dict) -> dict:  # pragma: no cover - interface
        raise NotImplementedError


class RecordedBackend(Backend):
    def __init__(self, requests_path: Path, expected_path: Path):
        self.responses: dict[str, dict] = {}
        requests = requests_path.read_text(encoding="utf-8").split("\n")[:-1]
        expected = expected_path.read_text(encoding="utf-8").split("\n")[:-1]
        if len(requests) != len(expected):
            raise BackendError("recorded requests and responses differ in length")
        for request, response in zip(requests, expected):
            try:
                envelope = json.loads(request)
            except json.JSONDecodeError:
                continue  # protocol-fault lines have no request to match
            line = json.loads(response)
            if "response" not in line:
                continue
            self.responses.setdefault(canonical(envelope["request"]), line)
        self.calls: list[dict] = []

    def handle(self, envelope: dict) -> dict:
        self.calls.append(envelope)
        key = canonical(envelope["request"])
        if key not in self.responses:
            raise BackendError(f"no recorded response for request {key[:80]}…")
        line = json.loads(json.dumps(self.responses[key]))
        line["response"]["action_id"] = envelope["action_id"]
        return line


class StubBackend(Backend):
    """Fixed lines in order, for failure paths. Each entry is a full line or
    a callable from envelope to line."""

    def __init__(self, lines):
        self.lines = list(lines)
        self.calls: list[dict] = []

    def handle(self, envelope: dict) -> dict:
        self.calls.append(envelope)
        if not self.lines:
            raise BackendError("stub exhausted")
        line = self.lines.pop(0)
        return line(envelope) if callable(line) else json.loads(json.dumps(line))
