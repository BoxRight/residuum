# Known header index: isolated polynomial-product experiment

Branch: `experiment-peano-after`, based on arithmetic checkpoint `a7822ef`.
Frozen `main` and `v1` remain at `8025bc6` and were not modified.
This follow-up changes only forward evidence access; no new arithmetic,
semi-naive scheduling, activation policy or provenance deduplication is added.

The profiling files from the preceding experiment remain in the worktree.
`src/derive/incremental.rs` has no new changes in this experiment: its earlier
test-only profiling remains, and its delta matcher is not indexed here.

## Implementation and scope

`src/derive/known_index.rs` stores an append-only index of Known positions:

```text
verb -> (polarity, arity) -> ordered positions in Known
```

The closure's existing `Vec<Event>` remains authoritative. Buckets contain
positions rather than cloned or normalized events, preserving the first supplied
representation and witness order. Initial unique seeds are indexed; each newly
derived logical event is indexed immediately before being appended to Known.
This allows later rules in the same round to observe it, as before.

Both ordinary and activated rule instances use indexed lookup in `close` and
`close_program`. Standalone matching/instantiation APIs and the incremental
delta matcher retain their linear access. The index does not schedule rules,
merge substitutions, inspect coefficients, filter temporal values or identify
justifications. All activated instances are still reevaluated each round;
duplicate candidate justifications are still rejected by the same linear scan.

Test builds can select the old linear evidence path in the same evaluator using
a scoped thread-local guard. It defaults to indexed access and is not a language
feature or production configuration. The linear comparison run also maintains
the index, so a small amount of index-maintenance overhead is included in that
run; its matching work is the old full scan. Historical pre-index results are
retained separately in `docs/peano-polynomial-product-profile.md`.

### Preservation argument and checks

`match_event` rejects every event whose verb, polarity or arity differs from the
pattern. The index returns exactly the ordered subsequence with those three
equal headers. Skipped events therefore cannot supply a match; all remaining
structural matching and substitution logic is unchanged.

Unit has the same result. Inductively, each side of And receives the same ordered
matches and substitutions. Thus each clause produces the same ordered candidate
vector. Activation, deduplication and stamping then receive the same inputs.
Since the index is updated alongside each append, the argument repeats at the
next rule evaluation and round. This explains why the optimization should
preserve the execution trace; it is not a termination or complexity claim.

Tests compare complete Debug representations of linear and indexed closures,
including events, proposition types/times, transition metadata, all witnesses,
substitutions, justification multiplicity and exact clock stamps. Comparisons
passed for `unitProduct`, `partialProduct` and all 16 subsets of the finite
`discrete_derivative.res` evidence universe (with duplicate seeds). That universe
also exercises Unit, cycles, positive/negative evidence, conjunctive antecedents
and new justifications for already known events.

An index-specific check compares bucket contents with ordered header filtering,
using pointer identity for the original Event representations. It checks both
polarities, mixed arities, duplicate positions, an absent verb and append updates.

## Direct small-product comparison

The same process computes the original linear path and indexed path separately
for `(1+2x)(2+x) = 2+5x+2x²`. Both return 223 events and 244 justifications.
The complete closure comparison passed.

| Measurement | Linear access | Indexed access |
|---|---:|---:|
| Event candidates examined by matching | 2,117,296 | 93,306 |
| Rejected by verb/polarity/arity | 2,023,990 | 0 |
| Rejected by arguments, bindings or temporal structure | 81,277 | 81,277 |
| Matching/instantiation time | 5,096.468 ms | 642.591 ms |
| Closure wall time | 5,196.620 ms | 733.676 ms |
| Fixpoint rounds | 36 | 36 |
| Clause evaluations | 6,375 | 6,375 |
| Activated rule instances | 226 | 226 |
| Candidate justifications generated | 6,246 | 6,246 |
| Duplicate candidate justifications | 6,002 | 6,002 |
| Provenance equality comparisons | 607,805 | 607,805 |

This run reduces examined evidence by 95.59%, matching/instantiation time by
about 7.93x and closure wall time by about 7.08x. It does not reduce generated
justifications or rule reevaluation. Timing ratios are observations on this
debug build, not guaranteed speedups.

The complete two-path comparison test took 6.5127 s and 18.09 MiB peak RSS.
A later standalone indexed profile returned identical work counts, taking
716.615 ms inside close and 1.3036 s for the whole test, including source loading.
The indexed Unit-product profile took 279.391 ms inside close; its exact
linear/indexed closure comparison also passed.

`structural_rejects` is a new test-only counter for failures after matching
headers: argument/constructor/record/binding incompatibility or temporal
incompatibility. It observes the same first failure and leaves binding behavior
unchanged. Historical reports before this counter cannot provide that count;
the direct small-product comparison measures both paths with the new counter.

## Same large case: still no completed fixpoint

The existing `principalProduct` was attempted once with the index, unchanged
15-second test timeout, existing binary and all original resource controls.
It timed out at 15.0074 s (exit -9), peaking at 32.69 MiB RSS. Cgroup memory
events recorded no OOM or OOM kill. No further verification was run after this
timeout. No limits were raised.

The last completed round advanced from 18 in the historical full-scan run to
48 in the indexed run. These partial outcomes are not two completed closures.
The large polynomial result and full final provenance remain unverified.

The 18 common completed rounds were compared mechanically from the two logs.
Every round has identical counts of known events, justifications, next-frontier
events, activated rule instances, clause evaluations, candidate justifications,
duplicates and provenance comparisons. Indexed probes equal linear probes minus
header rejects at every common round; indexed header rejects are zero.
No mismatches were found. These are aggregate-prefix checks, not a comparison
of serialized full provenance for the incomplete large case.

At the same completed round 18:

| Measurement | Historical full scan | Indexed |
|---|---:|---:|
| Known / justifications / next frontier | 530 / 639 / 11 | 530 / 639 / 11 |
| Event candidates examined | 5,530,664 | 218,862 |
| Header rejections | 5,311,802 | 0 |
| Matching/instantiation time | 13,831.323 ms | 1,344.721 ms |
| Activation time | 4.329 ms | 4.312 ms |
| Provenance deduplication time | 162.500 ms | 139.717 ms |

These timings come from different executions; they are not a controlled
whole-program speedup ratio. The previous run did not count structural rejects.
The indexed round-18 prefix records 204,073 of those rejects.

At the last completed indexed round, 48:

| Work/phase | Value |
|---|---:|
| Known / justifications / next frontier | 847 / 960 / 9 |
| Activated rule instances | 922 |
| Clause evaluations | 29,111 |
| Event candidates examined | 1,981,150 |
| Header rejections | 0 |
| Structural rejects | 1,915,374 |
| Generated candidate justifications | 31,693 |
| Duplicate candidates | 30,733 |
| Provenance equality comparisons | 11,642,448 |
| Matching/instantiation time | 12,894.946 ms |
| Activation time | 8.461 ms |
| Provenance deduplication time | 909.125 ms |

Work in the interrupted following round is not included. Matching and
instantiation still dominate. The remaining candidates share the same header
but often fail on bound arguments or record/time structure. Old rule instances
still repeat their searches and recreate old justifications. This experiment
does not attempt to remove that work.

If a future run completes the large profile, the verifier now requires every
output coefficient, not just the existing one-coefficient observation goal:
`[5,10,18,26,9,12,0]` in ascending exponent order, exactly one per exponent.
These values appear only in the verifier; the source inputs are unchanged and
no result coefficient is seeded. The current timeout never reached this final
assertion, so it provides no evidence of a successful large computation.

## Existing scenarios and resource controls

Twenty distinct selected tests passed individually; the large diagnostic variant
timed out and is not counted as passing. No full suite was run.

- Six index/profile tests: bucket integrity, two exact product comparisons,
  16 finite seed-subset comparisons, standalone indexed profile and the existing
  fixed-rule incremental/provenance comparison.
- Two existing library regressions: activation guards/trigger provenance and
  preservation of new justifications for old events with Unit axioms.
- All ten existing polynomial-product tests: complete small output, dense
  convolution, zero, unit, intermediate zeros, independent requests, partial
  application, product feeding addition, missing evidence and provenance replay.
- Two existing arithmetic regressions: polynomial addition of the principal
  inputs and recursive Peano multiplication `4*4=16`.

The product tests now complete individually in about 0.65–2.06 s, including
source loading and verification. Their highest peak RSS was 17.13 MiB.

The existing incremental matcher was deliberately left unchanged. Its small
fixed-rule update still examines 11 events and generates two justifications;
full recomputation now uses the index and examines six events, generating four
candidate justifications. Both still have identical logical/provenance results.
This tiny example is not a claim that one algorithm is faster; it demonstrates
that indexing close did not silently index or redesign the delta matcher.

All compilation, formatting and execution used `scripts/test_safe.py`:
1 GiB cgroup memory maximum, no swap, one Cargo job, one harness thread and
unchanged preventive controls. The first library compilation reported 802.75
MiB peak child RSS but 751.52 MiB cgroup peak; the runner's preventive 768 MiB
threshold uses the latter. The CLI compilation cgroup peak was 753.79 MiB.
Both compilations completed without OOM or a control change. Later library
recompilation reported 611.51 MiB child RSS. Compiled binaries were reused for
subsequent individual checks.

## Artifacts and reproduction

Diagnostics in the experimental worktree:

- `target/test-diagnostics/20261009T030242968151Z-3803660`: exact direct
  small-product comparison and finite seed-subset comparison.
- `target/test-diagnostics/20261009T040503981670Z-3847109`: four final index/profile
  checks, including Unit-product and unchanged incremental update.
- `target/test-diagnostics/20261009T040658609846Z-3848848`: ten existing product
  tests against the indexed production library.
- `target/test-diagnostics/20261009T042111893227Z-3859254`: existing activation and
  justification regressions.
- `target/test-diagnostics/20261009T043315286042Z-3868768`: two arithmetic regressions.
- `target/test-diagnostics/20261009T043332073631Z-3868967`: final format check.
- `target/test-diagnostics/20261009T043707827337Z-3872438`: one large indexed
  diagnostic, round logs, verified resource outcome and timeout.

Direct comparison:

```sh
python3 scripts/test_safe.py \
  --test derive::profile_tests::indexed_known_matches_linear_product_with_identical_provenance
```

Large diagnostic, only by deliberate opt-in and with the same limits:

```sh
RESIDUUM_PROFILE_PRODUCT=principalProduct python3 scripts/test_safe.py \
  --binary target/debug/deps/residuum-cfe44311058885c9 \
  --test derive::profile_tests::profile_polynomial_product_fixture
```

The binary path identifies this build and may change after recompilation.
No additional attempt at the large case was made. Raw diagnostic artifacts are
local and ignored by Git; the measurements above preserve the essential results.
