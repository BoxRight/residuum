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
