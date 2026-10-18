# Stepwise shared operations audit

The previously paused audit edits are now authorized for commit and integration.
This commit is based on `e62232a`; the main integration keeps the newer source
query syntax and explicit-binding compatibility fixes from the bindings branch.

* `residual::engine` extracts validation, seeded-hypothesis policy, budget checks,
  evidence/hypothesis classification, typed Rule expansion and proof construction
  from the original residual code. DFS and layered sessions call the same
  operations while retaining distinct continuation schedules.
* `programs::rule_use_substitution` shares the compiler's hygienic substitution
  map with original Rule-parameter scope resolution.
* `derive::activation_bindings` now owns the existing positive-trigger,
  verb-name and arity check previously repeated by its callers.
* `derive::retain_derivation` shares duplicate-proof retention, logical event
  insertion and stamping across full, incremental and synchronous evaluation.
  Scheduling and index updates remain in their original drivers.

No new unifier, substitution algorithm or forward certificate is introduced.
The synchronous forward loop remains necessary to observe frozen-snapshot T(X)
rounds; it does not replace the existing full or incremental scheduling paths.

Eleven named tests passed individually through `python3 scripts/test_safe.py`:
`target/test-diagnostics/20261014T165416011272Z-1587190`. These cover synchronous
T/activation timing, the derivative, incremental/profile provenance, residual
alternatives, active certification, supplied bindings and legacy validation.
Limits remain 1 GiB, zero swap, one Cargo job and one harness thread. The full
suite was not run. Main integration separately reruns compatibility and source
composition checks after resolving the merge.
