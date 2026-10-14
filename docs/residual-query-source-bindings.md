# Scoped residual bindings in Formal queries

The standalone `feat/residual-explicit-bindings` branch now accepts explicit
search scope, scoped bindings and active residual mode in `.res` experiments.
Existing query syntax retains its previous behavior. Bindings are query
constraints; they are never seeds, hypotheses added to Known, or mutations of
the module.

## Ordinary Rules

The existing compliance example can be queried without specializing a Program:

```text
experiment boundCompliance {
    seeds {}
    query residual compliance(gNorm, gAct, gSubject, gObject,
        gBeneficiary, administrative, gDate) @ gTau
    using rules {
        obligationCompliance,
        rightCompliance,
        filedObligation
    }
    with {
        rule obligationCompliance.conduct = gConduct,
        rule obligationCompliance.actAuthority = private,
        rule obligationCompliance.actEffective = gDate,
        rule rightCompliance.conduct = gConduct,
        rule rightCompliance.actAuthority = private,
        rule rightCompliance.actEffective = gDate
    }
}
```

`using rules { rightCompliance }` also supports a single Rule in a module with
no Program declarations. Scope is exact; dependencies are not silently added.
Names are resolved in the loaded module, including imports, and original
producer origins are retained.

Internal commas are mandatory; the final comma in a list is optional. `with`
follows `using`. Omitting `with`, or writing `with {}` with an explicit scope,
uses the existing empty-binding API contract for the selected view.

## Program scope

The corresponding Program form is:

```text
query residual compliance(gNorm, gAct, gSubject, gObject,
    gBeneficiary, administrative, gDate) @ gTau
using program ComplianceContext
with {
    program ComplianceContext.conduct = gConduct,
    program ComplianceContext.actAuthority = private,
    program ComplianceContext.actEffective = gDate
}
```

This searches precisely the Program's compiled RuleUses. Program identity means
the resolved identity, including a partially applied alias. Only remaining
shared parameters can be bound in Program scope. Original Rule parameters can
also be bound by their Rule name; the shared compiler RuleUse mapping resolves
them to the appropriate compiled expressions and detects fixed-value conflicts.

## Active mode

```text
experiment pending {
    seeds {}
    query active residual done(item) @ after tau
    using program P
    with { program P.authority = private }
}
```

In [the active example](../examples/experiments/active_query_bindings.res), this
requires `starts(item, private) @ tau` and preserves two explanations from two
original RuleUses. Supplying that trigger produces Unit. An existing public
trigger does not satisfy the private query constraint; it remains in observed
Known and contributes its own original justifications during hypothetical
certification.

Active scope selects the existing **guarded** search producers, never unguarded
compiled Horn clauses. Rule scope includes eligible guarded occurrences of each
named original Rule, including multiple Program uses and associations. Program
scope selects eligible occurrences of that resolved Program. Ordinary rules
outside the explicit search scope are not used to prove candidate guards.
Certification still evaluates the entire unchanged original module through
`close_program`, so it retains sufficient derivations outside the search scope.

`query active residual ...` without clauses selects the existing full-module
active API. Explicit bindings, including an explicit empty `with {}` clause,
require a `using` scope in source syntax. A declared Program without an eligible
active association does not acquire activation semantics merely by being named.

## Types, errors and compatibility

Binding values use the existing Formal term grammar: module constants,
`after(tau)`, nested temporal constructors, records and `none`. Type checking uses
declared parameter types and existing named-binding assignability. The term
elaborator infers each concrete value's type; `none` receives its parameter's
expected optional type. Temporal substitution values use existing canonical
conversion. RHS identifiers resolve to constants in the module; a homonymous
query parameter does not capture them. Source queries do not introduce runtime
variable or existential-enumeration syntax.

Source record literals normalize to declaration order through existing
`lower_term`. The typed Rust API continues to preserve supplied record order.
Named root bindings allow entity subtypes; positional Program application keeps
its existing exact-type requirement. These are the
[documented API boundaries](residual-explicit-bindings-api.md#record-order-and-root-subtyping-boundaries),
not new matching or assignability policies.

Unknown or duplicate scopes/parameters, unknown constants, bad value types,
bindings outside the selected scope, duplicate bindings and explicit goal/fixed
specialization conflicts are errors. Partial bindings elaborate successfully;
recursive search reports remaining typed parameters instead of enumerating
values or discarding unsupported selected rules. The local Rust API can still
return those pending parameters explicitly.

Legacy residual and FOUR queries use their original runner paths. In particular,
the old rejection of unscoped residual/FOUR queries in Program-bearing modules
is retained, even in a source file also containing configured queries. To query
such a module, every local experiment used by `query` must explicitly opt into
scoped residual or active residual mode. Imported experiments are still stripped
by the existing module loader.

FOUR queries do not accept these residual clauses or active mode. Defaults remain
unsupported for residual search. The source experiment runner keeps its existing
conservative acyclic termination checks: scoped Horn observation checks its
selected rules, while active certification checks original module rules and
activation edges. This does not change the underlying Rust query APIs' supported
fragment. `run` and `explain` still evaluate the original module and ignore query
constraints during execution.

Public AST data is extended with `FormalExperiment.query_options`,
`Experiment.query_options` and `ExperimentResult.active_residual`. Rust code
constructing these structs directly must provide the new fields, using defaults
or `None` for legacy behavior. Existing callable bindings APIs and public import
paths remain available; the scope/binding data types now live in `typed.rs` and
are re-exported from their previous paths.

## CLI and stepping

Run the supplied examples from the repository root:

```sh
./residuum query examples/experiments/legal_geometry/explicit_query_bindings.res
./residuum query examples/experiments/active_query_bindings.res
```

The launcher continues to use `test_safe.py` and enforced process limits.
No step budget is embedded in `.res`. The stepwise branch's `--steps` / `--trace`
interface remains separately preserved and paused; those flags have not been
transplanted into this standalone branch. Future integration can pass the same
typed `QueryOptions` to its sessions without new source stepping syntax.

## Reuse audit

| New component | Existing component reused | Actual call path | New logic and why it is necessary |
|---|---|---|---|
| Source query clauses (`formal.rs::formal_module_parser`, test-only `ManualParser::parse_experiment`) | Existing event, term, identifier and comma-list parsers | `parse_formal` → existing lexer/parser → `FormalExperiment.query_options` | Parse `active`, `using` and `with`; no inference. Manual parser parity remains a test oracle, not a production semantic path. |
| Typed query configuration (`typed.rs::QueryOptions`, `surface.rs::SurfaceQueryOptions`) | Existing `QueryBinding`, `QueryParameterScope`, `HornQueryScope`, Rule/Program references | Surface options → `elab::lower_query_options` → existing shared interface data | Carry query intent through the active Formal → SurfaceAST → TypedAST pipeline. Scope/binding types were moved and re-exported, not duplicated. |
| Source value elaboration (`elab.rs:410`, `lower_query_options`) | `elab::lower_term`, `residual::query_parameter_type`, existing binding validator/canonicalizer | `lower_term(value, empty lambda scope)` → inferred type/contextual None → `validate_query_options` → existing `resolve` / `preflight` / `constrained_goal` | Source name resolution and parameter context. No copied substitution, temporal conversion, matcher or type hierarchy. |
| Shared Horn scope selection (`bindings.rs:108`, `scoped_rules`) | The previous selection logic in `query_program_residual_with_bindings`, `compile_program` | Old bound API, source validation and source orchestration all call `scoped_rules` | Extract exact selection to avoid an independent compiler/Rule lookup path. |
| Shared parameter lookup (`bindings.rs:312`, `binding_parameter`) | The resolver's original Rule/Program parameter lookup | Old `resolve` and source `query_parameter_type` call the same lookup | Infer/check source values against the right scope without duplicating declaration lookup. Standalone Rules outside the module remain supported by the old API. |
| Scoped active API (`bindings.rs:154`, `query_active_program_residual_in_scope_with_bindings`) | `active::guarded_view`, original bound active path, producer origins, same Horn search and `active::certify` | `active_query` → `active_view` → existing guarded projection → producer filter → `resolve` / `preflight` → `program::query_with_constraints` → `certify` → original `close_program` | Filter eligible guarded search producers by explicit scope. No second guard construction, search, closure or certification loop. Unscoped empty bindings still delegate to the original API. |
| Source execution (`experiments.rs:157`, `run_configured_query`) | Existing bound Horn/active APIs, `close`, `close_program`, runner's acyclic checks | `cli::execute_file(Query)` → loader/elaboration → `run_experiments` → configured dispatcher → corresponding binding API | Dispatch and seed-only observation. Hypothetical certification remains inside the active API; no hypotheses are inserted into observed Known. |
| Shared termination check (`experiments.rs:259`, `require_program_acyclic`) | Existing forward runner's dependency/activation-edge check | Forward runner and active source dispatcher → extracted check → existing `require_acyclic_with_edges` | Reuse the source runner's termination restriction before hypothetical original closure. No forward driver or scheduling change. |
| Output (`experiments.rs:385`, `render_results`) | Existing event/term/requirement rendering and certified result objects | Same CLI adapter → existing runner → render scope/constraints and active guards | Show which query was evaluated and distinguish active certification from Horn observation; legacy output is unchanged. |

No runtime substitution, matching, DFS, closure, proof or activation machinery
was copied or independently implemented. New logic is parsing, interface data,
source elaboration context, scope filtering, dispatch and presentation. The
existing typed structural boundary validator and duplicated seeded-abducible
enumeration remain as recorded in the previous reuse review.

For the compliance fixture, the concrete path is: parse `rule
rightCompliance.actEffective = gDate` → `lower_term` resolves `gDate` as a
PropositionTime constant → parameter lookup finds the original Rule's
`actEffective` → existing `resolve` canonicalizes it to `Time(At("gDate"))` →
runtime `query_program_residual_with_bindings` selects original Rules → original
head matcher plus `bind_typed` merge constraints → existing substitution/DFS
expand the right and Act requirements → original witness replay certifies the
proof. Original producers and `LegalGeneral` origins remain intact.

## Verification and branch status

**38 distinct named tests passed individually**, plus two actual CLI fixtures,
through `python3 scripts/test_safe.py`. No full suite or ignored CNL tests ran.
All execution was sequential with 1 GiB cgroups, zero swap, one Cargo job and
one harness thread; `CARGO_PROFILE_TEST_DEBUG=0` was retained.

* Nine new parser/source tests: `20261014T053043495991Z-1115473`.
* Two of those tests expanded to cover a module with no Program wrapper and a
  successful partially applied Program, then rerun:
  `20261014T054607217881Z-1127778`.
* Three new CLI and two legacy CLI tests: `20261014T053356161032Z-1118084`.
* Sixteen complete binding/provenance/boundary regressions and eight legacy
  experiment/forward tests: `20261014T053827586220Z-1121858`.
* Actual compliance CLI fixture: `20261014T054054883611Z-1123847`.
* Actual active CLI fixture: `20261014T054327870004Z-1125918`.
* Format check: `20261014T055025601621Z-1131156`.

Comparisons include full event kind/transition metadata, proposition times,
substitutions, ordered proof trees and antecedents, original producers/origins,
and multiplicity. These tests compare source execution with direct existing APIs;
the previous independently specialized Rule/Program oracles were rerun too.
Module and real evidence immutability, partial failures and legacy errors remain
covered. Forward/explain output for configured queries is compared against the
same original module with legacy query options.

The standalone source extension is ready for integration review. No merge or
push has occurred. Main and v1 remain unchanged. The stepwise worktree remains
at `e62232a` with its ten tracked audit edits and untracked residual engine file
preserved and unverified. Its eleven-file SHA-256 remains
`f03f2023a65f20f9936063754bc0b3c3a973d6ba59cdc41608cc659772446969`.
No broad retention, forward-driver, incremental or profiling refactor was
transplanted; `src/derive.rs` and its submodules remain unchanged.

**Verdict:** existing inference infrastructure reused; scope/parameter lookup and
the source runner's existing termination check extracted into shared helpers;
new source/scoping/presentation logic only, with no parallel inference machinery.
