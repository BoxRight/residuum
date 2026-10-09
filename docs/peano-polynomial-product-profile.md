# Polynomial product: observational execution profile

## Reproducible arithmetic baseline

Branch: `experiment-peano-after`.
Committed algorithm and fixtures:
`a7822efe8801597489c5eb8f7511206a84170f9b`
(`Add recursive polynomial product experiment using Peano programs`).

The profiling follow-up is separate from this commit. Neither `main` nor frozen
`v1` was modified; both still resolve to
`8025bc6b4afee3415c652a5830576ef18aaf8cb5`.

See `docs/peano-polynomial-product-experiment.md` for the source algorithm,
successful products, provenance checks and previous timeout observations.
No arithmetic primitive, lookup table or supplied result was introduced.

## Measurement scope

`src/derive/profile.rs` and `src/derive/profile_tests.rs` instrument the real
evaluator only under `cfg(test)`. Ordinary builds have no profiling code.
The probes do not change rule ordering, frontier scheduling, substitutions,
stamping, event identity, activation guards or equality of justifications.

The provenance membership probe performs the same ordered equality comparisons
and first-match exit as `Vec::contains`, counting the comparisons. When disabled,
it calls the original `contains` directly. There is no replacement evaluator.

The phases are disjoint at the measured level:

- Preparation: resolve and compile associated Programs and collect ordinary rules.
- Activation: scan the frontier, bind known triggers and append activated rule instances.
- Matching/instantiation: evaluate ordinary and activated clauses against Known,
  including substitution-map cloning, structural matching, witness construction,
  consequent substitution and candidate creation. These sub-operations are not
  individually timed, so the profile does not attribute all this time to matching
  comparisons alone.
- Provenance deduplication: scan previously accepted unstamped justifications.
- Retention: append new justifications, stamp them and store new logical events.
- Remainder: unmeasured loop overhead, candidate destruction, allocations outside
  the timed regions, counter/report overhead and other bookkeeping.

Rounds, clause evaluations, activated rule instances, generated candidates,
duplicate candidates, exact provenance equality comparisons and event matching
probes are counted. Header rejections mean a mismatch in verb, polarity or arity,
before structural argument/time matching. The matcher currently clones the
substitution map before this rejection; the profile counts attempts, not the
number or cost of individual clones.

One activated rule instance is not necessarily one Program activation: a single
known trigger can activate several RuleUses. Reported `activated` counts these
rule instances, not distinct Program declarations or trigger events.

Each completed round prints cumulative statistics. A timeout does not return a
partial closure as a successful result; the last printed round is only a lower
bound on work completed before the process was killed.

The test `profiling_preserves_full_closure_and_stamps` runs the source-defined
`unitProduct` with probes disabled and enabled and compares the complete Debug
representation of the two closures. This includes types, transition metadata,
event/witness ordering, substitutions, justification multiplicity and exact
clock stamps, beyond logical Event equality. The comparison passed.

These are debug-build observations on this machine, not complexity laws or
performance guarantees. Timers and counters have overhead. Parsing and loading
the fixture occur before the measured closure; process times include that work.

## Completed degree-one product

Source scenario `partialProduct`: `(1+2x)(2+x) = 2+5x+2x²`.
Final run: 223 logical events, 244 distinct justifications, 36 rounds.

| Phase | Measured time | Share of measured closure wall time |
|---|---:|---:|
| Associated Program preparation | 1.149 ms | 0.02% |
| Program activation | 1.718 ms | 0.03% |
| Matching/instantiation | 5,005.368 ms | 98.01% |
| Provenance deduplication | 66.731 ms | 1.31% |
| Retention/stamping | 3.707 ms | 0.07% |
| Remainder | 28.568 ms | 0.56% |
| Total closure wall time | 5,107.241 ms | 100% |

The whole bounded test took 5.8404 s, peaking at 15.54 MiB RSS.

| Work counter | Count |
|---|---:|
| Total clause evaluations | 6,375 |
| Associated clause evaluations | 5,943 |
| Activated rule instances at completion | 226 |
| Event matching probes | 2,117,296 |
| Header rejections | 2,023,990 (95.59%) |
| Candidate justifications | 6,246 |
| Candidates already seen | 6,002 (96.09%) |
| Exact provenance equality comparisons | 607,805 |

A preceding run yielded exactly the same work counters, with 5,255.398 ms
closure wall time. Counts are reproducible here; timings vary.

The leading matching/instantiation costs in that preceding run were the two
members of `RightWalk` (742 evaluations each, about 832 and 828 ms), followed by
`LeftWalk::changePhase` (363 evaluations, about 544 ms). Complete per-rule
records are in the diagnostics listed below.

Inspection explains the repetition: `close_with_programs` evaluates every
ordinary rule and every previously activated rule instance on each nonempty
frontier round. `match_antecedents_with` scans all Known for each event leaf and
compatible preceding conjunct. Old candidates are recreated and only then
rejected by the linear provenance scan. No scheduling optimization was made.

## Larger product: incomplete diagnostic

The existing `principalProduct` was attempted once with profiling enabled,
using the already compiled binary. The unchanged runner stopped it at
15.0225 s, exit status -9, peak RSS 22.99 MiB. Cgroup memory events reported no
OOM or OOM kill. No further tests were executed after that timeout.

The last fully recorded round was 18; it was not the fixpoint:

| Counter/phase at round 18 | Value |
|---|---:|
| Known events | 530 |
| Distinct justifications | 639 |
| Next frontier | 11 |
| Activated rule instances | 536 |
| Clause evaluations | 6,605 |
| Matching probes | 5,530,664 |
| Header rejections | 5,311,802 |
| Candidate justifications | 7,750 |
| Duplicate candidates | 7,111 |
| Provenance equality comparisons | 1,942,634 |
| Activation time | 4.329 ms |
| Matching/instantiation time | 13,831.323 ms |
| Provenance deduplication time | 162.500 ms |
| Retention time | 11.522 ms |

Work in the interrupted following round is absent from this table. There is no
completed large result and no claim of termination or general correctness.
The measured bottleneck in this run is matching/instantiation, not activation
or memory exhaustion. Provenance scans are redundant work but a small fraction
of the measured time; optimizing them alone does not address the dominant cost.

## What the existing derivative does and does not cover

`IncrementalClosure` caches a closure under a fixed `Vec<Rule>`. It anchors each
conjunct in the newly added frontier, matching remaining conjuncts against Known.
It does not call `close_program`, discover new `Verb.program` activations or
maintain their bound rule-instance environment. Therefore the polynomial product
cannot currently be handed to this API as an equivalent incremental computation.
Flattening its Programs into ordinary rules without activation guards would
change semantics and provenance; that was not attempted.

Using the existing `forward_incremental.res` fixture, the experiment compares
an update with full recomputation on exactly the same inputs and fixed rules:

| Work | Incremental update | Full recomputation |
|---|---:|---:|
| Rounds | 3 | 2 |
| Clause evaluations | 6 | 4 |
| Event matching probes | 11 | 19 |
| Candidate justifications | 2 | 4 |
| Duplicate candidates | 0 | 2 |
| Provenance equality comparisons | 1 | 4 |

Logical events and full DerivationRecords, including witnesses, substitutions
and multiplicity, agree; `derivedAt` values are excluded according to the
incremental contract. A repeated identical delta performs zero matching and
zero clause evaluations, producing neither facts nor justifications.

This demonstrates reduced matching and candidate repetition in this fixture,
not fewer rounds/evaluations in general and not a reliable timing speedup for
this sub-millisecond workload. It does not establish that the existing derivative
reduces the cost of recursive associated Programs.

## Next performance questions, without implementation changes

An index on verb/polarity/arity is a concrete candidate for avoiding the measured
irrelevant Known scans. It would need to preserve existing witness order,
structural equality and all justifications. Filtering before cloning compatible
substitutions is another candidate. Neither optimization was implemented.

Generalizing the derivative to this product would require a delta-aware
activation layer: newly known triggers create new bound RuleUses; existing bound
rules react to relevant new evidence; activation witnesses and alternative
justifications remain intact. This is a scheduling extension to investigate,
not a new arithmetic primitive or a conclusion established by the present profile.

## Individual verification and reproduction

All compilation, formatting and test execution used `scripts/test_safe.py`.
Limits stayed at 1 GiB total, no swap, one Cargo job, one harness thread,
15 seconds per test and the existing preventive memory cutoff. No full suite
was run. Five distinct selected tests passed, covering the two profiles,
instrumentation equivalence, activation guards/provenance and preservation of
new justifications for old events with Unit axioms. The opt-in large diagnostic
timed out; it is not counted as passing.

The initial library test compilation peaked at 694.98 MiB RSS, below the runner's
768 MiB preventive cutoff; the subsequent incremental compilation peaked at
502.09 MiB. All selected passing tests peaked below 16 MiB RSS.

Normal individual profile (the default is always the completed small fixture):

```sh
python3 scripts/test_safe.py --test derive::profile_tests::profile_polynomial_product_fixture
```

Only when deliberately investigating the large diagnostic, still with the same
limits and one selected test:

```sh
RESIDUUM_PROFILE_PRODUCT=principalProduct python3 scripts/test_safe.py \
  --binary target/debug/deps/residuum-cfe44311058885c9 \
  --test derive::profile_tests::profile_polynomial_product_fixture
```

That binary path identifies this run's artifact and may change after rebuilding.
An opt-in timeout remains a failed run; no runner controls are weakened.

Diagnostics under the experimental worktree:

- `target/test-diagnostics/20261009T021636840601Z-3760979`: initial compilation
  and exact closure/stamp equivalence.
- `target/test-diagnostics/20261009T021711655504Z-3762039`: first small-product
  profile and fixed-rule incremental comparison.
- `target/test-diagnostics/20261009T021816045963Z-3763178`: final small-product
  profile, with the measurements in the completed-product table.
- `target/test-diagnostics/20261009T021835240594Z-3763795`: four individual
  regressions, including repeated instrumentation and incremental comparisons.
- `target/test-diagnostics/20261009T021858117706Z-3764038`: single large-product
  diagnostic, cumulative round logs and verified timeout/memory report.

`report.md`, `*.output.txt` and `*.result.json` retain process timings, raw counters
and resource outcomes. Diagnostic artifacts are local and ignored by Git; this
document preserves the measurements needed to assess the conclusion.
