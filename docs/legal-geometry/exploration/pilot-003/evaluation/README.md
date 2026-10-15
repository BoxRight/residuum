# Pilot 003, stage I: independent evaluation of the candidate

Status: **done, accepted by the stage's criterion.** Offline: the worker
alone, no model, no network. Pilot 003's records in `../run/` are
unchanged.

## Question

Which function does the candidate `c1` of pilot 003 implement on the six
dossiers, computed by Residuum and not by the model:

```
p_{r0}(S_i) = 1[ hypotheticalSuccessionImpediment ∈ C_{T ∪ {r0}}(S_i) ]
```

with `T` the theory of pilot 003 (`riggs_problem_manifest.res`,
`riggs_cases.res`), `r0` the candidate text exactly as proposed
(`candidate.formal`), the six dossiers of the manifest, Horn addition as in
the pilot, and the observation channel `hypotheticalSuccessionImpediment`.
No new legal valuation is introduced.

## Method

`tools/normative_explorer/candidate_evaluation.py` builds two request
scripts; each runs through `scripts/test_safe.py --worker-manifest
--worker-requests` (one bounded process, 1 GiB, no swap, 15 s).

1. **Reproduction** (`reproduction-*.json*`): the eight worker envelopes
   of pilot 003 rebuilt from its ledger (vocabulary freeze, the probe's
   four sends, the three explorer actions), against pilot 003's own worker
   manifest with only the workdir changed. Each response is compared with
   the archived one under two stated normalisations: the transport strips
   `wire_version` before archiving, and the private workdir path differs.
2. **Evaluation** (`evaluation-*.json*`, own identity
   `riggs-pilot-003-evaluation`, larger action, session and step budgets):
   the candidate proposed once, then for each dossier the baseline closure
   `C_T(S)` (forward session, no candidate) and the hypothetical closure
   `C_{T ∪ {r0}}(S)` (candidate evaluation), each advanced until complete.
   Run twice.

For each dossier the table (`summary.json`) records completeness, whether
the channel holds, the rules reaching it, the number of justifications,
and whether `C_{T ∪ {r0}}(S)` restricted to the reference vocabulary (the
40 verbs other than the channel) equals `C_T(S)`, with justifications
compared as multisets (event, rule, antecedents, substitution; the step
label excluded) and known events compared as sets.

## Results

**Reproduction.** All eight archived responses reproduced identically
(`reproduction-comparison.json`).

**Evaluation.** 61 envelopes, no fault; the two runs are byte-identical
(`evaluation-output.sha256`).

| Dossier | `C_T` complete | channel in `C_T` | `C_{T∪r0}` complete | channel in `C_{T∪r0}` | reference preserved | justifications `C_T` → `C_{T∪r0}` |
|---|---|---|---|---|---|---|
| `causalLinkUnestablished` | yes | no | yes | **no** | yes | 4 → 4 |
| `historicalRiggs` | yes | no | yes | **yes** (rule `impediment`) | yes | 6 → 7 |
| `killingByThirdParty` | yes | no | yes | **no** | yes | 4 → 4 |
| `killingWithoutAcquisitionPurpose` | yes | no | yes | **no** | yes | 5 → 5 |
| `noBeneficiaryKilling` | yes | no | yes | **no** | yes | 4 → 4 |
| `survivalUnestablished` | yes | no | yes | **no** | yes | 0 → 0 |

So `Cov_D(r0) = {historicalRiggs}`. In `historicalRiggs` the only added
justification is the candidate's: `hypotheticalSuccessionImpediment(
palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private,
will1880) @ dossierTime`. In every dossier the candidate adds nothing to
the reference vocabulary and removes nothing: a conservative extension on
these six dossiers.

The model's untested expectation in pilot 003 ("historicalRiggs derives
the impediment; others do not") holds on this domain, now by Residuum's
evaluation.

**For stage II, not interpreted yet.** Derived verbs of `C_T` per dossier:
`testamentaryEntitlement` holds in five dossiers (all but
`survivalUnestablished`); `purposefulAcquisition` only in
`historicalRiggs`; `acquisitionLinkedToWrong` in `historicalRiggs` and
`killingWithoutAcquisitionPurpose`.

## Acceptance

Criterion: six complete, reproducible results with derivation records and
preservation of the reference. Met: six complete in both closures, two
byte-identical runs, the archived pilot exchange reproduced, reference
preserved in all six. No unexplained discrepancy.

## Files

- `candidate.formal`: the candidate text, as proposed.
- `reproduction-manifest.json`, `evaluation-manifest.json`: worker
  manifests as run (absolute paths of this worktree).
- `*-requests.jsonl`, `*-output.jsonl`: scripts and worker protocol output.
- `reproduction-comparison.json`, `summary.json`, `evaluation-output.sha256`.
- `runner-reports/`: the runner's report and result for the three runs
  (`20261015T202300464409Z-2798859`, `20261015T202329841521Z-2799272`,
  `20261015T202330624428Z-2799282`).

Test: `test_candidate_evaluation` rebuilds both scripts from the ledger and
the candidate, the reproduction comparison and the table from the archived
outputs, and checks the facts above.
