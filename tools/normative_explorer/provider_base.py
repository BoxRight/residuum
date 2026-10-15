"""Provider-neutral pieces shared by the model adapters: the error a
provider raises when an answer or the transport is unusable, the Formal
syntax the proposal path accepts, and the initial public context, which is
identical for every provider and, except for condition, tools and episode
id, for every condition."""
from __future__ import annotations

import json

from .manifest import EpisodeManifest
from .schemas import catalog

FORMAL_SYNTAX = """Proposals are Formal rule declarations only (no module, import, verb, const or
experiment). One rule:

  rule <name>(<param>, ...) =
      <atom> [& <atom>]...      (or I for an empty body)
      <= <head atom>

An atom is <verb>(<arg>, ...) @ <time>; a negative seeded atom is ~<verb>(...) @ <time>.
Arguments are parameters (variables) or declared constants; `none` fills an optional
position. Times are a parameter, a constant, or `after <param>`. Every parameter used in
the head must occur in the body. The head verb must be one of the profile's heads; body
verbs must be in the profile's vocabulary; at most the profile's atoms and binders.
Rules are evaluated by the worker; proposing one does not make it part of the theory."""


class ProviderError(RuntimeError):
    """The model's output or the transport is not usable; no action executes.
    ``usage`` carries what the provider reported for the attempt, so the
    cost is charged even when the answer is rejected."""

    def __init__(self, message: str, usage: dict | None = None, details: dict | None = None,
                 fatal: bool = False):
        super().__init__(message)
        self.usage = usage
        # Structured facts of a provider rejection (HTTP status, error type,
        # request id) when the transport gives them.
        self.details = details
        # A configuration error the same request cannot recover from: the
        # controller ends the episode instead of sending it again.
        self.fatal = fatal


def initial_context(manifest: EpisodeManifest, vocabulary: dict | None = None) -> str:
    """What the model is told at the start: the public facts of the episode
    and, when the controller froze it, the vocabulary package (signatures,
    entities, constants, rules of the baseline) with the Formal syntax the
    proposal path accepts. Identical across conditions. No repository
    documents, no reserved artefacts, no catalogue."""
    c = manifest.controller
    facts = {
        "vocabulary": vocabulary,
        "syntax": FORMAL_SYNTAX,
        "dossier_seeds": manifest.worker["dossiers"],
        "episode_id": manifest.episode_id,
        "condition": manifest.condition,
        "dossiers": manifest.dossiers,
        "decision": manifest.worker["decision"],
        "profile": manifest.worker["profile"],
        "budgets": {"actions": c["max_actions"], "model_requests": c["max_model_requests"]},
        "observations": [{"dossier": o["dossier"], "required": o["required"], "status": o["status"]}
                         for o in c["observations"]],
        "tools": catalog(manifest.condition),
    }
    return json.dumps(facts, ensure_ascii=False, sort_keys=True)
