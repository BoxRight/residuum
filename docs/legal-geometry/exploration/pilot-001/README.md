# Pilot 001: first call to a real model, for review

Status: **closed as an integration failure.** Executed once on 2026-10-15
under authorisation; every request was rejected by the API before
inference (see *Result of the single run* at the end). It shows nothing
about exploration, learning or candidate quality. No second run under this
identifier; a new run uses another identifier and its own authorisation. The manifest is `episode.json` in this
directory; it validates with `tools/normative_explorer/manifest.py` and its
frozen hash is printed by the controller at start.

## Objective

Technical and bounded: show that a real model, through the verified
circuit (controller, worker, wire, transport, runner), issues valid tool
calls, gets at least one proposal through the source path, and drives an
evaluation to a certified result, with every request, answer, cost and
failure recorded. It is not a measurement of normative learning, not an
A/B/C comparison, and uses no new valuation: only the documented
historical observation already in the repository.

## Decisions

Reviewed favourably, pending the budget and instrumentation corrections
now made:

| Decision | Value | Why |
|---|---|---|
| Provider | `anthropic` | The only live connection; OpenAI has an adapter but no client factory, and that availability is not an approved experimental choice. |
| Model | `claude-sonnet-5-5` | Half the price of Claude Opus 5.5, capable at multistep tool use, same API constraints as the adapter assumes. A change of model starts another episode with its own identity and budget. |
| Effort and thinking | `effort: medium`, `thinking: adaptive` (reasoning text not returned) | The model defaults to `high` with recalibrated levels; `medium` is the documented starting point for multistep tool use. |
| Refusal fallback | off | A fallback would let another model answer inside the episode. A refusal is recorded as a model error with its category. |
| Condition | C | Exercises every tool path the circuit offers. |
| Model requests and explorer actions | 12 and 10 | Enough for inspect, a few queries, two proposals and their evaluation. |
| Worker actions | 14 = 10 explorer + 4 technical | One to freeze the vocabulary, three for the idempotence probe. The entry point refuses a manifest without this room. |
| Output tokens | 60 000 for the episode, at most 16 000 per request | Thinking counts as output. |
| Cost ceiling | **US$2.50, conditional** (next section) | Requires explicit acceptance with its assumptions. |
| Network deadline per request | 120 s, no retries | From the manifest; every failure is charged and recorded. |
| Exchange deadline under the runner | `--interactive-deadline 1500` | The option exists only for real-provider manifests, up to 1 500 s; local exchanges keep their fixed 15 s. Memory, swap, CPU, growth and compile controls unchanged. Independent of the 120 s per request. |

## Cost: estimate and conditional ceiling

These are two different things, and neither is an absolute guarantee of
what will be billed.

**Conditional ceiling (US$2.50).** Before every call the controller asks
the provider adapter for an upper bound of that call and reserves it:

- input tokens at most the UTF-8 bytes of the exact request body plus a
  4 000-token margin for what the API adds around tool definitions;
- output tokens at most the request's `max_tokens`;
- the whole input bound priced as a cache write (the most expensive input
  rate), the output bound at the output rate.

A call is sent only if what was already spent plus that bound stays within
US$2.50; if the bound cannot be computed, nothing is sent. After the
answer:

- with a complete breakdown (uncached input, cache writes, cache reads and
  output all declared), the actual cost is charged;
- with usage unknown or only partly declared, the full reservation is
  charged; an absent or null count is unknown, never zero;
- if the reported usage exceeds what was reserved (input, output or cost),
  a `reservation_exceeded` record goes to the ledger and no further call
  is reserved: the episode ends as budget exhausted.

**The condition.** The ceiling holds only if three assumptions hold, and
the code does not prove them:

1. the input bound (body bytes plus 4 000 tokens) covers the provider's
   internal tokenisation and additions; it is a proposed conservative
   estimate, not a property of the tokenizer;
2. the prices written in the manifest are the ones billed;
3. the provider reports usage truthfully.

If (1) fails on some call, that call may cost more than reserved; the
overrun is detected from the reported usage and stops the episode, so the
excess is limited to that one call. If (2) or (3) fail, the controller
cannot detect it. The person authorising the budget accepts the ceiling
under these conditions.

**Estimate.** Measured offline on this manifest, the first call's bounds
are 120 296 input and 16 000 output tokens, a reservation of about US$0.46.
The actual first call should cost much less, since most of its input is a
cache write of about 30 000 to 40 000 tokens (a rough 3 to 4 characters per
token on 116 000 characters). Later calls are mostly cache reads. Because
reservations are conservative and grow with the history, the ceiling may
stop the episode before its 12 requests.

Prices in the manifest, for Claude Sonnet 5.5: input $2, output $10, cache
reads $0.10 per million tokens, 5-minute cache writes at 1.25 times input.
For comparison, the same reservations would allow about half as many calls
on Claude Opus 5.5.

The token-counting endpoint is free, but it returns an estimate rather
than a contractual count, and it sends the same content to the provider.

## Content that leaves the machine

Sent to the provider, all from the repository and public:

- the explorer instructions in the manifest;
- the opening message: episode id, condition, tool list, the decision `D`
  and the P1 profile, the six Riggs dossiers' seed events, the documented
  observation's dossier and status, the vocabulary package of the baseline
  theory (verb signatures, entities, constants, the reference rules), and
  the Formal syntax summary;
- the tool schemas;
- each later turn: the condition-C view of the previous result.

Not sent: the 64-rule catalogue, the documents and certificates in `docs/`,
other episodes, the API key and the workspace id (client configuration
only), file paths of private modules (projected out of every view).

Riggs v. Palmer is a famous case; the model may know its outcome. That is
acceptable for a technical pilot and is a known limit for any learning
claim (contract §7).

## Order of the episode

1. Preflight and manifest recheck.
2. Vocabulary frozen from the worker (one technical action).
3. Idempotence probe on a pending forward session (three technical
   actions). If it fails, the episode stops with exit 6 before any model
   call is paid.
4. The model loop, under the cost ceiling.
5. Ledger and supervisor report.

## Planned commands (none executed, none authorised yet)

```sh
# 1. SDK in a virtual environment; the runner launches the controller with
#    its own interpreter in isolated mode, so it must run from this venv.
python3 -m venv .venv-pilot
.venv-pilot/bin/pip install anthropic==1.14.0

# 2. Credentials stay in the environment, never in the manifest.
export ANTHROPIC_WORKSPACE_ID=wrkspc_…      # the key is already exported

# 3. The run.
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 \
.venv-pilot/bin/python scripts/test_safe.py \
    --interactive-manifest docs/legal-geometry/exploration/pilot-001/episode.json \
    --interactive-ledger target/exploration/pilot-001/ledger \
    --interactive-deadline 1500
```

Acceptance is technical: the probe passes, the controller ends with a
recorded outcome, no technical stop by the supervisor, the cost charged in
the ledger stays within the ceiling and matches the reported usage, no
`reservation_exceeded` record, and the worker exits 0.

## Result of the single run (2026-10-15, 19:22:28 to 19:22:31 UTC)

Records archived in `run/`: the episode ledger (`run/ledger/`), the
runner's report (`run/runner-report/`) and its standard output. No
credentials appear in them.

- SDK `anthropic==1.14.0` installed in `.venv-pilot` (not committed). The
  workspace id was supplied in the command's environment only.
- Preflight, vocabulary freeze and the idempotence probe ran before any
  call and passed (replay identical, no double advance, no extra action or
  step charged).
- **All five model requests were rejected by the API with HTTP 400
  `invalid_request_error`, before inference:** "Schemas contains too many
  parameters with union types (57 parameters with type arrays or anyOf)
  ... (limit: 16 parameters with unions)". Request ids
  `req_011Cg4ZrXD8r1oHUiqfC8XsW`, `req_011Cg4ZrZKPGKKvyTVpWUaLX`,
  `req_011Cg4ZraaYFQ9Lo3ANZgEwi`, `req_011Cg4ZrbjjsBb28HApsF9WV`,
  `req_011Cg4ZrcuCS9bMTdxpZjqja`. The model never answered; no action was
  taken; no proposal exists.
- Accounting: the 400 carries no usage, so each call was charged at its
  full reservation (US$0.4607 each, `usage_unknown` 5). After five,
  US$2.3037 was charged and the sixth reservation did not fit the ceiling:
  outcome `budget_exhausted`, controller exit 3. No `usage_partial`, no
  `reservation_exceeded`. US$2.3037 is the conservative internal charge,
  not a confirmed billed amount; the billed amount is not known and is not
  assumed. It stays recorded as charged until reconciled with the console
  (see *Reconciliation*).
- Runner: 3.26 s, 82 MiB cgroup peak, no memory events. The worker closed
  on EOF with exit 0 before the controller, and the supervisor classified
  the end as `worker_lost`. The exit order did not show that: the worker
  exited 0 and the controller recorded budget exhaustion. The archived
  report keeps that label as produced; the corrected runner names this end
  `controller_budget_exhausted`.

### What the run shows

1. The schemas are strict (`strict: true`) and contain many nullable or
   union-typed parameters. Offline tests validated their structure but no
   test checked the provider's limit of 16 union parameters per request.
   Counted locally, the eleven tools of condition C carry 93 union nodes
   (`begin_residual` and `begin_active_residual` 21 each); the API counts
   57 by its own rule. Either is far above 16.
2. A deterministic 400 was resent on each turn: the controller treats
   every provider error as one failed request and lets the model loop
   continue. An identical request cannot succeed on retry, so these five
   reservations bought nothing.
3. The circuit's protections did their job: nothing ran without a model
   answer, the probe ran before paying, the ceiling stopped the episode
   and every request and error is in the ledger.

The review also missed it: the fake-client tests checked internal
correspondence, not the provider's acceptance of the whole catalogue.

### Corrections after the run (offline, not exercised against the API)

1. Strict tools within the documented per-request limits (20 strict
   tools, 24 optional parameters, 16 union parameters). A conservative
   count, labelled as such, expands each `$ref` at every use; the cheapest
   tools are strict while the totals fit (`inspect_theory`,
   `advance_session`, `begin_candidate_evaluation`, `request_valuation`:
   16 unions), the other seven go non-strict with the same full schema.
   No term, binding or legal configuration is removed; local and Rust
   validation are unchanged. The preflight checks the whole catalogue when
   the provider is built. The API's internal complexity limit cannot be
   checked offline.
2. A non-recoverable provider error (`invalid_request_error`,
   authentication, permission, not found, too large) ends the episode as a
   technical abort after that one request, with HTTP status, error type,
   message and request id kept as structured fields.
3. The runner separates the controller's outcome from the worker's health:
   a worker that exits 0 is never `worker_lost`; a nonzero controller is
   named by its recorded outcome.

## Reconciliation

Pending: the billed amount for the five request ids above, from the
console, to be added here next to the internal charge, which stays as
recorded.
