# Pilot 004-R run-011: evaluations against a stated baseline, and commit traces

## The change from run-010 (manifest flag `attribution`)

An observability change. The decision protocol, the loss, the corpus and the
available tools are unchanged.

**Why.** In run-010, `e2` (h2 on Brookside, with h1 committed) showed in
`new_events` ten rights: the five of h2 and the five of h1. `new_events` and
`activated_base_rules` are taken against `O` alone, while
`positions_contributed` is taken by dependency on the hypothesis's rules. The
learner read that correctly but could not check it from the view.

**1.1 Explicit baseline.** Every evaluation states `baseline`: `O` plus the
committed hypotheses other than the evaluated one.

**1.2 Attribution.** `added_events` lists the events of the evaluated theory
that are absent from the baseline closure. `contributed_justifications` lists
the justifications absent from the baseline closure, each with `rule`,
`origin` (`O` or the owning hypothesis), `antecedents`, `substitution` and
`event_already_in_baseline`. Together they cover:

- the hypothesis's own rules;
- the rules of `O` that fire on the new antecedents;
- the rules of committed hypotheses that fire on them (in Casa Fiesta, h2 on
  h5's obligations);
- new support for events the baseline already derives.

With the flag, these replace `new_events` and `activated_base_rules`.
`positions_contributed` is kept (positions that depend on the hypothesis's
rules). The `evaluate_rule` description says so (`contract.diff`); the system
prompt is unchanged.

**2.1 Commit traces** (report only, never shown to the learner). For every
commit, the harness records what the learner's view held when it asked:

- `before` and `requested`, with the `result` and the `reason`;
- `feedback_seen`, and `feedback_on_this_dossier_seen`;
- every evaluation visible, with id, dossier, baseline and status;
- `requested_evaluated_on_this_dossier`;
- the navigation actions and findings seen so far.

**Implementation.** The harness always computes the new fields; the bridge
renders them only with the flag. The final report gains evaluation ids,
baselines, attribution and `commit_traces`, also only with the flag.

Everything else is run-010: automatic evaluation, investigation paragraph,
planning, draft contract, findings, budgets, model and effort.

## Local verification (simulated provider, no cost)

**`../run-011-local-check/`** replays the 17 learner actions and plans of the
paid run-010 with the flag. For `e2`, the acceptance criteria hold:

1. Baseline: `["O", "h1"]`.
2. Added events: the five `right(saB_j, jant, …, usa, …)` only.
3. Contributed justifications: five, `origin: h2`, rule `obligationRight`,
   each from its `obligation(saB_j, jant, …)`.
4. What the learner had seen: the commit trace of the Brookside revise shows
   `before: [h1]`, `requested: [h2]`, feedback on Brookside seen, `e1` (baseline
   `O`) and `e2` (baseline `O, h1`) visible, and h2 evaluated on Brookside by
   `e2`.

The same replay shows the following:

- **Brookside explore:** h1 was committed with `e1` as its only evaluation,
  and `feedback_on_this_dossier_seen: false`.
- **Casa Fiesta, the interaction:** `e4` (h5, baseline `O, h2`) attributes
  the rights on cfA1 and cfA2 to `h2`/`obligationRight`, firing on h5's
  obligations.
- **Casa Fiesta, the commit:** `requested_evaluated_on_this_dossier` is `[]`
  for h2 and `[e4]` for h5.

**`../run-011-local-check-existing/`** is the case the replay does not cover.
h1 (`sameA`) is committed, and h2 (`sameB`) has the same body and head. `e2`
has baseline `["O", "h1"]` and `added_events: []`. Its five contributed
justifications are `origin: h2` with `event_already_in_baseline: true`.

**Reproduction without the flag:**

- The same 17 actions reproduce the paid run-010 report. Only two things
  differ: the work directory path inside h3's elaboration message, and the
  usage counters.
- The local checks of run-007 to run-010 reproduce their ledgers and reports
  exactly. Run-006 reproduces its ledger; its report differs only by the
  findings fields that run-007 added.
- The 21 pilot004r tests pass individually.

## Result (paid run, `run/`)

13 requests, US$0.277. Prequential loss 0 on all three dossiers. This is the
first zero-loss trajectory of the pilot, and it does not come from the change
under test (see the reading below). There were 2 drafts, both admitted and
both evaluated automatically. There was 1 commit, 1 discard and 1 abstention,
with no navigation, no learner-requested `evaluate_rule`, no comparison and
no findings.

### Brookside explore: the correct form, committed before feedback

The learner's first draft is the same-subject form (`obligationMirrorRight`,
`obligation(…) <= right(…)` with the same arguments): *"Beneficiary's right
mirrors the obligation; test consequences."* Its automatic evaluation `e1`
has baseline `O`, five `added_events` `right(saB_j, jant, …, usa, …)`, and
five justifications `origin: h1`. The learner commits h1 at once: *"Test
mirror-right hypothesis against feedback."* The commit trace records
`feedback_on_this_dossier_seen: false` and `e1` as the only evaluation. B7–B11
are hit (loss 0). In revise, the learner reads the feedback (*"Zero
discrepancies. Keep h1."*) and ends the phase.

### Casa Fiesta explore: a probe discarded from its attribution

The learner drafts h2 (`actComplies`: an `Act` gives its own `compliance`),
*"Probe: performed acts as compliance in casaFiesta."* `e2` has baseline
`O, h1`, four `compliance` events added, four justifications `origin: h2`, and
`positions_contributed: []` (compliance is not a projected position). The
learner discards it: *"Contributes no positions; speculative."* This is the
first `discard` in the pilot. No commit follows; h1 stays committed.

### Flowers

The learner plans an `inspect_theory` and an `evaluate_rule`, then abstains
without either: *"Flowers has no seeds and an empty closure, so no rule can
act on it."*

### Reading

- **The zero loss is not evidence for the change.** Brookside explore is the
  decisive step. There, the first evaluation has baseline `O` alone, so the
  attribution fields carry the same events as run-010's `new_events`. The first
  request is identical to run-010's; only the `evaluate_rule` description
  differs. The learner drafted the same-subject form first, where run-010
  drafted the swapped one first. That is a different sample, not a different
  reading of the results.
- **Where the change was visible**, in Casa Fiesta, the learner read the
  attribution correctly. The added events were its own compliance events, it
  saw no positions depending on them, and it discarded the draft instead of
  leaving it uncommitted as h4 was in run-010.
- **The exploration pattern is unchanged.** One draft per dossier, committed
  or discarded on its automatic evaluation alone. No alternative was drafted
  and nothing was compared. Planned consultations went unexecuted in all three
  dossiers. There were no findings.
- **The commit trace answers criterion 4 directly.** The only commit was made
  before feedback, with a single evaluation.
