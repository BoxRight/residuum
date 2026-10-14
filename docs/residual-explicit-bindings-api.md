# Explicit residual query bindings

This additive API starts from `9f78f88`. Existing query APIs retain their
contracts, including the local API's empty result for a satisfied rule. The new
local API retains Unit explicitly, so it distinguishes success without pending
requirements from a head that does not match.

`QueryBinding` contains a `QueryParameterScope::Rule(RuleRef)` or
`QueryParameterScope::Program(ProgramRef)`, a parameter name, a type and a
`SubstitutionValue`. Program identity refers to the resolved, possibly partially
applied Program; only its remaining shared parameters may be bound. Rule scope
refers to its original lambda, including each occurrence in compiled RuleUses.
Mapped parameters constrain the existing RuleUse expression; fixed mappings
cannot be overwritten. Ordinary rules never share bindings by spelling alone.

## Interfaces

* `query_residual_with_bindings(module, rule, goal, known, bindings)` reduces
  one Rule without a Program wrapper. Its requirements preserve typed pending
  parameters, supplied substitutions, evidence witnesses and Unit branches.
* `query_program_residual_with_bindings(module, scope, known, goal, bindings,
  limits, allowed)` searches exactly `HornQueryScope::Rules` or the compiled uses
  of `HornQueryScope::Program`. `allowed=None` retains the existing seeded policy;
  `Some(&[])` forbids new hypotheses.
* `query_active_program_residual_with_bindings(module, known, goal, bindings,
  limits, allowed, clock)` uses the existing guarded projection, then certifies
  hypotheses with the original module's `close_program`.

Each returns `QueryWithBindings<T>` containing the original-style query result
and validated supplied constraints. Temporal values are canonicalized using the
parameter type. Every rule proof's substitution contains supplied constraints
alongside head bindings; compiled proofs preserve their hygienic parameter names.
Activation metadata resolves to original producers and RuleUse origins.

Bindings are constraints, not evidence or mutations of the theory. Unknown
parameters, scopes outside the selected rules, incorrect types, duplicate or
inconsistent bindings, conflicts with fixed specialization and head conflicts
are errors. Head conflicts are checked even when Known already contains the goal.
Recursive search with unresolved parameters returns an error listing all pending
typed parameters of the unsupported producer. Local queries can represent those
parameters; no domain enumeration or implicit rule filtering is introduced.

Constants are checked against module declarations. A bare entity `Term::Var`
is an opaque runtime identity whose type is asserted by the caller; its spelling
does not reclassify it as a constant. Records and constructors are checked
structurally. Temporal bindings must be resolved; omitting a binding retains the
pending temporal parameter. Existing goal/evidence caller preconditions remain.

## Minimal CLI integration proposal

Keep the existing `query` command and Formal syntax. A future adapter can add
repeatable `--bind rule:Rule.parameter=value` and
`--bind program:Program.parameter=value`, with `--rules r1,r2` for explicit Horn
selection and `--active` for executable associations. Values should use the
existing term parser and module type declarations, preserving explicit constant
and runtime-identity distinctions. Program selection should be explicit as well.
Without those options, dispatch and output must remain unchanged.

The binding API provides a Rust interface without binding command or source
syntax. The accompanying stepwise CLI flags do not parse bindings. This API does
not alter defaults, State, forward semantics or ontology files.

## Verification

Individual tests in `src/residual/bindings/tests.rs` reuse
`legal_geometry/explicit_substitutions.res` and compare all eight evidence cases
against the previously verified ComplianceBound experiment. Both original
compliance producers and their multiplicity are retained; every hypothesis set
is sufficient under the original module. Other cases cover local partial
bindings, typed errors, Program scope, staged beta reduction, record types,
temporal constructors, homonymous runtime identities, active hidden guards,
Unit, pending requirements, absence and resource-limit errors.

Run only selected individual tests through `python3 scripts/test_safe.py`.

Validation on the standalone binding branch completed: 9 new library tests, 8 selected existing library
regressions and 4 selected CLI regressions (21 individual tests). Reports:

* `target/test-diagnostics/20261014T011838459644Z-918224`
* `target/test-diagnostics/20261014T012102064578Z-920229`
* `target/test-diagnostics/20261014T012425111376Z-922875`

All ran sequentially with 1 GiB cgroups, zero swap, one Cargo job and one harness
thread. Test debug information was disabled to reduce compilation memory; the
limits and stop controls were retained. The full suite was not run.
