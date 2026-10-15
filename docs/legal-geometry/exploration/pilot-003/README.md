# Pilot 003: every tool non-strict, for authorisation

Status: **prepared, not executed.** Requires its own authorisation and
budget; no earlier authorisation covers any further call.

## Purpose

Pilots 001 and 002 (`../pilot-001/`, `../pilot-002/`) were closed as
integration failures: the API rejected the requests before inference,
first for exceeding the documented limit on union-typed parameters in
strict schemas, then for a compiled grammar too large, whose internal
cause is not identified. Pilot 003 stops tuning a compilation mechanism
whose internal limit is unknown: every tool is sent non-strict, with its
complete schema. Everything else stays as in pilot 002, so that a change
of result can be attributed to this single difference. Objective and
technical acceptance are those of pilot 001.

## The single experimental difference

`controller.model.tool_strictness: "none"` in `episode.json`. Besides it,
the manifest differs from pilot 002 only in the episode id
(`riggs-pilot-003`), the opaque hash (`h-pilot-003`) and the workdir.

The request differs from pilot 002's only in the `strict` flag of the four
tools that had it: `test_pilot_003_request_differs_from_pilot_002_only_in_strict_flags`
builds both requests, sets every flag of pilot 002's to false, renames its
identity, and checks the two are equal: same tools, descriptions, schemas
and `$defs`, arguments, instructions, theory, dossiers, observation,
decision, profile, model parameters and budgets.

What this changes, and what it does not:

- **Changed:** the provider no longer constrains the model's generation
  to the tool schemas. The model may emit malformed arguments.
- **Unchanged:** what Residuum accepts to execute. Every action is
  validated locally against the condition's schema and the contract before
  any worker call, and again by the Rust worker. A malformed call is a
  recorded protocol error that costs a turn and an explorer action; it
  never runs (`test_without_strict_tools_malformed_arguments_are_rejected_before_execution`).
- **Unchanged:** a provider configuration error ends the episode after one
  request (`test_without_strict_tools_a_provider_configuration_error_still_ends_after_one_call`).
  The mode is fixed by the manifest: no automatic change of mode and no
  retry after another rejection.

Turns lost to invalid arguments may increase; the ledger counts them as
`protocol_error` records, and they will be reported.

## Fixed, as in pilots 001 and 002

Provider `anthropic`, model `claude-sonnet-5-5`, `effort: medium`,
`thinking: adaptive`, no refusal fallback; condition C; baseline theory
and dossiers; the single documented historical observation; decision `D`,
profile P1; explorer instructions; 12 model requests, 10 explorer actions,
14 worker actions (4 technical), 60 000 output tokens, 120 s per request,
no retries; the probe before any call; the supervisor deadline of 1 500 s.

## Budget for authorisation

Proposed: the **conditional ceiling of US$2.50**, enforced as before, under
the same assumptions (the input bound covers the provider's tokenisation,
the manifest's prices are those billed, usage is reported truthfully) and
with the same possible overrun of one call. First-call reservation
measured offline: US$0.4608 (input bound 120 307, output bound 16 000
tokens). If the API rejects this request too, the episode ends after it.

Separate from the internal charges of pilot 001 (US$2.3037) and pilot 002
(US$0.460757), both pending reconciliation with the console.

## Planned command (not executed, not authorised)

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 \
ANTHROPIC_WORKSPACE_ID=wrkspc_… \
.venv-pilot/bin/python scripts/test_safe.py \
    --interactive-manifest docs/legal-geometry/exploration/pilot-003/episode.json \
    --interactive-ledger target/exploration/pilot-003/ledger \
    --interactive-deadline 1500
```

One run, no automatic second run; ledger and runner report archived in
`run/` whatever the outcome. Acceptance as in pilot 001. To report in any
case: whether the API accepted the request, the number of model requests,
protocol errors (malformed arguments) and valid actions, the outcome, and
the internal charge.
