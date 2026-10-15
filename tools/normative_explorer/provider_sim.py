"""Simulated model provider: scripted actions, no network, no inference.
It receives exactly what a real adapter would (visible history, tool
catalogue, output cap) and ignores it, returning the next scripted action
and a reported usage. It has no access to the manifest, the worker or the
ledger, so it cannot change theory, D, profile or condition: only actions
pass, and the controller validates each.
"""
from __future__ import annotations


class ScriptedProvider:
    def __init__(self, actions, output_tokens_each: int = 20):
        self.actions = list(actions)
        self.output_tokens_each = output_tokens_each
        self.calls = 0
        self.seen_tools: list[list[str]] = []

    def next_action(self, visible_history: list, tools: list[str], output_cap: int):
        self.calls += 1
        self.seen_tools.append(list(tools))
        if not self.actions:
            return ({"tool": "finish_episode", "arguments": {"candidate_ids": [], "explanation": "script exhausted",
                                                             "rationale": "nothing left", "expected_observation": None}},
                    {"output_tokens": self.output_tokens_each})
        action = self.actions.pop(0)
        if callable(action):
            action = action(visible_history)
        return action, {"output_tokens": self.output_tokens_each}
