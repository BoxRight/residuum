# Pilot 004-R run-012: automatic minimal supports of an evaluation's goals

## The change from run-011 (manifest key `supports`)

This is step 1 of the geometric presentation: the goal selection
`G(h, S, B)` and its minimal supports, computed automatically for every
verified evaluation. Partitions and comparisons between hypotheses are not
part of it, and there is no new navigation policy. The decision protocol, the
loss, the corpus, the engine and LegalGeneral are unchanged. The learner's
tools are the same; only the `evaluate_rule` description changes
(`contract.diff`), and the system prompt is unchanged.

```json
"supports": {"goal_limit": 8, "max_queries": 40}
```

`supports` requires `attribution`. Both the driver and the bridge refuse a
manifest that has the first without the second.

### Baseline and dossier

Each evaluation of `h` on the dossier `S` is taken against
`B = O ∪ (R_k \ {h})`, where `R_k` is the committed set when it is
evaluated. The evaluation records `S` and `B` (`dossier`, `baseline`) as in
run-011.

### Goals

`G(h, S, B) = Events(J_h(S, B))`, where `J_h(S, B)` is the set of
justifications of the closure of `B ∪ {h}` on `S` that are absent from the
closure of `B`. Every added event (`Δ_h(S, B)`) has at least one such
justification, so `Δ_h(S, B) ⊆ G(h, S, B)`.

The goals are ordered deterministically:

1. events of `Δ_h(S, B)` before events that only gain a justification;
2. within each group, projected normative positions (`obligation`,
   `permission`, `prohibition`, `omissionProhibition`, `right`, the heads that
   valuations are written over) before other verbs;
3. finally, canonical Formal text, which is the same rendering the learner's
   view shows (`generative::formal_event`, now shared by the harness and the
   bridge).

**The priority of normative positions is a presentation policy of this
pilot, not a property of the geometry.** It only decides which goals fall
under the presentation limit. The support analysis does not depend on it.

### Selection, budget and omissions

- The first `goal_limit` goals are selected. The rest are
  `omitted_by_presentation_limit`.
- Each selected goal costs one support query from `max_queries`, which is
  counted over the trajectory (`counters.support_queries`). Selected goals
  past the budget are `omitted_by_budget`.
- A goal the analysis cannot answer is `omitted_by_analysis`, with
  `analysis_error`. This is the internal-limit category of the contract.
- The budget is separate from the navigation budget, so the learner's own
  navigation is not reduced and `navigation_actions` still counts only the
  learner's requests.

The learner's view gives, per evaluation, `minimal_supports` with:

- the number of goals and `goal_limit`;
- each answered goal with its supports;
- the three omitted lists.

The report adds the full ordered `goals`, `analysis_error` and, for each
answered goal, `checked`. The ledger has one `supports` entry per evaluation
with the counts.

### Scope of a support answer, and why it is exhaustive

The analysis is the algorithm behind the worker's `minimal_supports`
(`exploration::frontiers`), applied to `B ∪ {h}`. The worker's own tool can
only analyse `O ∪ {h}`, so it is not called.

- **Universe.** `E` is the closure of the seeds of `S` under `B ∪ {h}`. Every
  rule is grounded over `E`.
- **Leaves.** The leaves are the seeds of `S`. This differs from the
  worker's tool, whose leaves are all events in `E` with a Seeded verb. In
  LegalGeneral `right` is Seeded, so under the worker's choice a learned
  `right` would be its own support.
- **Result.** For each event of `E`, the antichain of minimal sets of leaves
  from which the grounded rules derive it. It is computed as a least fixpoint
  with no step, depth, size or time limit.

`frontiers` has no truncation: it either reaches its fixpoint or returns an
error, which here is `omitted_by_analysis`. The worker's response has no
truncation field, and it always answers `Complete`. Exhaustiveness rests on
two points:

- **Monotonicity.** The only negative literal is `~Act`, which is matched
  against explicit `EvidentialNot` seeds, not by absence. The closure of any
  subset of the seeds therefore lies inside `E`, and grounding over `E` loses
  no derivation.
- **The fixpoint.** It keeps every minimal combination of premise supports.

The answer is exhaustive relative to the theory `B ∪ {h}` and the seeds of
`S`. It does not cover other dossiers' seeds or hypothetical evidence. Run
limits (memory, deadline) abort the process; they do not truncate an answer.

**Independent check (`checked`).** The harness closes each support again
under `B ∪ {h}`. It checks that the support derives the goal, and that
removing any single seed stops it from doing so. By monotonicity, this is
minimality. Completeness, meaning that no minimal support is missing, is not
checked independently. It rests on the argument above.

## Local verification (simulated provider, no cost)

**`../run-012-local-check/`** replays the run-010 actions and plans, as
`../run-011-local-check/` does, with `supports` on.

- With `minimal_supports`, `support_queries` and the `supports` ledger
  entries removed, the report and all 75 turn payloads equal those of
  run-011-local-check. Only the `evaluate_rule` description in the `start`
  record differs.
- `e1` (h1, baseline `O`): 5 goals, the swapped rights `right(saB_j, usa, …,
  jant)`. Each has a single support, its `obligationFiled(saB_j, jant, …)`.
- `e2` (h2, baseline `O, h1`): 5 goals, the rights `right(saB_j, jant, …,
  usa)`, with the same single supports.
- `e3` (h4, baseline `O, h2`):
  - There are 16 goals: 4 obligations, then 4 rights, then 4 compliances,
    then 4 `do` events.
  - The limit selects the 8 obligations and rights and omits the 8 others by
    presentation.
  - The supports show what h5 then removed. Each `obligation(saCF_c, …,
    agree, …)` is supported by its own agreement `Act` alone. The cfA1 and
    cfA2 obligations need their `Act` plus `Act(saCF_c, …, agree, …)`.
- `e4` (h5, baseline `O, h2`): 8 goals, all answered, with no agreement
  obligations.
- All 26 answered goals are `checked`.

**`../run-012-local-check-limits/`** replays `../run-011-local-check-existing/`
(h1 committed, then h2 with the same body and head) with `goal_limit: 3` and
`max_queries: 4`.

- `e1`: 5 goals, 3 answered, 2 omitted by presentation.
- `e2`: `added_events` is empty, and its 5 goals are justifications on events
  the baseline derives. 1 goal is answered, 2 are omitted by budget and 2 by
  presentation.

The test `pilot004r_supports_select_goals_in_order_and_record_omissions`
covers the same case on Brookside with a non-performance seed. It checks:

- `enforceability` (an added non-position) after the five rights;
- the Formal order within the rights;
- both kinds of omission;
- that every support element is a seed;
- `checked`;
- the counter and the ledger entries.

**Reproduction of earlier runs.**

- The local checks of run-007 to run-011, and run-011-local-check-existing,
  reproduce their reports and turns exactly. The work directory path is
  normalised.
- Run-003 to run-006 reproduce their trajectories and views. Their reports
  differ only by fields that later runs added, all zero or `None`:
  `discarded`, `questions`, navigation and findings counters, and `findings`.
  Run-004 also differs by the absolute work directory, which was archived
  from a `/tmp` checkout.
- The paid run-010, replayed without flags, differs only by provider usage.
- The 22 pilot004r tests pass individually.

## Result (paid run, `run/`)

15 requests, US$0.388. Prequential loss 5: B7–B11 missed on Brookside, then 0
on Casa Fiesta and 0 on Flowers. There were 3 drafts, each admitted and
evaluated automatically with supports. There were 2 commits, no discard and 1
abstention. The learner requested no navigation and no `evaluate_rule` of its
own, ran no comparison and reported no findings. 18 support queries were
answered, with nothing omitted and every answer `checked`.

### Brookside: the run-010 path

- **Explore.** h1 is the swapped form (`creditorRight`, *"Give the creditor a
  right mirroring the obligation."*). `e1` (baseline `O`) has 5 goals, the
  `right(saB_j, usa, …, jant)`. Each has one support, its
  `obligationFiled(saB_j, jant, …)`. The learner commits h1 before feedback,
  giving the reason *"plausible documented right positions"*. The commit
  trace has `feedback_on_this_dossier_seen: false` and `e1` alone. Its plan
  listed `minimal_supports` and `compare_partition` among its operations, and
  it executed neither.
- **Revise.** After feedback, it drafts the same-subject form h2. `e2`
  (baseline `O, h1`) has 5 goals with the same single supports. It commits h2
  alone: *"Matches documented right orientation; drop h1 as unsupported."*

### Casa Fiesta: a probe left as a draft

h3 (`actRight`, an `Act` gives the right with the same arguments): *"Probe
whether performed Acts correspond to documented rights."*

- `e3` (baseline `O, h2`) has 8 goals: 4 rights, including rights to
  `agree`, and 4 compliances. Each has one support, its own `Act`.
- The learner ends the phase with an empty findings check.
- It neither commits nor discards h3, and it does not revise it.

### Flowers

The learner abstains: the dossier has no seeds.

### Reading

- **The supports carried no contrast in this trajectory.** All 18 answered
  goals have exactly one support of one seed. No support shows a dependency
  on a second fact, unlike `e3` in the local replay, where h4's obligations
  needed the agreement `Act`. The learner's drafts did not reach a case where
  the supports would discriminate.
- **No use of the supports is visible.**
  - The thinking blocks are redacted.
  - No text, reason or plan refers to supports.
  - Decisions follow the run-010 pattern: commit on the first evaluation,
    revise on feedback.
- **The Brookside sequence repeats run-010.** It also repeats the
  run-011 contrast: run-011 drafted the same-subject form first, and run-012
  the swapped form. The sampled first draft, not the presentation, decides
  the Brookside loss.
- **Exploration is unchanged.**
  - One draft per phase.
  - No alternative is drafted before committing, and nothing is compared.
  - All planned consultations went unexecuted, including `minimal_supports`
    as a tool.
  - h3 stays a draft without a decision.
