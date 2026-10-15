"""Episode manifest: frozen before the first turn, never edited by a request.

The file has two sections. ``worker`` is exactly the worker binary's
manifest (``boundary/src/manifest.rs``, ``WireManifest``); its
``manifest_hash`` is the opaque identity the worker checks on every
envelope. ``controller`` holds what the worker never sees: condition,
model configuration placeholder, model budgets, protocol versions and the
valuation source. The controller freezes the whole file by its SHA-256
over canonical JSON and re-checks that hash at every turn.
"""
from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path

WIRE_VERSION = "residuum-wire/1"
PROTOCOL_VERSION = "residuum-worker/0.1"
CONTROLLER_VERSION = "normative-explorer/0.1"
CONDITIONS = ("A", "B", "C")

WORKER_KEYS = {
    "wire_version", "episode_id", "manifest_hash", "imports", "workdir", "profile",
    "decision", "dossiers", "budgets", "transport",
}
CONTROLLER_KEYS = {
    "controller_version", "condition", "model", "max_model_requests", "max_actions",
    "max_output_tokens", "valuation_source", "explorer_instructions",
}
MODEL_KEYS = {"provider", "model_id", "parameters"}


class ManifestError(ValueError):
    """The manifest is not acceptable; the episode must not start."""


def canonical(value) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


@dataclass(frozen=True)
class EpisodeManifest:
    worker: dict
    controller: dict
    episode_hash: str
    source_path: Path

    @property
    def episode_id(self) -> str:
        return self.worker["episode_id"]

    @property
    def worker_hash(self) -> str:
        return self.worker["manifest_hash"]

    @property
    def condition(self) -> str:
        return self.controller["condition"]

    @property
    def dossiers(self) -> list[str]:
        return sorted(self.worker["dossiers"].keys())


def _require(mapping: dict, keys: set[str], where: str) -> None:
    missing = keys - set(mapping)
    extra = set(mapping) - keys
    if missing:
        raise ManifestError(f"{where}: missing {sorted(missing)}")
    if extra:
        raise ManifestError(f"{where}: unknown {sorted(extra)}")


def _ident(value, what: str) -> str:
    if not isinstance(value, str) or not value or len(value) > 128 or any(
        c.isspace() or ord(c) < 32 for c in value
    ):
        raise ManifestError(f"{what} {value!r} is not a valid identifier")
    return value


def _positive(value, what: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise ManifestError(f"{what} must be a positive integer")
    return value


def validate(document: dict, source_path: Path) -> EpisodeManifest:
    if not isinstance(document, dict) or set(document) != {"worker", "controller"}:
        raise ManifestError("manifest must have exactly the sections worker and controller")
    worker, controller = document["worker"], document["controller"]
    _require(worker, WORKER_KEYS, "worker")
    _require(controller, CONTROLLER_KEYS, "controller")
    if worker["wire_version"] != WIRE_VERSION:
        raise ManifestError(f"worker wire_version must be {WIRE_VERSION}")
    _ident(worker["episode_id"], "episode_id")
    _ident(worker["manifest_hash"], "manifest_hash")
    if not isinstance(worker["imports"], list) or not worker["imports"]:
        raise ManifestError("worker.imports must be a non-empty list")
    if not isinstance(worker["dossiers"], dict) or not worker["dossiers"]:
        raise ManifestError("worker.dossiers must be a non-empty object")
    for name in worker["dossiers"]:
        _ident(name, "dossier")
    for key in ("max_actions", "max_steps_per_action", "max_steps_total", "max_sessions",
                "max_candidates", "max_clauses"):
        _positive(worker["budgets"].get(key), f"worker.budgets.{key}")
    if controller["controller_version"] != CONTROLLER_VERSION:
        raise ManifestError(f"controller_version must be {CONTROLLER_VERSION}")
    if controller["condition"] not in CONDITIONS:
        raise ManifestError("condition must be one of A, B, C")
    _require(controller["model"], MODEL_KEYS, "controller.model")
    _ident(controller["model"]["provider"], "model.provider")
    _ident(controller["model"]["model_id"], "model.model_id")
    for key in ("max_model_requests", "max_actions", "max_output_tokens"):
        _positive(controller[key], f"controller.{key}")
    if not isinstance(controller["valuation_source"], str) or not controller["valuation_source"]:
        raise ManifestError("valuation_source must name the independent source")
    if not isinstance(controller["explorer_instructions"], str):
        raise ManifestError("explorer_instructions must be text")
    return EpisodeManifest(
        worker=worker,
        controller=controller,
        episode_hash=sha256_text(canonical(document)),
        source_path=source_path,
    )


def load(path: Path, max_bytes: int = 4 * 1024 * 1024) -> EpisodeManifest:
    data = path.read_bytes()
    if len(data) > max_bytes:
        raise ManifestError(f"manifest of {len(data)} bytes exceeds {max_bytes}")
    try:
        document = json.loads(data)
    except json.JSONDecodeError as error:
        raise ManifestError(f"manifest is not JSON: {error}") from error
    return validate(document, path)


def recheck(manifest: EpisodeManifest) -> None:
    """Every turn: the file on disk must still hash to the frozen value."""
    current = load(manifest.source_path)
    if current.episode_hash != manifest.episode_hash:
        raise ManifestError("manifest changed on disk; the episode must not continue")


def worker_manifest_text(manifest: EpisodeManifest) -> str:
    """The exact file handed to the worker binary."""
    return json.dumps(manifest.worker, indent=2, ensure_ascii=False) + "\n"
