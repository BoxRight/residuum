# Exploration analysis layer: verification record, 2026-10-14

Module `src/experiments/exploration.rs`; tests `src/experiments/exploration_tests.rs`.
Each test ran individually through `python3 scripts/test_safe.py --test-target lib`
with `CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0`, 1 GiB cgroup, zero swap,
one Cargo job, one harness thread. No full suite ran. Reports are under
`target/test-diagnostics/` of the worktree.

| Step | Report | Result |
|---|---|---|
| Compile lib tests (cold worktree, first success) | `20261014T233601114491Z-1878317` | exit 0, cgroup peak 744 MiB |
| Compile lib tests after P1/candidate additions | `20261014T233823505070Z-1880867` | exit 0 |
| `exploration_install_rejects_collisions_and_non_derived_heads` | `20261014T233833387538Z-1881182` | 1 passed |
| `exploration_profile_admits_riggs_catalog_and_reports_bounded_rejections` | `20261014T233834601180Z-1881315` | 1 passed |
| `exploration_frontiers_reproduce_certified_general_and_patrimonial_totals` | `20261014T233835607992Z-1881332` | 1 passed |
| `exploration_riggs_catalog_partition_and_discriminants_match_certificates` | `20261014T233836418502Z-1881365` | 1 passed |
| `cargo fmt` applied / checked | `20261014T233721406007Z-1880042` | exit 0 |

Hardening pass after review (configuration errors must never read as verdicts):

| Step | Report | Result |
|---|---|---|
| Compile lib tests after hardening | `20261015T001735546433Z-1909176` | exit 0, 693 MiB RSS |
| `exploration_install_rejects_collisions_and_non_derived_heads` | `20261015T001742727716Z-1909484` | 1 passed |
| `exploration_profile_admits_riggs_catalog_and_reports_bounded_rejections` | `20261015T001743786284Z-1909619` | 1 passed |
| `exploration_frontiers_reproduce_certified_general_and_patrimonial_totals` | `20261015T001744848594Z-1909633` | 1 passed |
| `exploration_riggs_catalog_partition_and_discriminants_match_certificates` | `20261015T001745660139Z-1909649` | 1 passed |

Controls added: `Decision::validate` (missing component or `y*` of the wrong
length is an `Err`, not `false`); `certify_addition` returns `Err` when a
declared new head is already produced by the reference, so the stripping
certificate is never applied outside its proven scope; `theory_preserved`
checks every theory declaration and origin and that additions claim no
theory module; `meet`/`finer` return `Err` on carriers of different length;
`Partition` carries hypothesis identifiers and refuses to combine partitions
whose identifiers differ or are ordered differently.

Two earlier cold compiles were stopped by the runner at a cgroup peak above
768 MiB (`20261014T233410568696Z-1876839`, `20261014T233511663184Z-1877672`);
the documented `CARGO_PROFILE_TEST_DEBUG=0` setting resolved it without any
change to the limits.

Oracles reproduced: `goal-frontier-results.json` totals (general 73/284 and
61/99; patrimonial 73/332 and 73/114 over the 73-event universe),
`riggs-rule-learning/partitions.tsv` (16 of 64 retained by the historical
valuation) and `riggs-rule-learning/questions.tsv` (per-case predictions of
the retained candidates). `riggs-separation-pool.tsv` records the separation
pool diagnostics; it is a report, not an oracle.

Source path (`src/experiments/proposal.rs`), after commit `f3091c7`:

| Step | Report | Result |
|---|---|---|
| Compile lib tests with the proposal module | `20261015T003114730417Z-1919215` | exit 0, 677 MiB RSS |
| `proposal_source_path_distinguishes_syntax_elaboration_profile_scope_and_evaluation` | `20261015T003143485814Z-1920146` | 1 passed |
| `exploration_install_rejects_collisions_and_non_derived_heads` (rerun) | `20261015T003146349847Z-1920184` | 1 passed |
| `exploration_profile_admits_riggs_catalog_and_reports_bounded_rejections` (rerun) | `20261015T003147411691Z-1920200` | 1 passed |
| `exploration_frontiers_reproduce_certified_general_and_patrimonial_totals` (rerun) | `20261015T003148425086Z-1920218` | 1 passed |
| `exploration_riggs_catalog_partition_and_discriminants_match_certificates` (rerun) | `20261015T003149235496Z-1920235` | 1 passed |

The generated private modules of the test live under
`target/exploration/proposals/Candidate<id>.res` next to their baselines.

## Increment 1: shared proposal stages, AST frontier and stepwise evaluation

Commit base `d17a194`. Same runner and settings. No engine, ontology,
fixture, limit, `main` or `v1` change.

| Step | Report | Result |
|---|---|---|
| Compile lib tests | `20261015T010516035771Z-1945907` (first test run compiled) | exit 0 |
| `proposal_source_path_distinguishes_syntax_elaboration_profile_scope_and_evaluation` | `20261015T010516035771Z-1945907` | 1 passed |
| `proposal_stages_separate_admission_from_evaluation_and_session_matches_full_closure` | `20261015T010518948480Z-1945927` | 1 passed |
| `forward_session_matches_close_program_on_riggs_candidates_and_geometry_fixtures` | `20261015T010520112045Z-1945964` | 1 passed |
| `exploration_install_rejects_collisions_and_non_derived_heads` (rerun) | `20261015T010522026559Z-1945981` | 1 passed |
| `exploration_profile_admits_riggs_catalog_and_reports_bounded_rejections` (rerun) | `20261015T010523044039Z-1946119` | 1 passed |
| `exploration_frontiers_reproduce_certified_general_and_patrimonial_totals` (rerun) | `20261015T010524053843Z-1946136` | 1 passed |
| `exploration_riggs_catalog_partition_and_discriminants_match_certificates` (rerun) | `20261015T010524916239Z-1946152` | 1 passed |
| `cargo fmt --check` | after the above | exit 0 |

Equivalence `ForwardSession::for_module` versus `close_program`
(`forward-equivalence.tsv`, 414 comparisons, all `equal`): 64 catalogue
candidates × 6 Riggs dossiers, the reference theory on the 6 dossiers, and
the 24 seeded experiments of `general`, `patrimonial` and `reference`.
Compared per completed closure: the set of full events, the multiset of
justifications (rule, ordered antecedents, canonical substitution) and the
rule origins; only `derivedAt` is normalised away. Rounds to the fixed point
range from 1 to 6. No counterexample file was produced; had one appeared it
would have been kept at `target/exploration/forward-equivalence-counterexamples.tsv`
and the test would have failed.

One failed run preceded the passing one (`20261015T010447304918Z-1945200`):
a stale `Candidatedeclares.res` from the previous commit's behaviour made
`!file.exists()` fail. `admit` now removes a previous private module under
the same id before deciding, so a rejected proposal leaves no stale file.

## Certificate as a precondition of acceptance

Review of `411ad1b` found that `conclude` returned `Evaluated` even when the
conservation certificate was computed but did not hold. Acceptance now
requires, jointly: completed closure, `certificate.holds()`, and only then
the reading of `D`. A failed certificate is the technical status
`certificate_failed` (`ProposalStatus::CertificateFailed(Conservation)`);
no compatibility is reported. Stages gained `closed` and `certified`.

Regression `proposal_certificate_failure_is_a_technical_status_not_a_verdict`:
a candidate with head `transferred`, absent from the Riggs reference closure
and therefore inside the certificate scope, is consumed by LegalGeneral
(`transferTitle`, `ownershipCapacity`) and produces events outside the
declared head; both the full-closure path and the stepwise session end in
`certificate_failed` with `events_preserved = false`.

| Step | Report | Result |
|---|---|---|
| `proposal_certificate_failure_is_a_technical_status_not_a_verdict` | `20261015T011521727919Z-1953698` | 1 passed |
| `proposal_source_path_distinguishes_syntax_elaboration_profile_scope_and_evaluation` (rerun) | `20261015T011522892348Z-1953846` | 1 passed |
| `proposal_stages_separate_admission_from_evaluation_and_session_matches_full_closure` (rerun) | `20261015T011525762965Z-1953896` | 1 passed |
| `forward_session_matches_close_program_on_riggs_candidates_and_geometry_fixtures` (rerun) | `20261015T011526880378Z-1953923` | 1 passed |
| four `exploration_tests` (rerun) | `20261015T011528800513Z-1953952`, `…529766439Z-1953978`, `…530732694Z-1954007`, `…531497669Z-1954035` | 1 passed each |
| `cargo fmt --check` | after the above | exit 0 |
