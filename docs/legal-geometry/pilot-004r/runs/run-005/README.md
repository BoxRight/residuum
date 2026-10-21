# Pilot 004-R run-005: question-first exploration

## The one protocol change from run-004

In explore, before proposing a hypothesis or stating an abstention, the
learner must pose an investigable question, consult Residuum about it, and
then decide in light of the result:

    Question → Consultation of Residuum → Result → Hypothesis or abstention

- `pose_question {question, tool}` records the question and the tool the
  learner plans to use. It executes nothing.
- A consultation is a navigation request sent to the worker or a verified
  evaluation (`evaluate_rule`) made after the latest question of the phase.
  Its relevance to the question is not judged; the record keeps both.
- Until a question and a consultation have happened in the current explore
  phase, `propose_rule` and `end_phase` are refused with their reason
  (`gate_refused` in the ledger). A question does not carry over to the next
  dossier.
- No proposal is ever required. Concluding that the evidence is insufficient
  is a valid outcome, stated as an abstention.
- Revise phases are not gated.
- If the turn limit closes an explore phase, the ledger says whether a
  question or a consultation was missing.
- The manifest flag is `question_first`. When it is absent, the run-001 to
  run-004 protocol applies (the run-004 local check reproduces its report).
  The flag requires navigation.

Instruction diff (the only text change; it appears only when the flag is on):

```diff
   The phase ends only after at least one proposal in it or with a stated abstention.
+  In explore, first pose an investigable question (pose_question) and name the tool that will answer it,
+  then consult Residuum with a navigation tool or evaluate_rule; in light of the result, propose a
+  hypothesis or state an abstention. Concluding that the evidence is insufficient is a valid outcome.
```

The `pose_question` tool is added to the catalogue. No rule, head, target
position or hypothesis is suggested.

Everything else is as in run-004: corpus, valuations, order Brookside → Casa
Fiesta → Flowers, model `claude-sonnet-5-5` (adaptive thinking, effort
medium), feedback condition, 40 proposals, 40 verified evaluations, 30 turns
per phase, 2 clauses, 60 navigation calls, 4000 engine steps, US$20 safety
ceiling and 400 requests.

## Local verification (no model, no cost)

- `pilot004r_question_first_gates_proposals_and_abstentions` checks:
  - refusals before a question and before a consultation;
  - an abstention accepted after a question and a navigation;
  - a proposal accepted after a question and an evaluation;
  - no carry-over of a question between dossiers;
  - revise is not gated;
  - the configuration error without navigation.
- All Pilot 004-R tests (19, including the contrast and role-signature tests)
  pass individually (runner report `20261021T015448380661Z-3198859`).
- `../run-005-local-check/` runs the scripted provider through the real
  bridge and driver: 3 questions, 3 navigation calls, 1 evaluation,
  2 refusals by the gate, 3 abstentions, 1 proposal in revise. Ledger
  archived in `../run-005-local-check/run/`.
