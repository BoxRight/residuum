"""Episode state machine and budgets (invocation protocol §6, §8).

States move only through the transitions listed; budgets are reserved
before a model request is sent and charged after; an exhausted budget ends
the episode with ``budget_exhausted`` rather than refusing silently.
"""
from __future__ import annotations

from dataclasses import dataclass, field

TRANSITIONS = {
    "CREATED": {"PREFLIGHT"},
    "PREFLIGHT": {"READY", "ABORTED"},
    "READY": {"MODEL_REQUESTED", "WAITING_VALUATION", "FINISHED", "BUDGET_EXHAUSTED", "ABORTED"},
    "MODEL_REQUESTED": {"MODEL_RECEIVED", "ABORTED"},
    "MODEL_RECEIVED": {"ACTION_VALIDATED", "READY", "ABORTED"},
    "ACTION_VALIDATED": {"TOOL_RUNNING", "READY"},
    "TOOL_RUNNING": {"RESULT_RECORDED", "ABORTED"},
    "RESULT_RECORDED": {"VIEW_RECORDED"},
    "VIEW_RECORDED": {"READY"},
    "WAITING_VALUATION": {"READY"},
    "FINISHED": set(),
    "BUDGET_EXHAUSTED": set(),
    "ABORTED": set(),
}
TERMINAL = {"FINISHED", "BUDGET_EXHAUSTED", "ABORTED"}


class StateError(RuntimeError):
    pass


class BudgetExhausted(RuntimeError):
    pass


@dataclass
class Budgets:
    max_model_requests: int
    max_actions: int
    max_output_tokens: int
    model_requests: int = 0
    actions: int = 0
    output_tokens: int = 0
    usage_unknown: int = 0

    def reserve_request(self) -> None:
        if self.model_requests >= self.max_model_requests:
            raise BudgetExhausted("model requests exhausted")
        self.model_requests += 1

    def charge_action(self) -> None:
        if self.actions >= self.max_actions:
            raise BudgetExhausted("actions exhausted")
        self.actions += 1

    def output_cap(self) -> int:
        remaining = self.max_output_tokens - self.output_tokens
        if remaining <= 0:
            raise BudgetExhausted("output tokens exhausted")
        return remaining

    def charge_usage(self, usage: dict | None) -> None:
        if usage is None or "output_tokens" not in usage:
            self.usage_unknown += 1
            return
        self.output_tokens += int(usage["output_tokens"])

    def snapshot(self) -> dict:
        return {
            "model_requests": [self.model_requests, self.max_model_requests],
            "actions": [self.actions, self.max_actions],
            "output_tokens": [self.output_tokens, self.max_output_tokens],
            "usage_unknown": self.usage_unknown,
        }


@dataclass
class EpisodeState:
    state: str = "CREATED"
    history: list[str] = field(default_factory=lambda: ["CREATED"])

    def to(self, target: str) -> None:
        if target not in TRANSITIONS[self.state]:
            raise StateError(f"illegal transition {self.state} -> {target}")
        self.state = target
        self.history.append(target)

    @property
    def terminal(self) -> bool:
        return self.state in TERMINAL
