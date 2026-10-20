# Pilot 004-R run-003: protocol changes from run-002

Run-002 (`../run-002/`) showed that a larger budget and mandatory stated
abstentions did not start any exploration: the model abstained in every
explore phase and proposed nothing. Run-003 changes only the operative
objective of the phases and the dossier order. Tools, budgets, model, engine,
LegalGeneral and dossiers are unchanged.

## Changes

1. **Explore objective** (`tools/pilot004r/driver.py`, `instructions()`):
   the learner is told to investigate the relations it can observe and to
   propose and test provisional generalisations even when LegalGeneral shows
   no discrepancy yet, instead of waiting for a signal of error. No rule,
   head, target position or connection between verbs is suggested, and no
   valuation or reserved ground is revealed.
2. **Revise objective** (same place): the learner may revise, keep or discard
   hypotheses also when no dossiers remain. The prequential score of each
   dossier stays final; the rescoring of every presented dossier after a
   revision is recorded separately as a retrospective adjustment
   (`compatibility_after_each_revision` in the report). The harness already
   behaved this way; only the instructions now say so.
3. **Order**: Brookside → Casa Fiesta → Flowers (run-001 and run-002 used
   Casa Fiesta → Flowers → Brookside), so the only discrepancy available
   (B7–B11) is delivered before the last dossier. Losses are therefore not
   comparable position by position with run-002.

Unchanged: `claude-sonnet-5-5` (adaptive thinking, effort medium), feedback
condition, 40 proposals, 40 verified evaluations, 30 turns per phase,
2 clauses, US$20 safety ceiling, 400 requests.

## Local check before the paid run

`../run-003-local-check/trajectory.json` drives the same order with the
scripted provider through the real bridge (no network, no cost). The scripted
rules are test fixtures and never reach a model. Result (archived in
`../run-003-local-check/run/`):

- Brookside explore: proposal h1 admitted, evaluated (certified), committed;
  prequential loss 3 (B7, B8, B10).
- Brookside revise: h2 proposed with parent h1, evaluated on Brookside
  (certified, 5 positions), committed; retrospective Brookside loss 0, while
  the prequential loss stays 3.
- Casa Fiesta explore: an alternative h3 proposed and evaluated, h2
  re-evaluated, h2 kept; prequential loss 0.
- Flowers explore: stated abstention; Flowers revise: all hypotheses
  discarded (commit of the empty set); retrospective Brookside loss back to 5.

Observation for reading the report: the per-hypothesis statuses use the
latest retrospective rescoring of each dossier, so a hypothesis discarded at
the end shows as `Unexposed` there; its earlier retrospective exposures remain
in `compatibility_after_each_revision`.
