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
    "MODEL_REQUESTED": {"MODEL_RECEIVED", "READY", "ABORTED"},
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
    input_tokens: int = 0
    usage_unknown: int = 0
    # Optional ceilings. Before each call the controller reserves the call's
    # upper bound (reserve_call); a call is sent only if what was spent plus
    # that bound stays within the ceiling, so the ceiling is never exceeded
    # as long as the bound is a true upper bound and the prices are right.
    max_input_tokens: int | None = None
    max_cost_usd: float | None = None
    pricing: dict | None = None
    cost_usd: float = 0.0
    pending_reservation: float | None = None
    pending_input_bound: int | None = None

    def reserve_request(self) -> None:
        if self.model_requests >= self.max_model_requests:
            raise BudgetExhausted("model requests exhausted")
        if self.max_input_tokens is not None and self.input_tokens >= self.max_input_tokens:
            raise BudgetExhausted("input tokens exhausted")
        self.model_requests += 1

    def call_cost_bound(self, input_upper: int, output_upper: int) -> float:
        p = self.pricing
        return (input_upper * p["input_per_mtok"] * p["cache_write_multiplier"]
                + output_upper * p["output_per_mtok"]) / 1e6

    def reserve_call(self, input_upper: int | None, output_upper: int | None) -> None:
        """Reserve the next call's worst case against the ceilings, before
        anything is sent. A missing bound or missing prices stop the episode
        when a ceiling is set: its cost could not be established."""
        if self.max_input_tokens is not None:
            if input_upper is None:
                raise BudgetExhausted("input ceiling set but the next call's input bound is unavailable")
            if self.input_tokens + input_upper > self.max_input_tokens:
                raise BudgetExhausted(f"input ceiling: the next call could add up to {input_upper} input tokens")
        if self.max_cost_usd is not None:
            if input_upper is None or output_upper is None or self.pricing is None:
                raise BudgetExhausted("cost ceiling set but the next call's cost bound is unavailable")
            bound = self.call_cost_bound(input_upper, output_upper)
            if self.cost_usd + bound > self.max_cost_usd:
                raise BudgetExhausted(f"cost ceiling: the next call could cost up to ${bound:.4f}, "
                                      f"${self.max_cost_usd - self.cost_usd:.4f} remain")
            self.pending_reservation = bound
        self.pending_input_bound = input_upper

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
        """Charged on reception, before any judgement of the answer: a
        truncated or unusable answer still cost tokens."""
        reservation, input_bound = self.pending_reservation, self.pending_input_bound
        self.pending_reservation = self.pending_input_bound = None
        if usage is None or "output_tokens" not in usage:
            self.usage_unknown += 1
            # Unknown consumption is charged at the full reservation.
            if reservation is not None:
                self.cost_usd += reservation
            if input_bound is not None:
                self.input_tokens += input_bound
            return
        self.output_tokens += int(usage["output_tokens"])
        self.input_tokens += int(usage.get("input_tokens", 0))
        if self.max_cost_usd is not None:
            p = self.pricing
            known = {"uncached_input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens"}
            if known <= set(usage):
                self.cost_usd += (usage["uncached_input_tokens"] * p["input_per_mtok"]
                                  + usage["cache_creation_input_tokens"] * p["input_per_mtok"] * p["cache_write_multiplier"]
                                  + usage["cache_read_input_tokens"] * p["cache_read_per_mtok"]
                                  + int(usage["output_tokens"]) * p["output_per_mtok"]) / 1e6
            else:
                # Without a breakdown, the reservation is charged.
                self.cost_usd += reservation if reservation is not None else 0.0

    def snapshot(self) -> dict:
        return {
            "model_requests": [self.model_requests, self.max_model_requests],
            "actions": [self.actions, self.max_actions],
            "output_tokens": [self.output_tokens, self.max_output_tokens],
            "input_tokens": [self.input_tokens, self.max_input_tokens],
            "cost_usd": [round(self.cost_usd, 6), self.max_cost_usd],
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
