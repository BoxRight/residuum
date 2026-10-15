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
    pending_output_bound: int | None = None
    # Answers whose usage came without a complete input breakdown; charged
    # at the reservation.
    usage_partial: int = 0
    # Set when reported usage exceeds what was reserved for the call: the
    # bound did not hold, so no further call is reserved.
    overrun: str | None = None

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
        if self.overrun is not None:
            raise BudgetExhausted(f"reservation exceeded earlier: {self.overrun}")
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
        self.pending_output_bound = output_upper

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
        reservation, input_bound, output_bound = (self.pending_reservation, self.pending_input_bound,
                                                  self.pending_output_bound)
        self.pending_reservation = self.pending_input_bound = self.pending_output_bound = None
        if usage is None or "output_tokens" not in usage:
            self.usage_unknown += 1
            # Unknown consumption is charged at the full reservation.
            if reservation is not None:
                self.cost_usd += reservation
            if input_bound is not None:
                self.input_tokens += input_bound
            return
        output = int(usage["output_tokens"])
        self.output_tokens += output
        overruns = []
        if output_bound is not None and output > output_bound:
            overruns.append(f"output {output} > reserved {output_bound}")
        if "input_tokens" in usage:
            self.input_tokens += int(usage["input_tokens"])
            if input_bound is not None and usage["input_tokens"] > input_bound:
                overruns.append(f"input {usage['input_tokens']} > reserved {input_bound}")
        else:
            # A missing input count is unknown, not zero: charge the bound.
            self.usage_partial += 1
            if input_bound is not None:
                self.input_tokens += input_bound
        if self.max_cost_usd is not None:
            p = self.pricing
            known = {"uncached_input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens"}
            if known <= set(usage):
                cost = (usage["uncached_input_tokens"] * p["input_per_mtok"]
                        + usage["cache_creation_input_tokens"] * p["input_per_mtok"] * p["cache_write_multiplier"]
                        + usage["cache_read_input_tokens"] * p["cache_read_per_mtok"]
                        + output * p["output_per_mtok"]) / 1e6
                if reservation is not None and cost > reservation + 1e-12:
                    overruns.append(f"cost ${cost:.6f} > reserved ${reservation:.6f}")
                self.cost_usd += cost
            else:
                # Without a complete breakdown, the reservation is charged,
                # or the actual output cost if that is larger.
                floor = output * p["output_per_mtok"] / 1e6
                self.cost_usd += max(reservation or 0.0, floor)
        if overruns and self.overrun is None:
            self.overrun = "; ".join(overruns)

    def snapshot(self) -> dict:
        return {
            "model_requests": [self.model_requests, self.max_model_requests],
            "actions": [self.actions, self.max_actions],
            "output_tokens": [self.output_tokens, self.max_output_tokens],
            "input_tokens": [self.input_tokens, self.max_input_tokens],
            "cost_usd": [round(self.cost_usd, 6), self.max_cost_usd],
            "usage_unknown": self.usage_unknown,
            "usage_partial": self.usage_partial,
            "overrun": self.overrun,
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
