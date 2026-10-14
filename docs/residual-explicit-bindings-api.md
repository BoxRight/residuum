# Explicit residual query bindings

This additive API starts from `9f78f88`. Existing query APIs retain their
contracts, including the local API's empty result for a satisfied rule. An empty
bindings list delegates to the corresponding legacy API, preserving its errors,
validation order, results and complete provenance.

## Local-query Unit contract

With nonempty bindings, the local API retains each satisfied branch as an
explicit `Unit`, including ordered evidence witnesses, canonical substitutions
and unresolved typed parameters. This is a syntactic reduction of one Rule,
not a forward-certified assertion. A head that does not match returns an empty
vector. A supplied binding conflicting with a matching goal is an error; an
independently specialized Rule whose head is incompatible returns no match.
An empty bindings list retains the legacy local convention: if any complete
witness satisfies the Rule, the entire local result is empty. This distinction
is intentional and is confined to local
queries; Horn and active queries already represent successful Unit explanations.

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

## Record order and root subtyping boundaries

Typed `Term::RecordLiteral` bindings must contain each declared field exactly
once, with a compatible field value. Validation looks fields up by name, accepts
any supplied field order and **does not reorder it**. Returned bindings,
substitutions and instantiated requirements preserve that order. Existing typed
equality and matching compare record fields positionally, including field names;
two permutations of the same fields are different typed values. Source
elaboration instead emits declaration order, including for named RuleUses.

For `Pair { left : Thing right : Thing }`, a caller-supplied typed value with
fields `[right = b, left = a]` behaves as follows:

| Query boundary | Result |
|---|---|
| Same permuted value in a variable-headed Rule goal and binding | Accepted, with the supplied order preserved. |
| Canonically ordered goal `[left = a, right = b]` and permuted explicit binding | `Err`: binding is inconsistent with the goal. |
| Matching permuted goal, but only canonical source evidence in Known | Evidence does not match. Local reduction remains pending; Horn and active queries return no explanation when `allowed=Some(&[])`. |
| Matching permuted goal and permuted evidence | Evidence matches; the tested local, Horn and active queries return Unit. |
| Source-specialized RuleUse fixes the canonical record, but an explicit Rule binding supplies the permutation | `Err`: binding conflicts with the specialized RuleUse. |
| Canonical specialized head versus permuted goal, with no explicit binding | No matching explanation. |

These are existing representation/matching policies, not a claim that typed
field permutations are interchangeable. Callers constructing records for use
with source-elaborated goals or evidence should use declaration order.

Root entity bindings use the existing named RuleUse policy:
`elab::rule_type_assignable`. A constant's declared type must first be assignable
to `QueryBinding.ty`; that supplied type must then be assignable to the queried
parameter's declared type. Subtype ancestry is transitive. Values and supplied
types are preserved, without coercing the constant or rewriting its identity.

For `Special <: Thing`, `Deep <: Special`, `child : Special`, `deep : Deep` and
`base : Thing`:

| Binding or application | Result |
|---|---|
| Thing parameter; `ty=Special`, value `child` | Accepted. |
| Thing parameter; `ty=Deep`, value `deep` | Accepted transitively. |
| Thing parameter; `ty=Thing`, value `child` | Accepted; the constant is compatible with the supplied supertype. |
| Thing parameter; `ty=Special`, value `base` | Rejected: the value is incompatible with the supplied type. |
| Special parameter; `ty=Thing`, value `child` | Rejected: the supplied type is too broad, even though the constant itself is Special. Supplying `ty=Special` succeeds. |
| Thing parameter; unrelated entity type/value | Rejected. |
| Named source RuleUse binds `child` to a Thing parameter | Accepted by the same named-binding assignability policy. |
| Positional `specialize_program(P, [child], constants)` or source `P(child)`, where P's first parameter is Thing | Rejected: positional application requires the constant's declared type to equal the parameter type. `P(base)` succeeds. |

The named root policy applies to local Rule, scoped Horn Rule, shared Program
and active query bindings. Record **fields** retain the source checker's stricter
`elab::is_assignable` policy; root subtype acceptance does not widen that policy.
Opaque runtime Vars continue to rely on the caller's supplied type.

No policy change is included or required for these regressions. Permutation
normalization or widening positional Program application would require a
separate policy proposal and review before implementation.

## Source and CLI interface

The agreed `.res` extension is now implemented: `using rules { ... }`,
`using program P`, scoped `with { rule r.parameter = value }` /
`with { program P.parameter = value }`, and `query active residual ...`.
Types are checked through the existing source term elaborator and binding APIs.
Explicit bindings require an explicit search scope. Existing queries without
these clauses retain their legacy path and behavior.

See [the source syntax and reuse review](residual-query-source-bindings.md) for
complete examples, supported fragment, errors, call paths and verification.
The CLI command remains `./residuum query case.res`. Stepwise flags remain on
the separately preserved branch. Defaults, State, forward semantics and ontology
files are unchanged.

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

Validation completed: 9 new library tests, 8 selected existing library
regressions and 4 selected CLI regressions (21 individual tests). Reports:

* `target/test-diagnostics/20261014T011838459644Z-918224`
* `target/test-diagnostics/20261014T012102064578Z-920229`
* `target/test-diagnostics/20261014T012425111376Z-922875`

All ran sequentially with 1 GiB cgroups, zero swap, one Cargo job and one harness
thread. Test debug information was disabled to reduce compilation memory; the
limits and stop controls were retained. The full suite was not run.

The compatibility continuation adds legacy delegation for empty bindings,
independent specialization oracles with full metadata/provenance comparisons,
and a shared compiler RuleUse mapping. Final checks pass 35 selected tests
individually. See [the reuse and compatibility review](residual-explicit-bindings-reuse-review.md)
for call paths, reproduced failures, typed-validation policies, reports and the
separately preserved stepwise audit status.

The two boundary regressions subsequently passed individually, without changing
production code or policy:

* `record_order_boundary_preserves_typed_order_and_positional_matching`:
  `target/test-diagnostics/20261014T040921511039Z-1055328`.
* `root_subtyping_boundary_uses_named_assignability_and_exact_program_application`:
  `target/test-diagnostics/20261014T040618694605Z-1053144`.
