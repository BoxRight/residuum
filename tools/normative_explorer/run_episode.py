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
    try:
        outcome = ctl.run()
    finally:
        backend.close()
    summary = {"outcome": outcome, "state": ctl.state.state, "turns": ctl.turns, "worker_calls": backend.calls,
               "decisions": getattr(policy, "decisions", []), "budgets": ctl.budgets.snapshot()}
    print(f"controller: {json.dumps(summary, ensure_ascii=False)}", file=stderr, flush=True)
    return EXIT.get(outcome, 4)


if __name__ == "__main__":  # pragma: no cover - launched by the supervisor only
    sys.exit(main(sys.argv[1:]))
