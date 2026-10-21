# Pilot 004-R run-007: findings reports

Purpose: let explore end with verifiable knowledge about the space of
hypotheses even when it ends with no learned rule. That knowledge must be
independent of the reserved valuations.

## Change from run-006 (manifest flag `findings_report`)

- **Report.** `end_phase` takes an optional `findings` report in the claims
  format of `tools/normative_explorer`: kind, text, and an assertion that is
  either null or `{ref, attribute, value}`. The format and its limits are the
  existing ones: `schemas._check("claims")`, at most 8 items per report.
- **Verification by the existing verifier, unchanged.** The driver checks
  each finding with `tools/normative_explorer/claims.py` (`claims.check`,
  condition C) against the records of the session:
  - navigation responses `n…`, in their worker wire form;
  - evaluations `e…`, recorded as `evaluation` payloads whose closure is the
    evaluated theory's closure.

  No new claim logic is added. The verdicts are the existing ones: `match`,
  `mismatch`, `unreferenced`, `out_of_view` and `unchecked`, plus
  `malformed` when the report does not have the claims format.
- **Novelty, diagnostic only and never presented as a proven property.**
  - Whether the cited record was produced in the current phase.
  - For a matched `contains_event`: whether that verb was already in the
    view's closure of that dossier at the start of the phase.
  - Every other case is recorded as `not_assessed`.
- **Accumulation and provenance.**
  - The harness keeps every finding with its dossier, phase, reference,
    assertion, verdict, detail and novelty.
  - The view shows all findings so far, and gives each evaluation its `id`
    so it can be cited.
  - The driver ledger keeps the raw records (`records`) the verdicts refer
    to.
- **Separation.** Drafts (hypotheses), findings (`findings`) and the
  committed set stay apart. Findings are never scored and their number is
  never rewarded.
- **Empty reports.**
  - An explicit empty report is recorded as `empty report`.
  - An `end_phase` without the field is recorded in the driver ledger as
    `report: absent`.
  - Neither is refused.
- **Unchanged:** the loss, the valuations, LegalGeneral, the engine, the
  corpus, the worker and its operations, the claims verifier, and the run-006
  contract and budgets.

`contract.diff` contains the instruction diff (one paragraph added) and the
`end_phase` schema diff.

## Limits of the reused verifier (as it is)

- `contains_event` checks the presence of a verb in the cited record, not a
  specific event. For example, a `right` derived with usa as subject
  satisfies `contains_event=right` just as one with jant does.
- `component` and `compatible` refer to the decision D of pilots 001–003,
  which does not exist in 004-R. They are answered `unchecked`.
- `differs_from` compares two complete records.

## Local verification (simulated provider, no cost)

- `pilot004r_findings_are_recorded_accumulated_and_never_scored` checks:
  - evaluation ids and closures;
  - verified and unverified findings and the empty report in the ledger;
  - accumulation in later views;
  - separation from drafts and commitments;
  - loss unaffected.
- All Pilot 004-R tests (21, including contrast and role signatures) pass
  individually (runner report `20261021T025942696458Z-3282206`).
- `../run-007-local-check/` runs the scripted provider through the real
  bridge and driver with real verification. In the Brookside explore report:

  | Assertion | Verdict |
  |---|---|
  | `e1 contains_event right` | `match`; novelty: produced in this phase, verb absent from the view at phase start |
  | `e2 contains_event right` | `match` (verb-level, see the limits above) |
  | `e1 differs_from e2` | `match` |
  | `n3 classes 2` | `match` (partition of h1/h2) |
  | `n5 contains_event right` | `match` (forward closure under h1) |
  | `e1 contains_event enforceability` | `mismatch` |
  | `n99 …` | `unreferenced` |
  | no assertion | `unchecked` |

  Casa Fiesta has an explicit empty report and Flowers a malformed report
  (`malformed`). The revise phases have no report (`absent`). The Casa
  Fiesta view shows the 8 findings already accumulated.
- With this code, the local checks of run-003 to run-006 reproduce their
  reports, views, system prompts and tool catalogues exactly.
