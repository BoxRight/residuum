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
    "max_output_tokens", "valuation_source", "explorer_instructions", "observations",
}
OBSERVATION_KEYS = {"dossier", "required", "source", "status"}
OBSERVATION_STATUS = ("documented", "hypothetical", "pending")
MODEL_KEYS = {"provider", "model_id", "parameters"}
# Real providers need an explicit network deadline; the simulated one has none.
REAL_PROVIDERS = ("anthropic", "openai")
CLIENT_KEYS = {"timeout_seconds"}


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


OPTIONAL_CONTROLLER_KEYS = {"max_input_tokens", "max_cost_usd", "pricing"}
PRICING_KEYS = {"input_per_mtok", "output_per_mtok", "cache_write_multiplier", "cache_read_per_mtok"}
# Worker actions the instrumented entry point uses besides the explorer's:
# one to freeze the vocabulary, three for the idempotence probe.
TECHNICAL_ACTIONS = 4
TECHNICAL_STEPS = 3
TECHNICAL_SESSIONS = 1


def instrumentation_problems(m) -> list[str]:
    """Worker budgets must leave room for the technical actions on top of
    everything the explorer may use, so that a valid exploration never fails
    for the harness's own needs."""
    w, c = m.worker["budgets"], m.controller
    problems = []
    if w["max_actions"] < c["max_actions"] + TECHNICAL_ACTIONS:
        problems.append(f"worker max_actions {w['max_actions']} < explorer {c['max_actions']} + technical {TECHNICAL_ACTIONS}")
    if w["max_steps_total"] < TECHNICAL_STEPS + 1:
        problems.append("worker max_steps_total leaves no step after the probe")
    if w["max_sessions"] < TECHNICAL_SESSIONS + 1:
        problems.append("worker max_sessions leaves no session after the probe")
    return problems


def validate(document: dict, source_path: Path) -> EpisodeManifest:
    if not isinstance(document, dict) or set(document) != {"worker", "controller"}:
        raise ManifestError("manifest must have exactly the sections worker and controller")
    worker, controller = document["worker"], document["controller"]
    _require(worker, WORKER_KEYS, "worker")
    _require({k: v for k, v in controller.items() if k not in OPTIONAL_CONTROLLER_KEYS},
             CONTROLLER_KEYS, "controller")
    if "max_input_tokens" in controller:
        _positive(controller["max_input_tokens"], "controller.max_input_tokens")
    if "max_cost_usd" in controller:
        cost = controller["max_cost_usd"]
        if not isinstance(cost, (int, float)) or isinstance(cost, bool) or cost <= 0:
            raise ManifestError("controller.max_cost_usd must be a positive number")
        if "pricing" not in controller:
            raise ManifestError("a cost ceiling needs controller.pricing to bound each call")
    if "pricing" in controller:
        _require(controller["pricing"], PRICING_KEYS, "controller.pricing")
        for key, value in controller["pricing"].items():
            if not isinstance(value, (int, float)) or isinstance(value, bool) or value <= 0:
                raise ManifestError(f"controller.pricing.{key} must be a positive number")
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
    model = controller["model"]
    if not isinstance(model, dict):
        raise ManifestError("controller.model must be an object")
    _require({k: v for k, v in model.items() if k != "client"}, MODEL_KEYS, "controller.model")
    if model["provider"] in REAL_PROVIDERS:
        if "client" not in model:
            raise ManifestError("a real provider needs controller.model.client.timeout_seconds")
        _require(model["client"], CLIENT_KEYS, "controller.model.client")
        timeout = model["client"]["timeout_seconds"]
        if not isinstance(timeout, (int, float)) or isinstance(timeout, bool) or timeout <= 0:
            raise ManifestError("controller.model.client.timeout_seconds must be a positive number")
    elif "client" in model:
        raise ManifestError("controller.model.client is only for real providers")
    _ident(controller["model"]["provider"], "model.provider")
    _ident(controller["model"]["model_id"], "model.model_id")
    for key in ("max_model_requests", "max_actions", "max_output_tokens"):
        _positive(controller[key], f"controller.{key}")
    if not isinstance(controller["valuation_source"], str) or not controller["valuation_source"]:
        raise ManifestError("valuation_source must name the independent source")
    if not isinstance(controller["explorer_instructions"], str):
        raise ManifestError("explorer_instructions must be text")
    if not isinstance(controller["observations"], list):
        raise ManifestError("observations must be a list")
    for i, obs in enumerate(controller["observations"]):
        _require(obs, OBSERVATION_KEYS, f"observations[{i}]")
        if obs["dossier"] not in worker["dossiers"]:
            raise ManifestError(f"observations[{i}]: unknown dossier {obs['dossier']!r}")
        if not isinstance(obs["required"], bool) or obs["status"] not in OBSERVATION_STATUS:
            raise ManifestError(f"observations[{i}]: required must be boolean and status documented, hypothetical or pending")
        if not isinstance(obs["source"], str) or not obs["source"]:
            raise ManifestError(f"observations[{i}]: source required")
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
