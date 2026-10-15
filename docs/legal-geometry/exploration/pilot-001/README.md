# Pilot 001: first call to a real model, for review

Status: **prepared, not executed.** No request has been sent to a model
with this manifest, no dependency has been installed, and the latest review
does not yet authorise either. The manifest is `episode.json` in this
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
| Cost ceiling | **US$2.50, enforced** (next section) | |
| Network deadline per request | 120 s, no retries | From the manifest; every failure is charged and recorded. |
| Exchange deadline under the runner | `--interactive-deadline 1500` | The option exists only for real-provider manifests, up to 1 500 s; local exchanges keep their fixed 15 s. Memory, swap, CPU, growth and compile controls unchanged. Independent of the 120 s per request. |

## Cost: estimate and guaranteed ceiling

These are two different things.

**Guaranteed ceiling (US$2.50).** Before every call the controller asks the
provider for an upper bound of that call and reserves it:

- input tokens at most the UTF-8 bytes of the exact request body plus a
  4 000-token margin for what the API adds around tool definitions (a token
  never covers less than one byte);
- output tokens at most the request's `max_tokens`;
- the whole input bound priced as a cache write (the most expensive input
  rate), the output bound at the output rate.

A call is sent only if what was already spent plus that bound stays within
US$2.50. After the answer, the actual cost is charged from the reported
usage breakdown (uncached input, cache writes, cache reads, output). If the
usage is unknown, for example after a transport error, the full
reservation is charged. If the bound cannot be computed, nothing is sent.
The guarantee rests on three stated assumptions: the byte bound above, the
margin, and the prices written in the manifest being the ones billed.

**Estimate.** Measured offline on this manifest, the first call's bounds
are 120 296 input and 16 000 output tokens, a reservation of about US$0.46.
The actual first call should cost much less, since most of its input is a
cache write of about 30 000 to 40 000 tokens (a rough 3 to 4 characters per
token on 116 000 characters). Later calls are mostly cache reads. Because
reservations are conservative and grow with the history, the ceiling may
stop the episode before its 12 requests; that is the cost of a guarantee.

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
the ledger stays within the ceiling and matches the reported usage, and the
worker exits 0.
