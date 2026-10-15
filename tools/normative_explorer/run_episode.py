"""Controller entry point for the supervised interactive mode. Launched only
by the runner's supervisor, which hands over the two channel descriptors;
this module opens nothing else and starts no process.

    python3 -I -c "<bootstrap>" --manifest FILE --ledger DIR --policy NAME \\
                                --to-worker FD --from-worker FD

Exit codes: 0 finished with a normative outcome (verified, none, awaiting);
3 budget exhausted; 4 technical abort; 2 usage; 5 manifest rejected.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from . import controller as controller_mod
from . import ledger as ledger_mod
from . import manifest as manifest_mod
from . import provider_sim
from .channel import ChannelBackend
from .policies import POLICIES

EXIT = {"verified_compatible_candidate": 0, "no_verified_candidate": 0, "awaiting_independent_valuation": 0,
        "budget_exhausted": 3, "technical_abort": 4}
TECHNICAL_ACCEPTANCE_FAILED = 6


def parse(argv: list[str]) -> dict:
    expected = {"--manifest", "--ledger", "--policy", "--to-worker", "--from-worker"}
    values = dict(zip(argv[0::2], argv[1::2]))
    if set(values) != expected or len(argv) != 10:
        raise SystemExit("usage: --manifest FILE --ledger DIR --policy NAME --to-worker FD --from-worker FD")
    try:
        values["--to-worker"] = int(values["--to-worker"])
        values["--from-worker"] = int(values["--from-worker"])
    except ValueError as error:
        raise SystemExit(f"descriptors must be integers: {error}") from error
    if values["--policy"] not in POLICIES:
        raise SystemExit(f"unknown policy {values['--policy']!r}")
    return values


class PolicyProvider:
    """Adapts a branching policy to the provider interface."""

    def __init__(self, policy):
        self.policy = policy
        self.calls = 0

    def next_action(self, visible_history, tools, output_cap):
        self.calls += 1
        return self.policy.next_action(visible_history, tools, output_cap), {"output_tokens": 20}


def main(argv: list[str], stderr=sys.stderr) -> int:
    try:
        values = parse(argv)
    except SystemExit as error:
        print(f"controller: {error}", file=stderr, flush=True)
        return 2
    try:
        manifest = manifest_mod.load(Path(values["--manifest"]))
    except (manifest_mod.ManifestError, OSError) as error:
        print(f"controller: manifest rejected: {error}", file=stderr, flush=True)
        return 5
    ledger = ledger_mod.Ledger(Path(values["--ledger"]), manifest.episode_hash)
    goal = dict(manifest.worker["decision"]["components"][-1])
    policy = POLICIES[values["--policy"]](goal)
    backend = ChannelBackend.from_descriptors(values["--to-worker"], values["--from-worker"])
    ctl = controller_mod.Controller(manifest, ledger, backend, PolicyProvider(policy))
    probe = None
    try:
        outcome = ctl.run()
        # Technical acceptance on mutable state against the same live worker,
        # outside the model loop and after the normative outcome is recorded.
        if outcome != "technical_abort":
            probe = ctl.idempotence_probe(probe_dossier(manifest))
    finally:
        backend.close()
    summary = {"outcome": outcome, "state": ctl.state.state, "turns": ctl.turns, "worker_calls": backend.calls,
               "decisions": getattr(policy, "decisions", []), "budgets": ctl.budgets.snapshot(),
               "idempotence_probe": probe}
    print(f"controller: {json.dumps(summary, ensure_ascii=False)}", file=stderr, flush=True)
    return exit_code(outcome, probe)


def probe_dossier(manifest) -> str:
    """The documented observation's dossier when there is one; the probe
    needs a forward closure that is still pending after two rounds."""
    observations = manifest.controller["observations"]
    return observations[0]["dossier"] if observations else manifest.dossiers[0]


def exit_code(outcome: str, probe: dict | None) -> int:
    """The normative outcome sets the code; a failed technical acceptance
    overrides a successful one with 6 while the outcome stays recorded."""
    code = EXIT.get(outcome, 4)
    if code == 0 and probe is not None and not probe["passed"]:
        return TECHNICAL_ACCEPTANCE_FAILED
    return code


if __name__ == "__main__":  # pragma: no cover - launched by the supervisor only
    sys.exit(main(sys.argv[1:]))
