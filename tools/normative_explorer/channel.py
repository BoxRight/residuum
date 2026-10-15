"""Channel backend: the controller talks to a live worker through descriptors
that the runner's supervisor prepared and handed over. The controller never
launches the binary: no ``Popen``, no path to an executable, only two open
channels. One request in flight, bounded line reading, framing validation,
correlation of the answer with the request, and explicit technical errors
(worker lost, unexpected EOF, timeout, oversize, misframed) that the
controller records as ``technical_abort``, never as incompatibility.
"""
from __future__ import annotations

import json
import os
import select
import time

from .backend import Backend, BackendError

WIRE_VERSION = "residuum-wire/1"


class ChannelError(BackendError):
    """Technical: the channel, not the worker's answer, failed."""


class ChannelBackend(Backend):
    def __init__(self, to_worker, from_worker, max_line_bytes: int = 16 * 1024 * 1024,
                 request_timeout: float = 10.0):
        """``to_worker`` is a binary writable file object; ``from_worker`` a
        binary readable one with a real descriptor (``fileno``)."""
        self.to_worker = to_worker
        self.from_worker = from_worker
        self.max_line_bytes = max_line_bytes
        self.request_timeout = request_timeout
        self._buffer = bytearray()
        self._in_flight = False
        self.calls = 0
        self.closed = False

    @classmethod
    def from_descriptors(cls, to_worker_fd: int, from_worker_fd: int, **kwargs) -> "ChannelBackend":
        """Adopt descriptors inherited from the supervisor."""
        return cls(os.fdopen(to_worker_fd, "wb", buffering=0), os.fdopen(from_worker_fd, "rb", buffering=0), **kwargs)

    # -- one request, one line ----------------------------------------------

    def handle(self, envelope: dict) -> dict:
        if self.closed:
            raise ChannelError("channel closed")
        if self._in_flight:
            raise ChannelError("a request is already in flight; the protocol allows one")
        self._in_flight = True
        try:
            line = json.dumps(envelope, ensure_ascii=False, separators=(",", ":")).encode("utf-8") + b"\n"
            if len(line) - 1 > self.max_line_bytes:
                raise ChannelError(f"request of {len(line) - 1} bytes exceeds the line limit")
            try:
                self.to_worker.write(line)
                self.to_worker.flush()
            except (BrokenPipeError, OSError) as error:
                raise ChannelError(f"worker lost while writing: {error}") from error
            self.calls += 1
            raw = self._read_line(time.monotonic() + self.request_timeout)
            return self._frame(raw, envelope["action_id"])
        finally:
            self._in_flight = False

    def _read_line(self, deadline: float) -> bytes:
        fd = self.from_worker.fileno()
        while True:
            newline = self._buffer.find(b"\n")
            if newline != -1:
                raw = bytes(self._buffer[:newline])
                del self._buffer[: newline + 1]
                if len(raw) > self.max_line_bytes:
                    raise ChannelError(f"worker line of {len(raw)} bytes exceeds {self.max_line_bytes} bytes")
                return raw
            if len(self._buffer) > self.max_line_bytes:
                raise ChannelError(f"worker line exceeds {self.max_line_bytes} bytes; channel abandoned")
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise ChannelError(f"no answer within {self.request_timeout} s")
            ready, _, _ = select.select([fd], [], [], remaining)
            if not ready:
                raise ChannelError(f"no answer within {self.request_timeout} s")
            try:
                chunk = os.read(fd, 65536)
            except OSError as error:
                raise ChannelError(f"worker lost while reading: {error}") from error
            if not chunk:
                partial = len(self._buffer)
                raise ChannelError(f"unexpected EOF from the worker with {partial} unread bytes")
            self._buffer.extend(chunk)

    @staticmethod
    def _frame(raw: bytes, action_id: str) -> dict:
        if not raw.strip():
            raise ChannelError("empty line from the worker")
        try:
            line = json.loads(raw)
        except json.JSONDecodeError as error:
            raise ChannelError(f"misframed line from the worker: {error}") from error
        if not isinstance(line, dict) or len(line) != 1 or next(iter(line)) not in ("response", "fault"):
            raise ChannelError("worker line is neither a response nor a fault")
        if "response" in line:
            body = line["response"]
            if body.get("wire_version") != WIRE_VERSION:
                raise ChannelError(f"response wire version {body.get('wire_version')!r}")
            if body.get("action_id") != action_id:
                raise ChannelError(f"response for {body.get('action_id')!r} while {action_id!r} was in flight")
            body.pop("wire_version")
        else:
            fault = line["fault"]
            if fault.get("wire_version") != WIRE_VERSION:
                raise ChannelError(f"fault wire version {fault.get('wire_version')!r}")
            if fault.get("action_id") not in (None, action_id):
                raise ChannelError(f"fault for {fault.get('action_id')!r} while {action_id!r} was in flight")
        return line

    def close(self) -> None:
        self.closed = True
        for handle in (self.to_worker, self.from_worker):
            try:
                handle.close()
            except OSError:
                pass
