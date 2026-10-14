# Stepwise evaluation

This additive change starts at `9f78f88`. Existing full evaluators keep their
behavior; new sessions expose their intermediate states. Step budgets remain
CLI execution options. The integrated explicit-bindings change separately adds
[scoped Formal queries](residual-query-source-bindings.md). CNL, State execution
and default selection policy are unchanged by the stepwise feature.

## Forward and the discrete derivative

`ForwardSession::new(seeds, rules)` evaluates an ordinary rule set;
`ForwardSession::for_module(seeds, module)` includes executable Verb.program
associations. `step(clock)` reads a frozen X and returns:

* `consequences`: T(X), including already known conclusions;
* `witnesses`: distinct immediate rule instances, retaining alternative proofs;
* `added`: T(X) outside X;
* `known`: X union T(X);
* new stamped `derivations`, `round`, and `complete`.

This is synchronous: a conclusion from one rule cannot enable another rule in
the same round. A positive activation trigger must already be in X. A newly
produced trigger activates associated bodies in the next round. An associated
unit body still requires its guard. Logical Event identity deduplicates events;
rule, substitution and ordered witnesses identify justifications.

For finite Horn antecedents, the seed-inclusive immediate-consequence operator
F(X) = S union T(X) is Scott-continuous and its Kleene approximations have union
equal to the least closure. The session starts at S and uses the equivalent
inflationary iteration X := X union T(X). Program associations contribute
guarded immediate consequences using the same matching and compilation machinery.

`IncrementalClosure::begin_stepwise_update(additions)` constructs a derivative
session from its privately maintained closed base B. Its first step returns
the additions outside B; subsequent steps approximate

```
Phi(D) = (additions outside B) union (T(B union D) outside B).
```

`DerivativeStep.delta` is cumulative and disjoint from B. Old base derivations
are retained without restamping. These reference steps use full snapshot
matching; they do not replace the existing semi-naive incremental implementation.

Rounds do not supply PropositionTime or DerivationTime. The caller's external
clock stamps only previously unseen justifications. Completion means the event
fixed point has been verified, after retaining all justifications enabled by
that snapshot. It may include a new proof for an already known Event.

## Other evaluators

`ResidualSession::new(module, evidence, goal, limits, allowed)` expands all
currently open event obligations by one rule layer. A `ResidualSearchTree`
retains rule identities, substitutions, conjunctions and alternative branches.
Evidence and admissible hypotheses become leaves. `ResidualStep.pending` is an
explicit frontier, never a returned hypothesis set. Only a completed session
offers `result()`, certified by the existing forward witness replay. Unit,
pending hypotheses and no explanation remain distinct.

`ActiveResidualSession` uses the existing guarded projection and certifies its
completed branches with the original `close_program`. Projected proof producers
resolve to actual rules/RuleUses in activation metadata; hypothetical closure
records come from the original evaluator. No query changes Known or the module.
Hypothetical certification still requires terminating forward inputs.

The old cycle and typed goal/evidence preconditions remain. Parameters must be
determined by the goal or supplied scoped bindings. Unresolved parameters are
errors. SearchLimits bound cumulative search
work, including intermediate products. Layered search has different allocation
metrics/scheduling from depth-first search, so budget exhaustion need not occur
at the same point. Errors are terminal and never produce truncated success.

`FourSession` retains the original finite ground universe (at most four atoms).
Each step filters candidate interpretations by one premise or rule, returning
eliminated and surviving interpretation IDs. Two bits per sorted atom encode
positive and negative support. Entailment remains None until every constraint
has been checked; final model counts and vacuity follow the existing evaluator.
Intermediate model sets depend on constraint order, although the final set does
not. This is semantic model refinement, not the forward Horn T operator.

`DefaultSession` preserves the existing workflow: mandatory forward rounds,
one selection step against the complete mandatory context, and selected forward
rounds from the original seeds. It never reevaluates selection against selected
consequences. This is phase progression, not a monotone operator over defaults.

All sessions expose `step` and `trace(max_steps, ...)`. A bounded trace can end
before completion; an infinite temporal closure can have arbitrarily many finite
steps without a complete result. The step count does not replace RAM or timeout
controls. Sessions retain state in memory for subsequent API calls.

## CLI

Existing invocations retain legacy dispatch and output. New flags are:

```sh
./residuum run examples/experiments/residual_proof.res --steps 1
./residuum explain examples/experiments/residual_proof.res --steps 64 --trace
./residuum query examples/experiments/residual_proof.res --steps 64 --trace
./residuum query examples/experiments/four_counterexample.res --trace
```

`--steps N` advances at most N transitions and displays the latest snapshot.
`--trace` displays every transition and defaults to 64 steps. N is 1..4096.
The CLI starts a fresh session per invocation. Status explicitly distinguishes
complete evaluation from a step limit. Residual requirements and FOUR entailment
are displayed only after completion. `query` retains the legacy rejection of
unscoped queries in Program-bearing modules; explicit scoped residual and active
residual source queries can opt into those modules. Defaults remain unsupported
for residual queries. Active traces include original producers and guard metadata.
Forward step sessions can observe bounded prefixes of cycles without computing
an unbounded full closure. No flags execute State transformations.

The launcher still compiles/runs through `scripts/test_safe.py`; the same flags
are accepted there with `--cli` and `--fixture`. All existing cgroup, memory,
swap, job, thread, timeout and stop controls remain in force.

## Scoped bindings in sessions

Explicit typed bindings were developed independently on
`feat/residual-explicit-bindings`, also based at `9f78f88`. The integration includes
that API and adds `begin_residual_query_with_bindings`,
`begin_active_residual_query_with_bindings` and its explicitly scoped variant
`begin_active_residual_query_in_scope_with_bindings`. They return `QueryWithBindings` of
the corresponding resumable session, preserving validated supplied constraints
alongside the evolving search. Both full and stepwise queries use the same scoped
resolver and typed constraint merger. Pending obligations never bypass ordinary
forward replay or original-module active certification. No global-name
substitution interface or existential enumeration is introduced. Bindings are
provided through the agreed `.res` `using` / `with` syntax, not extra binding
flags. Both Rust callers and source queries can combine bindings with stepping.

The authorized audit commit `1f7b274` extracts common residual operations into
`residual::engine` and derivation retention into `derive::retain_derivation`.
DFS and layered search retain different continuation schedules and budget
accounting. The synchronous forward loop remains a separate scheduler for
frozen-snapshot T(X), sharing existing inference operations and proof retention.
See the [integration audit](worktree-integration-20261014.md) for exact call paths.

## Verification

Individual tests compare completed event sets and justification records with
the existing evaluators, ignoring only external clock stamp differences. They
cover rule order, unit rules, conjunction and signed evidence, old-event proofs,
activation timing, finite cycles, incomplete infinite prefixes, the additive
derivative, residual alternatives and provenance, guard certification, FOUR
model refinement, and one-time default selection. CLI tests cover both new
options and unchanged legacy dispatch. The full suite is not run.

Initial successful selected verification reports (before the source integration):

* 14 evaluator tests: `target/test-diagnostics/20261014T013402154660Z-931558`
* 8 CLI tests: `target/test-diagnostics/20261014T013615213566Z-933704`
* 1 additional derivative test and 8 library regressions:
  `target/test-diagnostics/20261014T013859212409Z-935996`
* 3 combined-session tests, 9 binding tests and 3 regressions:
  `target/test-diagnostics/20261014T014337018788Z-939963`
* End-to-end FOUR trace: `target/test-diagnostics/20261014T014435251454Z-940991`
* End-to-end single forward round:
  `target/test-diagnostics/20261014T014536251396Z-942232`
* 3 final CLI regressions after integration:
  `target/test-diagnostics/20261014T014639659934Z-942907`

These reports contain 49 selected individual test executions, plus the two
end-to-end CLI fixture runs. The standalone binding branch separately records
21 selected tests. Some compatibility tests are deliberately repeated after
integration; these counts do not describe a full-suite run.

The final combined integration reruns 24 library and 15 CLI tests individually,
including empty-binding compatibility, independent specialization, source traces,
activation guards, incremental provenance and the literal ontology partition.
Reports are recorded in the [integration report](worktree-integration-20261014.md).

Tests ran sequentially in verified 1 GiB cgroups, zero swap, one Cargo job and one
harness thread, with all timeout and stop controls retained. Disabling test debug
information reduced compilation memory without increasing those limits. No
ignored CNL tests were used as completion criteria.
