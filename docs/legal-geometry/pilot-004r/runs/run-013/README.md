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

## Preflight (`run/preflight.json`)

- **Models API:** 200 for `claude-fable-5-1` ("Claude Fable 5.1"), with
  1,000,000 input tokens, 128,000 output tokens and effort `low` to `max`.
- **`count_tokens`:** the exact system prompt, tools and opening were accepted
  on both models. The count was 12,245 tokens on Fable 5.1 and 12,245 on
  Sonnet 5.5, so the tokenizer ratio is 1.0.

## Result (paid run, `run/`)

17 requests, US$2.93 (driver estimate), 154 s. Prequential loss 0 on all
three dossiers. There were 3 drafts, all admitted and evaluated
automatically, and 14 support queries, all `checked` with nothing omitted.
The learner made 3 commits and 1 discard, with 1 voluntary navigation
(`inspect_theory`), no voluntary `evaluate_rule`, no comparison, and 4
findings over 3 reports. No ceiling was reached, no HTTP error occurred and
nothing was refused.

### Brookside explore: an alternative drafted to see its consequence, then discarded

- **h1** (`correlativeRight`) is the same-subject form: *"Every obligation has
  a correlative right of the beneficiary; orientation follows rightCompliance
  (subject = obligor, beneficiary = holder)."* `e1` (baseline `O`) adds the
  five rights. Each is supported by its `obligationFiled`.
- **h2** (`prohibitionRight`, a prohibition gives the right with the same
  tuple) is drafted *"to see the formal consequence, orientation uncertain."*
  `e2` adds `right(saB_u, usa, sue, premisesB, jant)`, supported by the
  prohibition.
- **The discard of h2:** *"right(saB_u, usa, sue, premisesB, jant) has the
  orientation that would make usa's suing a compliance under rightCompliance,
  contradicting the prohibition."* This is the first discard in the pilot
  reasoned from a draft's consequence together with a rule of LegalGeneral.
- **The commit of h1**, before feedback: the commit trace shows
  `feedback_on_this_dossier_seen: false`, with `e1` and `e2` visible. Loss 0.
- **Revise:** the learner abstains (*"0 discrepancies … no revision is
  needed"*).

### Casa Fiesta: a narrowed rule committed, then withdrawn for lack of confirmation

- **The plan** names two drafts. Rule A keys on the mutual agreement plus the
  performed `Act`. Rule B is narrower or different, *"to compare consequence
  sets"*. The plan notes that keying on the agreement alone and keying on
  agreement plus performance *"coincide on casaFiesta but diverge
  elsewhere"*. It also plans to evaluate on Brookside *"to check it adds
  nothing excluded there"*, and to run `compare_partition` and
  `begin_residual`.
- **h3** (rule A, two rules restricted to `adoptPolicy` and `post`) is the
  only draft. It requires both agreement `Act`s and the performed `Act`.
  - `e3` (baseline `O, h1`): obligation, right (through h1), compliance and
    `do` for cfA1 and cfA2.
  - Each goal has one support of three seeds: the performed `Act` and both
    agreement `Act`s.
  - Unlike run-010's h4, h3 derives no obligation to `agree`.
- **Rule B was not drafted**, and no `evaluate_rule` on Brookside, comparison
  or residual followed. The commit reason asserts that h3 *"adds nothing on
  brookside since no adoptPolicy/post Acts occur"*. That is true, but it was
  reasoned, not tested.
- **Commit `[h1, h3]`.** Casa Fiesta has 0 valued positions, so the loss is 0
  with 4 unvalued positions derived.
- **Revise:** the learner withdraws h3: *"casaFiesta had 0 valued positions, so
  h3 contributed nothing confirmable; retaining only h1 … keeps the theory
  minimal."* This revision comes from the absence of feedback, not from a
  consequence.

### Flowers

The learner plans `inspect_theory` and executes it. This is its first
voluntary consultation in the pilot. It then abstains: *"empty seed set
(confirmed by inspect_theory, record n9)."*

### Findings

The learner reported 4 findings. Three carry `contains_event` assertions on
`e1`, `e2` and `e3`, written as full events. All three events are in the
cited evaluations: the `right` on jant, the `right` on usa/sue, and the
obligation on cfA1. The verifier nevertheless returned `mismatch`, because the
verifier (`claims.check`) compares the value with event verbs
(`e["verb"] == value`), and the `end_phase` description does not say that the
value is a verb. This is an interface ambiguity, not a false claim. The
specification (`docs/llm-explorer-explanation-claims.md`) defines the rule as
"`contains_event` (pertenencia al resultado)", which is the learner's reading.
Re-run on the same records, `claims.check` gives `mismatch` for each full
event and `match` for the same claim written with only the verb. As agreed,
it is recorded here and in `../../closing-report.md` §6.1, and not fixed.

### Reading against the four questions

1. **Alternatives.**
   - Yes, in Brookside: h2 was drafted explicitly to see its consequence,
     and discarded by reasoning on it before feedback.
   - In Casa Fiesta the alternative was planned, with a stated point of
     divergence, but not drafted.
2. **Voluntary evaluations.**
   - None: all 3 verified evaluations were automatic.
   - The planned evaluation of h3 on Brookside was replaced by an argument.
3. **Geometric navigation.**
   - None. The only consultation was `inspect_theory`, which is an
     inventory.
   - `minimal_supports`, `compare_partition` and `begin_residual` were
     planned in Brookside and Casa Fiesta and not executed.
   - No text, reason or finding refers to the automatic supports.
4. **Revision from consequences.**
   - Yes, once: the discard of h2, from the consequence it adds read against
     `rightCompliance`.
   - h3 was designed to avoid the over-generation that run-010 had to revise.
     That was anticipated in the plan, not revised from a result.
   - The withdrawal of h3 in revise is driven by the missing confirmation,
     not by consequences.

Compared with Sonnet 5.5 (run-010 to run-012), Fable 5.1:

- drafted an alternative and discarded it with a stated consequence;
- wrote longer, more specific plans and reasons;
- reported findings;
- executed one planned consultation.

The structural pattern persists. Decisions follow the automatic evaluation.
Planned comparisons, evaluations and geometric queries are not executed. In
Brookside the correct form was again the first draft, so this run alone does
not separate model capability from sampling on that dossier.
