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

## In-process worker and DTOs (`src/experiments/worker.rs`)

Commit base `9b464d3`. Same runner and settings. No transport, serialisation,
controller or provider; no engine, ontology, fixture, limit, `main` or `v1`
change. Compile of the lib test target peaked at 756 MiB of cgroup memory
(`20261015T0135…`, before the test runs below), under the 768 MiB stop.

| Step | Report | Result |
|---|---|---|
| `worker_runs_a_scripted_episode_with_statuses_budgets_and_idempotency` | `20261015T013742732536Z-1972682` | 1 passed |
| `worker_budgets_are_enforced_as_errors_not_verdicts` | `20261015T013744153803Z-1972711` | 1 passed |
| `proposal_certificate_failure_is_a_technical_status_not_a_verdict` (rerun) | `20261015T013756839701Z-1972895` | 1 passed |
| `proposal_source_path_distinguishes_syntax_elaboration_profile_scope_and_evaluation` (rerun) | `20261015T013758152026Z-1972914` | 1 passed |
| `proposal_stages_separate_admission_from_evaluation_and_session_matches_full_closure` (rerun) | `20261015T013801018711Z-1972939` | 1 passed |
| `forward_session_matches_close_program_on_riggs_candidates_and_geometry_fixtures` (rerun) | `20261015T013802230796Z-1972975` | 1 passed |
| four `exploration_tests` (rerun) | `20261015T013804193923Z-1972991`, `…805153601Z-1973008`, `…806164450Z-1973025`, `…806977695Z-1973041` | 1 passed each |
| `cargo fmt --check` | after the above | exit 0 |

Two earlier failing runs of the scripted episode were the script's fault,
not the worker's, and are kept as findings: `20261015T013545100485Z-1969603`
(an unscoped residual query over `literalTestamentaryEntitlement` ends in the
engine's documented `unresolved query parameters` error, which the worker
mapped to `unresolved_parameters`; the script now asserts exactly that and
adds the scoped, typed-binding variant that completes) and
`20261015T013700788053Z-1970892` (typed bindings resolve constants against
the module graph, so the episode manifest must import the problem constants;
the script now imports `riggs_cases.res` besides the problem manifest).

## Worker: search-limit cap and complete audit output

Two small commits after review of `5676a6c`.

**Search limits (`f308464`).** Explicit `SearchLimits` in residual and
active requests may tighten the episode's but never widen `max_depth`,
`max_nodes` or `max_branches`; an excess is `budget: search limits …` and
opens no session. Scripted episode extended (`20261015T020239524579Z-1991602`),
budget test rerun (`20261015T020240939126Z-1991650`).

**Audit output.** `Payload::Forward` now carries every round advanced by the
action (`ForwardRound`: added events, stamped justifications, witness count)
and the full `known`; `Payload::Residual` carries every layer with its
search trees and metrics (`ResidualRound`), the complete structured result
on completion (`ResidualOutcome::{Horn, Active}`, including proofs and, for
active queries, the hypothetical closure) and the engine error text when the
session fails; `Payload::Evaluation` carries the candidate session's rounds
and, on completion, the closure with its derivations beside the report.
`EvaluationStep` exposes the round's events and justifications instead of
counts. The scripted episode now checks that the justifications accumulated
over the forward rounds of an action sequence equal those of `close_program`
on the same dossier.

| Step | Report | Result |
|---|---|---|
| `worker_runs_a_scripted_episode_with_statuses_budgets_and_idempotency` | `20261015T020521036628Z-1993835` | 1 passed |
| `worker_budgets_are_enforced_as_errors_not_verdicts` | `20261015T020522552364Z-1994005` | 1 passed |
| `proposal_stages_separate_admission_from_evaluation_and_session_matches_full_closure` (rerun) | `20261015T020523318153Z-1994033` | 1 passed |
| `proposal_certificate_failure_is_a_technical_status_not_a_verdict` (rerun) | `20261015T020524435710Z-1994060` | 1 passed |
| `proposal_source_path_distinguishes_syntax_elaboration_profile_scope_and_evaluation` (rerun) | `20261015T020525603479Z-1994089` | 1 passed |
| `forward_session_matches_close_program_on_riggs_candidates_and_geometry_fixtures` (rerun) | `20261015T020528575979Z-1994120` | 1 passed |
| four `exploration_tests` (rerun) | `20261015T020530492541Z-1994147`, `…531458811Z-1994174`, `…532527760Z-1994220`, `…533344195Z-1994251` | 1 passed each |
| `cargo fmt --check` | after the above | exit 0 |

Compile of the lib test target reached 793 MiB of process RSS with the
cgroup under the 768 MiB stop; the margin is now small and should be
watched before adding further test modules to the library target.

## Wire DTOs at the worker boundary (`src/experiments/wire.rs`)

Commit base `4101360` (dependency adopted). Same runner and settings.

**Memory regression, recorded before any reorganisation.** Registering
`wire.rs` and its tests in the library test target stopped the compile:

| Modality | Report | cgroup peak | Result |
|---|---|---|---|
| lib test target, incremental cache warm, `wire.rs` + `wire_tests.rs` | `20261015T024743096226Z-2027679` | 806 MiB | stopped (`near_memory_limit`) |
| lib test target, from scratch (no incremental cache), same content | `20261015T024813797959Z-2028061` | 772 MiB | stopped |
| lib test target, from scratch, `wire.rs` without `wire_tests.rs` | `20261015T024849494711Z-2028449` | 747 MiB | exit 0 (13.9 s, 771 MiB RSS) |

The module itself fits; its test module does not fit beside the other
library tests. The wire tests therefore live in the CLI test binary
(`src/bin/residuum/cli/exploration_wire_tests.rs`), which is where the
repository already keeps its heavy harness tests. No limit, runner or older
test was changed. The library target now has about 20 MiB of margin in the
from-scratch modality; the next addition to it should be measured first.

The first CLI compile in this worktree (`20261015T024946912437Z-2029524`,
768 MiB) was stopped while it built the plain library rlib and the test
binary in the same cgroup; a second invocation in a fresh cgroup finished
the test binary in 2.0 s at 442 MiB (`20261015T025019471898Z-2030400`).

| Step | Report | Result |
|---|---|---|
| `cli::tests::exploration_wire::wire_round_trips_every_request_variant_structurally` | `20261015T025039081471Z-2030863` | 1 passed |
| `cli::tests::exploration_wire::wire_round_trips_worker_responses_with_full_provenance` | `20261015T025039669686Z-2030888` | 1 passed |
| `cli::tests::exploration_wire::wire_rejects_malformed_oversized_deep_and_unknown_inputs` | `20261015T025041087698Z-2030923` | 1 passed |
| `serde_probe_round_trips_a_boundary_dto` (rerun) | `20261015T025041604511Z-2030952` | 1 passed |
| two `worker_tests` (rerun) | `20261015T025042115268Z-2031025`, `…043524952Z-2031113` | 1 passed each |
| four `proposal_tests` (rerun) | `20261015T025044338140Z-2031129`, `…045447563Z-2031148`, `…046560283Z-2031166`, `…049427365Z-2031204` | 1 passed each |
| four `exploration_tests` (rerun) | `20261015T025051341054Z-2031223`, `…052302634Z-2031243`, `…053313733Z-2031261`, `…054125149Z-2031278` | 1 passed each |
| `cargo fmt --check` | after the above | exit 0 |

What the wire tests check: structural round trip of every request variant
(Var versus Const, nested `after`, `none`, a record literal, both
substitution values, both scopes and parameter scopes, a negative
abducible, explicit limits, transition metadata that `Event` equality
ignores); structural round trip of every response of a real worker episode
(inventory, forward with all rounds, admitted and rejected candidates,
evaluation with closure, scoped Horn and active results with proofs and
hypothetical closure, an unresolved-parameters failure, supports,
partition, an error), with the justifications accumulated over forward
rounds compared record by record against `close_program` (ordered
antecedents, substitutions, `derivedAt`, multiplicity); and rejections:
oversized and malformed input as transport errors, nesting depth beyond
the limit before parsing, wrong wire version, unknown fields and variants
(outer and inner), invalid identifiers, negative counts, unknown stages and
an inconsistent partition as validation errors.

## Line transport (`src/experiments/transport.rs`)

Commit base `199aadf`. Same runner and settings. Service function over
injectable `BufRead`/`Write`, tested in memory; no process, socket, runner
change, controller or provider.

**Memory regression, recorded before any reorganisation and left as is.**
Adding `transport.rs` to the library pushed the library test target over
the runner's stop in every modality, and the CLI target over it when the
plain rlib and the test crate build in one cgroup:

| Modality | Report | cgroup peak | Result |
|---|---|---|---|
| lib test target, incremental, `transport.rs` added | `20261015T041643454603Z-2092002` | 772 MiB | stopped |
| lib test target, from scratch | `20261015T041731873377Z-2093036` | 770 MiB | stopped |
| cli target, rlib + test crate in one cgroup | `20261015T041649657712Z-2092025` | 770 MiB | stopped |
| stage 1, `--fixture … --cli run` (rlib + bin) | `20261015T041828368238Z-2093979` | 772 MiB | stopped |
| stage 2, `--compile-only --test-target cli` after stage 1 | `20261015T041842189529Z-2094367` | 768 MiB | stopped |
| per-test compile in the following invocations | `20261015T041859620291Z-2095071` and later | ≤ 452 MiB | completed |

Consequences, stated plainly: the library test target no longer compiles
under the runner in a single invocation, so the eleven library tests of
this branch (`exploration_tests`, `proposal_tests`, `worker_tests`,
`serde_probe_tests`) cannot be re-run until the regression is resolved;
their last passing reports predate `transport.rs`. The CLI target only
completes across invocations, each resuming where the previous stop left
the artefacts, which does not satisfy the open obligation of a reproducible
cold preparation within the limits. The decision (relocating this branch's
own tests to the CLI binary, gating the serialisation modules, or another
arrangement) is left open on purpose: this increment moved no further tests
and changed no limits.

Transport tests (CLI target), all individually:

| Step | Report | Result |
|---|---|---|
| `transport_serves_scripted_lines_in_order_matching_direct_worker` | `20261015T041944854892Z-2096162` | 1 passed |
| `transport_faults_are_framed_and_do_not_execute` | `20261015T041902944203Z-2095503` | 1 passed |
| `transport_output_limit_reports_instead_of_truncating` | `20261015T041903761221Z-2095531` | 1 passed |
| `transport_write_errors_stop_the_service_after_execution` | `20261015T041904626629Z-2095561` | 1 passed |
| three `exploration_wire` tests (rerun) | `20261015T041905492156Z-2095588`, `…906058276Z-2095615`, `…907328752Z-2095642` | 1 passed each |
| `cargo fmt` | `20261015T0419…` | exit 0 |

One failing run of the ordering test (`20261015T041859620291Z-2095071`) was
the script's: two workers with different work directories produce reports
whose `file` differs. They now share the directory.

What the transport tests check: a ten-line script served in order yields
one output line per input line, each response structurally equal to the
direct worker's for the same envelope, idempotent replay byte-identical,
reused identifier and unknown session as execution errors inside responses;
forward provenance through framing equals `close_program`; empty,
whitespace, malformed, unknown-field, over-long (with kept and discarded
counts), too-deep lines produce framing, transport or validation faults
without executing and without consuming actions; an unterminated final line
is a framing fault, not executed, stopping the service; an over-long
unterminated line is never accumulated beyond the limit; a response above
the output limit is reported as an `output_limit` fault naming the action,
nothing truncated or sent, the action having executed, with the same fault
on replay and the full response under a wider limit; a write error stops the
service after the request executed, and the answer is recoverable by replay.

## Boundary crate and reproducible cold preparation

Decision of the project author after the regression above: the
communication boundary becomes its own crate, `residuum-boundary`
(`boundary/`), depending on `residuum`; `residuum` does not depend on it.
`wire`, `transport`, the serde probe and their tests moved there with their
cases, comparisons and oracles unchanged (imports and organisation only).
`serde`/`serde_json` left `residuum`'s manifest. The exploration, proposal
and worker tests stayed in the library target. The runner gained one target
selection, `--test-target boundary`, in a separate commit: it compiles
`-p residuum-boundary --lib` and finds the test binary by target name;
isolation, limits and stop policy are untouched.

**Staged cold preparation, each stage one invocation that completes.**
Stages: `--compile-only --test-target lib`, then `boundary`, then `cli`.

| Scenario | Stage | Report | Elapsed | RSS | cgroup peak |
|---|---|---|---|---|---|
| A: project-cold (`cargo clean -p residuum -p residuum-boundary`, third-party deps kept) | lib | `20261015T042913111466Z-2103249` | 12.5 s | 675 MiB | 664 MiB |
| A | boundary | `20261015T042925724400Z-2103673` | 10.1 s | 460 MiB | 449 MiB |
| A | cli | `20261015T042935993784Z-2104215` | 8.4 s | 545 MiB | 612 MiB |
| B: fully cold (`target/debug` removed, dependencies rebuilt) | lib | `20261015T042944618580Z-2104643` | 16.2 s | 687 MiB | 712 MiB |
| B | boundary | `20261015T043000942888Z-2105192` | 17.5 s | 481 MiB | 510 MiB |
| B | cli | `20261015T043018522208Z-2105995` | 8.5 s | 554 MiB | 636 MiB |

All six exit 0 with no stop; no compile was resumed from a stopped one.
This is the first time in this worktree that a fully cold preparation
completed stage by stage. The library target is back to where it was
before the serialisation modules (664 MiB project-cold, as in the baseline
measurement), and the boundary crate compiles with more than 250 MiB of
margin.

Tests after the move, all individually:

| Target | Test | Report |
|---|---|---|
| boundary | `serde_probe_tests::serde_probe_round_trips_a_boundary_dto` | `20261015T043128908035Z-2107500` |
| boundary | `wire_tests::wire_round_trips_every_request_variant_structurally` | `20261015T043130574636Z-2107678` |
| boundary | `wire_tests::wire_round_trips_worker_responses_with_full_provenance` | `20261015T043131088100Z-2107706` |
| boundary | `wire_tests::wire_rejects_malformed_oversized_deep_and_unknown_inputs` | `20261015T043132306247Z-2107733` |
| boundary | `transport_tests::transport_serves_scripted_lines_in_order_matching_direct_worker` | `20261015T043132820842Z-2107760` |
| boundary | `transport_tests::transport_faults_are_framed_and_do_not_execute` | `20261015T043134587886Z-2107908` |
| boundary | `transport_tests::transport_output_limit_reports_instead_of_truncating` | `20261015T043135356477Z-2107935` |
| boundary | `transport_tests::transport_write_errors_stop_the_service_after_execution` | `20261015T043136122861Z-2107962` |
| lib | two `worker_tests` | `20261015T043050583707Z-2106833`, `…051943868Z-2106851` |
| lib | four `proposal_tests` | `20261015T043052753616Z-2106987`, `…053915270Z-2107004`, `…055025715Z-2107022`, `…057940207Z-2107062` |
| lib | four `exploration_tests` | `20261015T043059902689Z-2107078`, `…100860174Z-2107098`, `…101870858Z-2107114`, `…102678661Z-2107131` |
| all | `cargo fmt --check` | exit 0 |

One failing boundary run preceded these (`20261015T043047633480Z-2106667`
and the transport tests of that batch): the moved tests resolved fixtures
from the crate's manifest directory; they now resolve from the workspace
root.

The output-limit fault is now described as the review asked: the action
executed and its result exists in the worker, but it is not deliverable
under the episode's output budget; replaying the identifier yields the
same fault, and the limit is not raised during an episode.

## Minimal worker binary (`boundary/src/bin/residuum-worker.rs`)

Commit base `a843126`. Start-up logic in `residuum_boundary::startup::run`
over injectable streams; the binary only wires stdin, stdout and stderr and
exits with the returned code. Manifest DTO (`manifest::WireManifest`)
validated at the boundary and converted once to the worker's `Manifest`;
no request can modify it. The binary itself was **not** compiled or
executed: that waits for the runner's bounded worker mode to be reviewed.

Boundary test target compile with the new modules: `20261015T0439…`,
cgroup 444 MiB. Tests, individually (`--test-target boundary`):

| Test | Report | Result |
|---|---|---|
| `startup_tests::startup_serves_injected_streams_matching_the_direct_worker` | `20261015T043938711494Z-2113930` | 1 passed |
| `startup_tests::startup_exit_codes_and_diagnostics_are_documented_and_distinct` | `20261015T043940230420Z-2113957` | 1 passed |
| `startup_tests::manifest_dto_round_trips_resolves_paths_and_is_immutable_to_requests` | `20261015T043941397448Z-2114124` | 1 passed |
| eight earlier boundary tests (rerun) | `20261015T043942063790Z-2114153` … `20261015T043948482274Z-2114355` | 1 passed each |
| `cargo fmt --check` | after the above | exit 0 |

Exit codes: 0 clean EOF; 2 usage; 3 manifest unreadable, oversized,
malformed or invalid; 4 worker cannot start (imports do not elaborate);
5 input ended inside a line, that line not executed; 6 stream error. Every
diagnostic is one stderr line prefixed `worker:`; stdout carries protocol
lines only, checked by the tests. Tested: a six-request script served
through `run` equals the direct worker response by response; every exit
code with its diagnostic; manifest DTO round trip field by field; relative
paths resolved against the manifest's directory; oversized manifest and
zero budgets rejected before any work; an invented `set_manifest` request
rejected at the wire.

## Bounded worker mode of the runner (proposed, not executed)

`scripts/test_safe.py --worker-manifest <file> --worker-requests <file>`
builds `residuum-worker` (`cargo build --offline --locked -p
residuum-boundary --bin residuum-worker`, one job, 45 s) and runs it once
with the requests file on stdin (regular file, at most 1 MiB, ending in a
newline), protocol lines to `NN.stdout.jsonl` and diagnostics to
`NN.output.txt`, 15 s like a fixture run, the existing abnormal-growth
control extended to the `worker` stage. Compilation and execution run in
separate scopes, each its own cgroup with the same controls, as every stage
of the runner does. Memory and swap limits, CPU quota, job and thread counts
and the stop policy are unchanged; the mode takes only its two options and
rejects fixtures, tests, formatting and CLI options (review of `f600bb3`:
the 60 s budget was reduced to 15 s and the exclusions completed). `run_bounded` gained optional `stdin` and
`stdout` arguments, used only by this mode; existing stages keep their
behaviour. Only `python3 -m py_compile` and `--help` were run on the script.
The mode has **not** been executed and the binary has not been built: both
wait for review, after which the equivalence between direct worker,
in-memory service and bounded process is to be checked, provenance and
failure states included.
