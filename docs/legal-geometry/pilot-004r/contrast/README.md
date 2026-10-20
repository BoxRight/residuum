# Pilot 004-R: h1/h2 divergence map

Step 1 of the contrast agreed after run-003. No model was called. The worker
operations `minimal_supports` and `compare_partition`, the frontier analysis
and the engine's closure are used as they are. The explorer tools, the engine,
LegalGeneral, the corpus, the loss and the frozen contract are unchanged.

- Generated map (verbatim test output): `divergence-map.md`
- Counterfactual pool (test fixture): `counterfactual_pool.res`
- Local test: `src/bin/residuum/cli/pilot004r_contrast_tests.rs`,
  `cli::tests::pilot004r_contrast::pilot004r_contrast_h1_h2_divergence_map`,
  1 passed (runner report `20261020T213617472417Z-2973540`).

## Origin of the two hypotheses (run-003)

| | Rule | Origin | Status |
|---|---|---|---|
| h1 `obligationYieldsRight` | `obligation(id,s,c,o,b,au,e) ⇒ right(id,s,c,o,b,au,e)` | Brookside **revise**, generated **after the feedback** that listed the five missing `right` positions. It reproduces our formalization convention for B7–B11 (the obligated party is the subject of the `right`), so fitting B7–B11 confirms that the convention was recovered, not a legal orientation. | Valid; compatible with Brookside (5 exposures); no prospective support |
| h2 `obligationYieldsRightCreditor` | `obligation(id,s,c,o,b,au,e) ⇒ right(id,b,c,o,s,au,e)` | Casa Fiesta **explore**, an exploratory proposal **without any prior discrepancy**. | Valid; unexposed; discarded by the model by its own reasoning, not by any refutation |

## Method

- **Pool.** One Brookside stipulation (`adoptPolicy policyA`) with its filed
  obligation, plus a performance `Act` and a non-performance `~Act` by each
  party (`actJ` by jant, `actU` by usa). These atoms are counterfactual: they
  carry no valuation and are not documented facts.
- **Universe.** The joint closure of LegalGeneral + h1 + h2 over the pool. The
  admissible leaves are its seeded events (`Act`, `~Act`, `obligationFiled`,
  `right`). `right` is a seeded verb, so a seeded `right` is itself always a
  minimal support of a `right` goal.
- **Per goal.** Minimal supports under O (LegalGeneral alone), O+h1 and O+h2
  (`minimal_supports`). Then `compare_partition` over (h1, h2), giving the
  region pool, the signatures, the discriminant regions and the inclusion
  (`covers`). Then, on each minimal configuration, the full derivation chain
  of the goal under each theory, with rule, owning module, antecedents and
  substitution.
- **Current loss.** The unchanged scorer on the real Brookside dossier with
  {h1}, {h2} and {h1, h2}.

## Map

The minimal configurations listed are those that separate h1 from h2.
"Seeded right" stands for a configuration that contains the `right` itself.
O derives the goal from that configuration under both hypotheses.

| Goal (for `adoptPolicy policyA`) | Scored by the current loss? | Minimal configuration where they diverge | O+h1 | O+h2 | Chain where it derives |
|---|---|---|---|---|---|
| `right(jant, …, usa)` | **yes** (`right` is scored; B7–B11 are documented here) | {`obligationFiled(jant…)`} | derives | no | `filedObligation` → `obligationYieldsRight` |
| `right(usa, …, jant)` | **yes, but no valuation exists** in the corpus | {`obligationFiled(jant…)`} | no | derives | `filedObligation` → `obligationYieldsRightCreditor` |
| `compliance(actJ, jant, …, usa)` | no (would need extended positions) | **none**: same minimal supports under O, O+h1 and O+h2 | — | — | O already derives it by `obligationCompliance`. h1 only adds a redundant justification by `rightCompliance`. |
| `compliance(actU, usa, …, jant)` | no (would need extended positions) | {`obligationFiled(jant…)`, `Act(actU, usa…)`} | no | derives | `filedObligation` → h2 → `rightCompliance` |
| `enforceability(actJ, jant, …, usa)` | no (would need extended positions) | {`obligationFiled(jant…)`, `~Act(actJ, jant…)`} | derives | no | `filedObligation` → h1 → `rightEnforceability` |
| `enforceability(actU, usa, …, jant)` | no (would need extended positions) | {`obligationFiled(jant…)`, `~Act(actU, usa…)`} | no | derives | `filedObligation` → h2 → `rightEnforceability` |

`compare_partition` reports exactly these regions as discriminants for five
goals, and none for `compliance(actJ…)`. For each discriminating goal, the
hypothesis that derives the goal covers the other, because its region strictly
contains the region of the other hypothesis for that goal.

### Brookside under the current loss (real dossier, unchanged scorer)

| Committed | Loss | Missed | Excess | Unvalued derived | Exposures |
|---|---|---|---|---|---|
| {h1} | 0 | — | 0 | 0 | h1 correct on B7–B11 |
| {h2} | 5 | B7–B11 | 0 | 5 | none |
| {h1, h2} | 0 | — | 0 | 5 | h1 correct on B7–B11 |

## Observable divergences and ones that need more valuations

1. **Observable now.** Only `right(jant, …, usa)` in Brookside. It separates
   the hypotheses **as replacements**: {h2} alone loses 5 and {h1} alone loses
   0. It does not separate them **as an addition**. {h1, h2} also loses 0,
   because the five `right(usa, …, jant)` that h2 derives have no valuation.
   They are not attributed as error (`unvalued_derived 5`, no exposure), and a
   missing position is never attributed to any hypothesis. That is why h2 can
   be neither refuted nor confirmed with the current corpus.
2. **Expressible under the current loss, but without a valuation.**
   `right(usa, …, jant)`. A documented valuation, required or excluded, on the
   counterparty's `right` over the same stipulation would discriminate h2
   without changing the loss. No such valuation exists in the corpus.
3. **Would only appear with more valued positions.** `compliance(actU…)`,
   `enforceability(actJ…)` and `enforceability(actU…)`. These need documented
   performance or non-performance by one of the parties, plus a valuation on
   `compliance` or `enforceability`. Neither head is scored today. The loss is
   unchanged here.
4. **No divergence.** `compliance(actJ…)`: LegalGeneral already derives it
   from the obligation and the act. No experience about the obligated party's
   performance can distinguish h1 from h2 through `compliance`.

## Minimal counterfactual configurations that separate them

Each configuration is stated for one stipulation. None has a valuation, and
none is presented as a fact.

- **C1** {`obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, …)`}: h1
  yields the obligated party's `right`, h2 the beneficiary's (`right` head,
  scored).
- **C2** C1 + `~Act(actJ, jant, adoptPolicy, policyA, usa, …)`: only h1
  yields `enforceability` against jant.
- **C3** C1 + `~Act(actU, usa, adoptPolicy, policyA, jant, …)`: only h2
  yields `enforceability` against usa.
- **C4** C1 + `Act(actU, usa, adoptPolicy, policyA, jant, …)`: only h2 yields
  `compliance` by usa.

## Limits of interpretation

- **Formal divergence, not legal correctness.** The map shows where h1 and h2
  produce different consequences inside LegalGeneral. It does not show which
  orientation is legally correct. That takes an external valuation over one of
  these configurations, and the decision remains external to the LLM.
- **Dependence on our formalization convention.** The `right(jant, …, usa)`
  divergence is measured against B7–B11, which encode our decision that the
  obligated party is the subject of `right` (reservations R-a and R-b in
  `docs/pilot-004r-implementation.md`). h1 matches B7–B11 because it
  reproduces that convention.
- **The orientation already built into LegalGeneral.** `rightCompliance` and
  `rightEnforceability` read the `Act` of the `right`'s own subject. Under h2,
  the counterparty's act counts as compliance with a duty that the documents
  place on the obligated party. This is a consequence of the theory, not a
  valuation. Treating it as evidence against h2 would mean adopting the
  ontology as the criterion, and that has to be decided explicitly.
- **h1 and h2 are compatible.** They can be committed together without any
  contradiction (`inconsistent` does not appear). Only an excluded valuation
  would make one of them recalcitrant.
- **Scope of the pool.** It covers a single stipulation and the conduct
  variables of these two rules. The other four Brookside stipulations behave
  in the same way, by the symmetry of the rules (they were not enumerated).
  Configurations with other verbs (`prohibition`, `permission`,
  `omissionProhibition`) do not interact with h1 or h2, because neither rule
  reads or writes them.
- **Seeded `right` as a leaf.** Because `right` is a seeded verb, every
  `right` goal has itself as a trivial minimal support, and O shares those
  regions. The discriminant regions are the ones that do not contain the goal.
