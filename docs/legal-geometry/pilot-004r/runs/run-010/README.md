# Pilot 004-R run-010: automatic formal evaluation after each admitted draft

## The single change from run-009 (manifest flag `auto_evaluate`)

When `propose_rule` returns `hN: admitted`, the driver immediately sends
`evaluate_rule(hN, <current dossier>)` to the harness, with no model call.
The model receives the admission and the evaluation together, for example:

```
Result of your last action: h1: admitted
Automatic evaluation: h1 on brookside: certified
```

The view that follows carries that evaluation (`eN`, certificate, new events,
positions), like any `evaluate_rule`. This applies in explore and in revise.

- The automatic evaluation uses the existing `Evaluate` action of the harness,
  so it is certified and recorded like a requested one. It counts against
  `max_verified_evaluations`, takes one action of the phase, and can be cited
  by a finding.
- It never returns valuations or loss, and it changes neither the committed
  set nor the scoring.
- In the driver ledger, the action is recorded as `automatic_action` and the
  combined result as `automatic_result`. The plan trace counts these
  evaluations separately (`automatic_evaluations`), not as the learner's
  consultations.
- The contract text says so in one sentence of the explore section and one in
  the `propose_rule` description (`contract.diff`).

Motivation (run-009): the learner drafted h1 before feedback, h1 was admitted,
and the learner ended the phase without evaluating it. Its stated reason was
*"speculative rights rule is untested against documented positions"*.

Everything else is run-009: investigation paragraph, planning step, draft
contract, findings, tools, budgets, loss, corpus, valuations, model and effort.

## Local verification (simulated provider, no cost)

- `../run-010-local-check/` is the run-009 scripted check with the flag. Each
  of the two admitted drafts is evaluated automatically (`e1`, `e2`). The
  script's own evaluation becomes `e3`, so there are 3 verified evaluations.
  `e1` shows the five `right(saB_j, jant, …, usa, …)` new events, and the
  finding citing `e1` is still verified. Prequential results equal the
  run-009 check.
- Without the flag, the local checks of run-007, run-008 and run-009 reproduce
  their ledgers and reports exactly. Run-006 reproduces its ledger; its report
  differs only by the counters `findings`, `findings_verified` and `reports`
  (all 0), which run-007 added to the report.

## Result (paid run, `run/`)

18 requests, US$0.427. Prequential loss 5 (B7–B11 missed on Brookside), then
0 on Casa Fiesta and 0 on Flowers. The losses are as in run-003 to run-009,
but the trajectory is different. There were 5 attempts and 4 admitted drafts,
each evaluated automatically (4 verified evaluations). There were 3 commits,
no navigation, no learner-requested `evaluate_rule`, no comparison, no
discard and no findings.

### Brookside explore: a draft, its evaluation, a commitment before feedback

The learner drafts the creditor form, subject and beneficiary swapped (the
form run-003 called h2, here h1): *"Obligations of debtor yield a mirror right
for the creditor."* The automatic evaluation `e1` shows the five
`right(usa, …, jant)`. The learner then commits it in the same phase
(40 thinking tokens), giving the reason *"Creditor right mirrors obligation;
plausible documented positions."* Unlike runs 003–009, it does not abstain.
It still drafts no alternative and runs no comparison. The commitment misses
B7–B11: the five usa rights are unvalued, not excluded (`unvalued_derived` 5).

### Brookside revise: correction from feedback

The learner drafts the same-subject form (h2 = h1 of the earlier runs):
*"Documented rights have the obligor as subject."* It commits h2 alone:
*"drop h1 whose rights were not documented."*

### Casa Fiesta explore: drafts revised from their formal consequences

With no feedback available (Casa Fiesta has 0 valued positions), the learner
drafts a new kind of rule: an Act performed under an agreement gives an
obligation.

- h3 is rejected by elaboration (unused parameter `saU`) and corrected in h4:
  *"Remove the unused parameter."*
- Its automatic evaluation `e3` shows, besides the obligations for cfA1 and
  cfA2, obligations to `agree` itself:
  `obligation(saCF_c, casaFiesta, agree, saDocCF, usa, …)`,
  `obligation(saCF_u, usa, agree, saDocCF, casaFiesta, …)`.
- The learner drafts h5, two rules restricted to `adoptPolicy` and `post`:
  *"Narrow version avoiding obligations to agree."* Its evaluation `e4` drops
  exactly those consequences and keeps compliance for cfA1 and cfA2.
- It commits h2 and h5, and leaves h4 uncommitted without a `discard`.

This is the first revision in the pilot driven by the formal consequences of
a draft rather than by feedback. Whether h5 is correct cannot be scored: Casa
Fiesta has no valued positions (4 unvalued positions derived), and Flowers
has no seeds.

### Flowers

The learner abstains, with no operation (empty dossier).

### Reading

Returning the evaluation with the admission changed what the learner does
with a draft. In Brookside it turned an untested draft into a commitment.
In Casa Fiesta it supported a draft–consequence–narrower draft cycle. The
learner still requests no consultation of its own (`evaluate_rule`,
`compare_partition`, navigation). It does not draft the alternative it
plans before committing, and it reports no findings. Its revise message also
says that h2's evaluation listed the five usa rights. `e2` contributes only the
jant rights; the usa rights come from h1, still committed at that point. The
learner guesses as much and says it did not check.
