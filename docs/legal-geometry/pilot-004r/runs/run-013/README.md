# Pilot 004-R run-013: run-012 with the most capable model

This is the last exploratory run of 004-R. The contract, corpus, dossier
order, tools, automatic evaluation, minimal supports, budgets and loss are
those of run-012. Only the model block of the manifest changes:

| Field | run-012 | run-013 |
|---|---|---|
| `model_id` | `claude-sonnet-5-5` | `claude-fable-5-1` |
| `pricing` (USD per MTok) | 2 / 10, cache read 0.10 | 10 / 50, cache read 0.25 |
| `timeout_seconds` | 180 | 600 |

Unchanged: `max_tokens` 16000, `thinking: {"type": "adaptive"}`,
`output_config.effort: "medium"`, `max_cost_usd` 20, `max_model_requests` 400.

**Model.** Claude Fable 5.1 is Anthropic's most capable widely released
model. The prices are from the API reference (cached 2026-10-06). The cache
write multiplier is 1.25, as for every model, and cache reads cost $0.25 per
MTok. The identifier and the tool definitions are checked before the run by
`tools/pilot004r/preflight.py`, which makes no generation and costs nothing:

- it asks the Models API for the model;
- it counts tokens for the exact system prompt, tools and opening on Fable
  5.1 and on Sonnet 5.5.

**Compatibility with the current configuration.**

- Adaptive thinking is accepted, and on Fable 5.1 thinking is always on.
  Effort `medium` is a supported level, so effort is unchanged.
- The planning call uses `tool_choice: {"type": "none"}`, which Fable 5.1
  accepts. Only the forced `any`/`tool` choices are rejected, and the driver
  does not use them.
- The conversation is append-only and thinking blocks are passed back
  unchanged, as required by preserved thinking. The same driver ran under the
  same check with Sonnet 5.5 in run-012.
- Fable 5.1 requires 30-day data retention. An organisation without it gets
  a 400 on the first request, before any cost.
- No refusal fallback is configured, because a fallback would answer with
  another model. A refusal shows as a turn without a tool call and is
  recorded in the ledger.

**Why the timeout changed.** Fable 5.1 turns can be longer. An expired
socket timeout is not a ceiling in the driver: it would abort the run
mid-trajectory. The timeout is a transport limit; it changes nothing the
model receives.

**Contract identity.** `../run-013-local-check/` runs the run-012 local check
replay under the run-013 manifest. Its system prompt, opening and tool list
are identical to those of run-012's local check, and its report is identical
apart from the trajectory id.

**Cost estimate.** The run-012 trajectory used 44 uncached input tokens,
109,782 cache-write tokens, 731,516 cache-read tokens and 4,066 output
tokens.

- At Fable 5.1 prices, the same trajectory costs about $1.8, or $2.4 if the
  tokenizer counts 1.35 times as many tokens.
- A trajectory with several times more turns, which is what we hope to
  observe, should stay well under the $20 ceiling.
- The driver checks the ceiling before each request.
