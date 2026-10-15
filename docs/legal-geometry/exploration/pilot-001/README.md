# Pilot 001: first call to a real model, for review

Status: **prepared, not executed.** No request has been sent to a model
with this manifest. The manifest is `episode.json` in this directory; it
validates with `tools/normative_explorer/manifest.py` and its frozen hash
is printed by the controller at start.

## Objective

Technical and bounded: show that a real model, through the verified
circuit (controller, worker, wire, transport, runner), issues valid tool
calls, gets at least one proposal through the source path, and drives an
evaluation to a certified result, with every request, answer, cost and
failure recorded. It is not a measurement of normative learning, not an
A/B/C comparison, and uses no new valuation: only the documented
historical observation already in the repository.

## Decisions, each to confirm

| Decision | Proposed value | Why |
|---|---|---|
| Provider | `anthropic` | The only live connection; OpenAI has an adapter but no client factory, and choosing a provider is not settled by that availability. |
| Model | `claude-opus-5-5` | Current default Opus. The model is part of the episode identity; a change starts another episode. |
| Effort and thinking | `effort: medium`, `thinking: adaptive` (reasoning text not returned) | Medium is this model's default, set explicitly; thinking cannot be disabled on it. |
| Refusal fallback | off | A fallback would let another model answer inside the episode. A refusal is recorded as a model error with its category. |
| Condition | C | Exercises every tool path the circuit offers. A or B would send less. |
| Requests and actions | 12 model requests, 10 actions | Enough for inspect, a few queries, two proposals and their evaluation. |
| Output tokens | 60 000 for the episode, at most 16 000 per request | Thinking counts as output. |
| Input tokens | 600 000 ceiling, checked before each request | The overshoot is at most one request's input. |
| Network deadline per request | 120 s, no retries | From the manifest; every failure is charged and recorded. |
| Exchange deadline under the runner | **15 s today; proposed 1 500 s for this pilot only** | 12 requests × 120 s plus local time. Not applied: it needs a runner option, reviewed separately, accepted only for a real-provider manifest. Memory, swap, CPU and compile limits unchanged. |

## Estimated cost ceiling

Measured offline on this manifest: the opening request is about 116 000
characters (system 1 760, tools 34 800, opening message 79 500 with the
vocabulary package and the dossier seeds). In the archived simulated run,
condition C views averaged about 6 700 characters and peaked at about
30 000. Characters are not tokens; a rough conversion of 3 to 4 characters
per token puts the opening at 30 000 to 40 000 tokens. An exact count
needs the token-counting endpoint, which is a network call.

At Claude Opus 5.5 rates (input $4, output $20 per million tokens; cache
reads $0.20; 5-minute cache writes at 1.25 times input, as the Anthropic
prompt-caching documentation states):

| Item | Bound | Worst case |
|---|---|---|
| Input | ≤ 600 000 + one request (≈ 100 000) | ≈ $3.50 if every token were a cache write; ≈ $2.80 uncached |
| Output | ≤ 60 000 + one request (16 000) | ≈ $1.52 |
| **Total** | | **≈ $5.00** |

The prefix is append-only with top-level prompt caching, so the expected
cost is well below the ceiling. The ceiling is enforced from the usage the
provider reports, not from this estimate.

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

## Planned commands (none executed)

```sh
# 1. SDK in a virtual environment; the runner launches the controller with
#    its own interpreter in isolated mode, so it must run from this venv.
python3 -m venv .venv-pilot
.venv-pilot/bin/pip install anthropic==1.14.0

# 2. Credentials stay in the environment, never in the manifest.
export ANTHROPIC_WORKSPACE_ID=wrkspc_…      # the key is already exported

# 3. After the deadline option is reviewed and merged:
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 \
.venv-pilot/bin/python scripts/test_safe.py \
    --interactive-manifest docs/legal-geometry/exploration/pilot-001/episode.json \
    --interactive-ledger target/exploration/pilot-001/ledger
```

The run produces the ledger (every request body before the network, every
raw answer before parsing, usage, faults, the idempotence probe) and the
supervisor's report. Acceptance is technical: both processes exit 0 or the
controller exits with a recorded outcome, no technical stop, the probe
passes, and the costs reported match the ledger.

## Still open before running

1. Review of this manifest and of the decisions above.
2. A runner option for the exchange deadline of a real-provider episode.
3. Installing the SDK in the pilot's virtual environment.
4. Optionally, an exact token count of the opening request through the
   token-counting endpoint. It is a network call sending the same content;
   whether it is billed has not been verified here.
