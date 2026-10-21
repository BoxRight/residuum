# Pilot 004-R run-006: draft contract

The aim is to observe whether the model starts investigating provisional
hypotheses, testing alternatives and deciding which to commit, once
exploration is a means of understanding the dossier and no longer a risk
against the loss.

## Changes from run-005 (manifest flag `draft_contract`)

1. **Explore contract.** The explore paragraph and the task statement are
   rewritten; `contract.diff` is the exact diff against run-004/005.
   - Explore is presented as investigating the dossier's relations, the
     consequences of possible generalisations, and how alternatives differ.
   - `propose_rule` creates a draft with no effect on R_k that is never
     scored.
   - `evaluate_rule` is a hypothetical test without penalty.
   - `commit` is the decision to incorporate hypotheses into R_k so that they
     are contrasted with later experience.
   - The loss is described as the evaluator's contrast of the committed
     hypotheses, not as the learner's task. No rule, head, target or
     hypothesis is suggested.
2. **`discard {hypothesis, reason}`** records that a hypothesis is set aside.
   `commit` takes an optional `reason`. Both are traceability only: they
   execute nothing, are never checked and filter nothing. A discarded
   hypothesis can still be committed later.
3. **The view shows each hypothesis as `draft`, `committed` or `discarded`.**
4. **The mandatory question gate is removed** (`question_first` is not set).
   `pose_question` stays available as an optional record.
5. **Unchanged:**
   - stated abstention without any obligation to produce rules;
   - the loss, the valuations, LegalGeneral, the dossiers and Residuum's
     tools;
   - order, model, feedback, budgets (40 proposals, 40 verified
     evaluations, 30 turns per phase, 60 navigation calls, 4000 steps),
     US$20 safety ceiling and 400 requests.

## Compatibility

- Without `draft_contract`, the earlier protocols apply unchanged.
- Rerunning the local checks of run-003, run-004 and run-005 with this code
  gives identical reports and views.
- The driver regenerates the archived system prompts and tool catalogues of
  run-003, run-004 and run-005 byte for byte.
- Views of manifests without navigation no longer carry the empty
  navigation fields added in the run-004 integration.

## Local verification (simulated provider, no cost)

- `pilot004r_drafts_are_never_scored_discard_and_reason_are_traced` checks:
  - two drafts tested and not committed take no part in the prequential
    score (loss 5, no exposures);
  - discard and commit reasons are recorded in the ledger;
  - discarding an unknown hypothesis is refused;
  - the view marks the discarded draft.
- All Pilot 004-R tests (20, including contrast and role signatures) pass
  individually (runner report `20261021T020659496242Z-3216710`).
- `../run-006-local-check/` runs the scripted provider through the real
  bridge and driver:
  - in Brookside explore: an optional question, two drafts, two
    evaluations, one `compare_partition`, a discard with reason, and no
    commit before scoring (loss 5);
  - in revise: a commit with reason.
  
  The last view shows h1 `committed` and h2 `discarded`.
