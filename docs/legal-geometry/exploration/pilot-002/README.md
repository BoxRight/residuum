# Pilot 002: rerun of the integration after pilot 001, for authorisation

Status: **closed as an integration failure.** Executed once on 2026-10-15
under its own authorisation; the first request was rejected by the API
before inference (see *Result of the single run* at the end). It shows
nothing about exploration, learning or candidate quality. No second run
under this identifier.

## Purpose

Pilot 001 (`../pilot-001/`, closed as an integration failure) never reached
the model: the API rejected every request because the strict tool schemas
exceeded the provider's limit on union-typed parameters. Pilot 002 keeps
everything else fixed, so that a change of result can be attributed to the
integration. Its objective and technical acceptance criteria are those of
pilot 001.

## What is identical to pilot 001

`episode.json` differs from `../pilot-001/episode.json` only in the
episode id (`riggs-pilot-002`), the opaque manifest hash (`h-pilot-002`)
and the worker's workdir (`target/exploration/pilot-002/work`). A test
(`test_pilot_002_differs_from_pilot_001_only_in_identity_and_paths`)
checks this. So these are unchanged:

- provider `anthropic`, model `claude-sonnet-5-5`, `effort: medium`,
  `thinking: adaptive`, no refusal fallback;
- condition C, the baseline theory and dossier imports, the single
  documented historical observation, decision `D` and profile P1, the
  explorer instructions;
- 12 model requests, 10 explorer actions, 14 worker actions (4 technical),
  60 000 output tokens, 120 s per request with no retries;
- the cost ceiling with its prices and assumptions (below).

## The only differences: code, not manifest

From commit `fabd851`, exercised offline only:

1. **Mixed tool catalogue.** Four tools are strict (`inspect_theory`,
   `advance_session`, `begin_candidate_evaluation`, `request_valuation`),
   16 union parameters by a conservative count; the other seven are sent
   non-strict with the same full schemas. The catalogue satisfies the
   documented per-request limits (20 strict tools, 24 optional parameters,
   16 union parameters) by that conservative count. **Whether the API
   accepts it is what this run checks**; it also has an internal
   complexity limit that cannot be checked offline.
2. **Non-recoverable provider errors end the episode** after one request,
   as a technical abort, with HTTP status, error type, message and request
   id recorded.
3. **Supervisor classification**: a worker that exits 0 is never
   `worker_lost`; a nonzero controller is named by its recorded outcome.

Point 1 changes what the model sees: for the seven non-strict tools the
API no longer constrains the arguments to the schema, so malformed
arguments become possible. They are rejected by the local validator as
protocol errors, recorded, and cost a turn, as in every condition.

## Budget for authorisation

Proposed: the same **conditional ceiling of US$2.50**, enforced as in
pilot 001 (reservation of each call's bound before sending, unknown or
partial usage charged at the reservation, overrun recorded and blocking
further calls), under the same assumptions: the input bound (request body
bytes plus 4 000 tokens) covers the provider's tokenisation, the manifest's
prices are those billed, and usage is reported truthfully. If the first
assumption fails on a call, that call may exceed its reservation.

Measured offline: the first call's reservation is US$0.4608 (input bound
120 303, output bound 16 000 tokens). If the API rejects the catalogue
again, the episode now ends after that one request, with US$0.46 charged
internally.

This budget is separate from pilot 001's internal charge (US$2.3037),
which stays recorded there pending reconciliation with the console.

## Planned commands (none executed, none authorised)

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 \
ANTHROPIC_WORKSPACE_ID=wrkspc_… \
.venv-pilot/bin/python scripts/test_safe.py \
    --interactive-manifest docs/legal-geometry/exploration/pilot-002/episode.json \
    --interactive-ledger target/exploration/pilot-002/ledger \
    --interactive-deadline 1500
```

The SDK is already installed in `.venv-pilot` (`anthropic==1.14.0`, not
committed). One run; no automatic second run. The ledger and the runner
report are archived in `run/` whatever the outcome.

Acceptance, as in pilot 001: the probe passes, the controller ends with a
recorded outcome, no technical stop, the cost charged stays within the
ceiling and matches the reported usage, no `reservation_exceeded`, and the
worker exits 0. A first accepted request alone already answers the
integration question; the rest of the episode is the original pilot's
technical objective.

## Result of the single run (2026-10-15, 19:39:16 to 19:39:18 UTC)

Run from commit `99f1f86`. Records archived in `run/`: the ledger
(`run/ledger/`), the runner report (`run/runner-report/`) and its standard
output. No credentials appear in them.

- Preflight, vocabulary freeze and the idempotence probe passed before any
  call.
- **The first request was rejected with HTTP 400 `invalid_request_error`,
  before inference:** "The compiled grammar is too large, which would cause
  performance issues. Simplify your tool schemas or reduce the number of
  strict tools." Request id `req_011Cg4b8pfgs13WpVcUCvTn5`. The documented
  union limit was no longer the cause; this is the provider's internal
  complexity limit, which the README above names as not checkable offline.
- The corrections of `fabd851` behaved as designed: the error was recorded
  with its structured fields, marked non-recoverable, and the episode ended
  as `technical_abort` after one request (controller exit 4); the worker
  exited 0 and the supervisor named the end `controller_technical_abort`,
  not `worker_lost`.
- Accounting: one request, usage unknown, charged at its reservation
  (US$0.460757). This is the conservative internal charge, not a billed
  amount; reconciliation with the console pending, as for pilot 001.
- Runner: 1.84 s, 80 MiB cgroup peak, no memory events.

### What the run shows

The mixed catalogue fits the public limits by the conservative count but
the provider still refused the grammar compiled from it. The rejection is
confirmed; its precise internal cause is not identified.

*Correction (after review):* an earlier version of this paragraph
attributed the rejection to shared term, time and type definitions carried
by the strict tools. The archived request contradicts that: the four
strict tools (`inspect_theory`, `advance_session`,
`begin_candidate_evaluation`, `request_valuation`) were sent with empty
`$defs`. The explanation was withdrawn; no other is offered.

The run confirms the one-request abort, the structured error record and
the supervisor's classification. It gives no evidence about the model's
behaviour. The next attempt is `../pilot-003/`.

## Reconciliation

Pending: the billed amount for `req_011Cg4b8pfgs13WpVcUCvTn5`, from the
console, next to the internal charge, which stays as recorded.
