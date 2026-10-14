# Explicit residual bindings: implementation and reuse review

Worktree: `/tmp/residuum-explicit-query-bindings`, branch
`feat/residual-explicit-bindings`, based on main `9f78f88`. No merge or push.
Main remains `9f78f88`; v1 remains `f6a83e3`.

## Changes in this continuation

The existing implementation was commit `804a995`. This continuation adds:

| Commit | Change |
|---|---|
| `e967fb1` | Empty-binding regressions and delegation; local Unit contract documented first. |
| `448ed0d`, `938c921` | Independent specialization comparisons, full metadata/provenance checks, and original active-closure multiplicity assertions. |
| `d376295` | Separate, narrow RuleUse mapping extraction with targeted tests. |
| `371eee6` | Typed record fields use the source checker's existing assignability policy. |

No synchronous forward driver, `retain_derivation`, incremental driver or
profiling refactor was transplanted. `src/derive.rs` and its submodules are
unchanged from main. CLI/source syntax, defaults, State and ontology files are
unchanged. The CLI integration remains a proposal in
[the API document](residual-explicit-bindings-api.md).

## Actual reuse

Locations below were recorded before the source syntax extension. The current
[source syntax review](residual-query-source-bindings.md#reuse-audit) records
its actual paths and additional shared helpers. Internal paths are included because an
interface wrapper alone does not establish reuse.

| New component | Existing component reused | Actual call path | New logic and why it is necessary |
|---|---|---|---|
| Empty local interface ([bindings.rs:50](../src/residual/bindings.rs#L50)) | Legacy `query_residual` ([query.rs:18](../src/residual/query.rs#L18)) | `query_residual_with_bindings([])` → `query_residual` → `query_with_constraints(..., false)` → `match_goal` / `reduce` | Return envelope only. Preserves legacy omission of satisfied local branches and every legacy witness. |
| Empty Horn/active interfaces ([bindings.rs:80](../src/residual/bindings.rs#L80), [150](../src/residual/bindings.rs#L150)) | Both legacy APIs and their `_with_abducibles` variants | Selected Horn view → `query_program_residual[_with_abducibles]`; active directly → `query_active_program_residual[_with_abducibles]` → guarded search → `certify` | Horn selection is necessary because the interface requires explicit scope. `None` and `Some([])` dispatch separately; no validation or proof transformation is added after selection. |
| Scoped resolver ([bindings.rs:237](../src/residual/bindings.rs#L237)) | `program_rule_origin`, `shared_parameter`, existing `unify_term` → `bind_typed` | `resolve` → producer origin lookup → `rule_use_substitution` for original Rule scope (or shared Program binder) → `query::unify_term` ([207](../src/residual/query.rs#L207)) → `derive::bind_typed` ([671](../src/derive.rs#L671)) | Identify original Rule versus shared Program parameters, detect unknown/out-of-scope/duplicate/conflicting constraints, and avoid global spelling-based substitution. |
| Shared RuleUse mapping ([programs.rs:202](../src/programs.rs#L202)) | Mapping extracted from `compile_program`; existing `programs::substitute_term` and temporal canonicalization | **Compiler:** `compile_program` → `rule_use_substitution` → `elab::substitute_rule_body` ([926](../src/elab.rs#L926)). **Resolver:** `resolve` → the same `rule_use_substitution` → `as_term` → existing unifier | Compiler logic moved, not copied. The resolver needs the compiler's exact mapping for fixed expressions, constructors and hygienic local binders. It previously reconstructed this mapping independently. |
| Typed input boundary ([bindings.rs:349](../src/residual/bindings.rs#L349)) | `rule_type_assignable`, `is_assignable` ([elab.rs:1245](../src/elab.rs#L1245)), `Constructor::signature`, `SubstitutionValue::canonical_for_type` ([typed.rs:555](../src/typed.rs#L555)), `TimeExpr::{to_term,from_term}` | `resolve` → `check_type` / `check_value` / `check_term` → existing assignability predicates and canonical conversion | New structural validation of caller-supplied typed values. Source parsing would risk reclassifying runtime variables as constants or capturing homonymous names. The structural walk itself is new; it is not source elaboration reused wholesale. |
| Constraint merge and local reduction ([query.rs:26](../src/residual/query.rs#L26), [75](../src/residual/query.rs#L75)) | Existing head/event/term/time unifiers, `bind_typed`, `reduce`, `substitute_antecedent`, `substitution_record` | `query_with_constraints` → `constrained_goal` → original `match_goal` → merge via `bind_typed` → original `reduce` → existing typed substitution/recording | Merge after matching the original head so an explicit conflict is an error. `retain_unit` selects the documented local representation; no second reducer. |
| Constrained Horn search ([program.rs:162](../src/residual/program.rs#L162)) | Existing DFS `Search::{event,antecedent,visit,branch}`, admissibility checks and proof replay | `query_program_residual_with_bindings` → `program::query_with_constraints` → `Search::event` ([374](../src/residual/program.rs#L374)) → `constrained_goal` → `derive::substitute_antecedent` → original recursion → `replay_proof` ([285](../src/residual/program.rs#L285)) → `instantiate_from_witness` ([derive.rs:77](../src/derive.rs#L77)) → original forward `match_event` | Supply per-producer constraints and typed unresolved-parameter diagnostics. Limits, branch ordering, proof alternatives and certification execute in the existing engine. No second search or forward matcher. |
| Bound active search and certification ([active.rs:208](../src/residual/active.rs#L208), [111](../src/residual/active.rs#L111)) | Existing `guarded_view`, `activation_steps`, original `close_program` | Bound active API → `guarded_view` → resolver/preflight → same Horn search → extracted `certify` → `close_program` ([derive.rs:192](../src/derive.rs#L192)) → existing program preparation, activation and closure; `activation_steps` records originals | Existing certification loop extracted in `804a995`, now used by both old and bound APIs. Bindings constrain candidate guards; certification still receives the unchanged original module. |

## A concrete query trace

For `suppliedContext` in `explicit_substitutions.res`, select exactly
`obligationCompliance`, `rightCompliance`, `filedObligation`. Bind `conduct =
gConduct`, `actAuthority = private`, `actEffective = gDate` separately in the two
compliance Rule scopes; no Program wrapper is required.

1. `query_program_residual_with_bindings` selects original Rules. `resolve`
   checks module types/constants and canonicalizes `gDate` to
   `Time(At("gDate"))`. Each constraint map is keyed by the original producer.
2. `preflight` calls `constrained_goal`. Existing `match_goal` determines the
   other parameters from the compliance goal. Existing `bind_typed` merges the
   three supplied values and reports conflicts, even if Known contains the goal.
3. The immutable selected view enters `program::query_with_constraints`.
   `Search::event` uses the same constrained match, rejects still-unresolved
   parameters, and calls existing `substitute_antecedent`. For this ordinary Rule
   query, specialization is constraint application to the antecedent, not a
   rewritten Rule or a call to `specialize_program`.
4. Existing DFS reduces the obligation/right and Act requirements, preserving
   tensor order and separate explanations. `substitution_record` records all
   supplied and head-derived values in canonical order.
5. `replay_proof` runs `instantiate_from_witness` against each **original Rule**,
   using the forward matcher's ordered witnesses. Certification can reject a
   syntactic residual that the forward evaluator cannot support. The two
   compliance producers retain their original names and `LegalGeneral` origins.

For Program scope the real beta path is `compile_program` → shared RuleUse
mapping → `elab::substitute_rule_body`; supplied constraints subsequently apply
to those compiled producers without changing Program identity. For active
queries the search additionally includes original activation guards and calls
the original `close_program` after Horn certification.

The hidden-authority regression has two uses of `r` in `P`. Binding
`P.authority = private` requires `starts(item, private) @ tau`; it cannot invent
activation. When Known also contains `starts(item, public) @ tau`, the original
hypothetical closure retains four justifications: two producer uses for each
authority. The candidate remains constrained to private. Applying the binding
globally to that closure would incorrectly discard the public justifications.

## Empty-bindings counterexamples and correction

All four regressions ran and failed before delegation was implemented. The
failure logs remain under `target/test-diagnostics`:

| Counterexample | Legacy expected | Previous empty-binding result | Red report |
|---|---|---|---|
| Local `sales`, `residual_proof.res`, experiment `both` | `Ok([])` | One explicit Unit requirement with evidence/substitutions | `20261014T021458991398Z-966852` |
| Selected compliance Rules, empty Known | Error: `rule 'obligationCompliance' has parameter 'conduct' not determined by the goal` (backticks in the actual message) | Enhanced unresolved typed-parameter error | `20261014T021517187849Z-967445` |
| Sale defaults, ordinary Horn query | Legacy monotonic-program error; invalid limits take precedence | New bound-specific defaults error, checked earlier | `20261014T021542004728Z-967649` |
| Active `P` with authority absent from the goal | Legacy first-unbound-parameter error | Enhanced unresolved typed-parameter error | `20261014T021543428157Z-967675` |

Empty interfaces now delegate without supplying `Some(empty_constraints)` or
enabling local Unit retention. Additional tests compare complete results,
signature errors, invalid/exhausted budgets, default rejection precedence,
explicit hypothesis policy, pending local parameters, metadata, duplicate
witnesses, active guards and full hypothetical provenance.

## Independent comparisons and their limits

`compatibility_tests.rs::specialize_rules` uses **manually specified** typed
values and existing `elab::substitute_rule_body`, removes the fixed binders, then
runs legacy APIs. It never calls the new resolver. Test-only restoration adds
the deliberately eliminated supplied parameters back into oracle substitution
records, so complete original-source proofs can be compared.

The named `ComplianceBound` experiment is compared across all eight evidence
subsets. Test-only `source_proof` first checks each alias/use identity and origin,
then decodes its substitution back to the source Rule. This is an explicit
identity correspondence, not ignoring producer names. A second Program oracle
calls existing `specialize_program` while preserving the same Program identity;
it also tests staged `CompliancePartial` application.

Active tests independently specialize the existing guarded projection, run the
legacy Horn query, restore eliminated substitutions, then certify with the
original module and original templates. This tests binding integration, not an
independent implementation of the activation or closure semantics.

Comparisons use full structural Debug output because `Event::PartialEq` excludes
proposition kind and transition metadata. They include every goal/evidence
event, proposition/state times, canonical substitution, ordered tensor and
antecedent, branch order, metrics, failure, activation origin and justification
multiplicity. Only independently allocated `DerivedEvent.derived_at` values are
normalized. Clock-sensitive empty delegations also reuse the caller's clock.

Explicit conflicting bindings intentionally error even when a separately
specialized incompatible head returns no match/no explanation. Nonempty
recursive unresolved parameters intentionally produce a typed list rather than
the legacy first-parameter error. Local partial results match independent Rule
specialization after restoring supplied values. Satisfied nonempty local Unit
results follow the previously documented adapter contract; legacy local queries
omit them, so the tests check their complete ordered forward witnesses instead
of claiming identical result representations.

## Typed-validation differences

| Difference | Classification and current behavior |
|---|---|
| Typed `Const` versus opaque runtime `Var` | Necessary representation preservation. The checker does not infer identity class from spelling. A caller asserts the runtime variable's type, as with existing typed Event inputs. |
| `SubstitutionValue::Time` and temporal constructors | Necessary typed-value support. Existing canonical conversion and constructor signatures execute; no new time conversion algorithm. |
| Structured value walk | Necessary new boundary validation; source `lower_term` consumes SurfaceTerms and performs lexical name resolution. Existing type predicates are called, but the walk is independently implemented. |
| Record-field entity subtyping | Accidental divergence found and corrected. `check_term` now calls the same `elab::is_assignable` used by source record fields. Root named binding checks retain `rule_type_assignable`. The corrected regression first reproduced prior acceptance. |
| Root subtyping versus positional Program application | Interface policy, **not forced by typed input**. Named bindings follow existing named RuleUse assignability, while `specialize_program` accepts constant strings with exact parameter types. The boundary regression verifies direct/transitive subtype acceptance, supplied-supertype compatibility, rejection of invalid narrowing/unrelated types, and stricter positional application. The paths intentionally differ; subtype equivalence is not claimed. |
| Record field ordering | Already-typed values retain supplied order; source `lower_term` emits declaration order. This preservation choice is not logically forced by typing. The boundary regression verifies positional matching, preserved substitutions, explicit goal/RuleUse conflicts and absence of matches for permuted evidence. Permutations are different typed values; no automatic normalization is performed. |
| Resolved temporal bindings only; depth limit 64; duplicate rejection | Explicit interface policies, not consequences of already-typed input. Omit a binding to retain a pending parameter. No existential enumeration. |
| Nonempty preflight and validation/error ordering | Intentional explicit-constraint policy, including conflicts when Known already contains the goal. Empty bindings bypass it and preserve the exact old path. |

## Duplication and verdict

The resolver's independent RuleUse substitution reconstruction was removed by
the narrow extraction. No new runtime substitution, matching, DFS, closure or
proof-replay algorithm exists. The preexisting `programs::substitute_term`
(term-only specialization) and `derive::substitute_term` (typed runtime
substitution) remain distinct baseline helpers; this work did not create or
unify them.

Two new boundary operations remain: the typed structural validator and
`bindings::default_abducibles`, which repeats the existing seeded/both-polarities
policy enumeration. The latter is policy duplication, not a second inference
path; a future small policy-helper extraction could remove it. Test-only oracle
mapping/restoration is intentionally independent of production scoping and
executes no new inference. No reuse claim is based merely on matching outputs.

* **Existing infrastructure reused:** typed binding/canonical conversion,
  unification, substitution, beta reduction, residual reduction/search,
  forward-witness replay, activation projection, original closure and provenance.
* **Existing infrastructure refactored into shared components:** local/Horn
  constraint entry points and active certification in `804a995`; compiler
  `rule_use_substitution` in `d376295`; existing field-assignability visibility in
  `371eee6`.
* **Parallel machinery introduced:** typed input/scoping validation and the
  duplicated default-abducible enumeration described above. **No parallel
  inference machinery.** The record-order and root-subtyping boundaries are now
  covered by regressions. The interfaces intentionally differ as documented;
  equivalence across those policy boundaries is not claimed.

## Verification and separately preserved stepwise work

Final code checks: **35 named tests passed individually**, sequentially, through
`python3 scripts/test_safe.py`, with 1 GiB cgroups, zero swap, one Cargo job and
one harness thread. `CARGO_PROFILE_TEST_DEBUG=0` was retained; no limit was raised.

* 23 bindings tests: `20261014T023940540244Z-991966`.
* Six legacy library tests: `20261014T024036986645Z-992772`.
* Six CLI tests, including both existing explicit-specialization experiment
  tests: `20261014T024156543040Z-993760`.
* Format check: `20261014T024237989602Z-994804`.

These are focused compatibility results, not a full-suite claim. Initial oracle
test assumptions about diagnostic formatting and active Unit provenance were
corrected after failures; all corrected tests passed in the final run. The first
record-policy fixture had a parser error; the corrected fixture subsequently
reproduced the actual validation discrepancy before the final correction.

`/tmp/residuum-stepwise-query` remains at `e62232a`, with its **ten modified
tracked files** and untracked `src/residual/engine.rs` preserved. These audit edits
contain the broad shared residual engine, retention/driver and related
refactors. They remain uncompiled and unverified; prior committed stepwise test
results do not validate them. This continuation did not edit or test that
worktree. A SHA-256 over those eleven audit files is identical before and after:
`f03f2023a65f20f9936063754bc0b3c3a973d6ba59cdc41608cc659772446969`.

No synchronous loop is required for explicit bindings. The separate stepwise
driver's scheduling and provenance questions remain part of the paused
stepwise review, not assumptions used by this implementation.

## Boundary follow-up and integration readiness

Only two regression functions and documentation were added in this follow-up;
production matching, assignability and inference code is unchanged. The record
fixture's first run failed because a zero-parameter Program definition omitted
`()`; correcting that test fixture produced the passing result below.

* Record order: `20261014T040921511039Z-1055328` — one test passed individually,
  including explicit assertions of Horn and active Unit results.
* Root subtyping: `20261014T040618694605Z-1053144` — one test passed individually.
* Format check: `20261014T040937974778Z-1055802` — passed after formatting.

Both ran through `python3 scripts/test_safe.py` with the same enforced 1 GiB,
zero-swap, one-job, one-thread limits. They use the existing full-structure
comparisons and independent specialization helpers; active comparison ignores
only independently allocated derivedAt values. No other tests were run in this
follow-up. Combined with the previous 35 passing selections, there are 37
distinct selected tests verified, not a full-suite result.

The [API document](residual-explicit-bindings-api.md#record-order-and-root-subtyping-boundaries)
states the precise accepted values, errors and non-matches. No policy change is
proposed for this integration. Normalizing typed record permutations or widening
positional Program application would be a separate semantic change requiring a
proposal before implementation.

**The standalone bindings branch is ready for integration review under these
documented contracts.** The two previously unverified boundaries no longer
require a semantic decision for this change. The later agreed source/CLI
integration is now implemented and covered in the
[source query review](residual-query-source-bindings.md). No merge or push
has occurred. The stepwise worktree remains separately paused, unchanged and
unverified; its broad audit refactors are not prerequisites for this branch.
