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
