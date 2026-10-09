# Constructed temporal values as terms

Implemented only on `experiment-peano-after`, following the explicit temporal
identity correction. Frozen main and v1 remain unchanged. This extends symbolic
terms; it does not add arithmetic evaluation or change the meaning of after.

## Representation and source

`Term::Constructor { constructor: Constructor::After, argument }` is unary.
Its declared signature is exactly PropositionTime -> PropositionTime. The
elaborator checks the input type even inside records and Program bindings.
Neither entities such as Natural nor optional values can be used as its input.

Events now carry SurfaceTerm arguments rather than identifier strings. Existing
identifiers elaborate as before. Formal accepts constructors recursively:

```text
coefficient(polynomial, after(after(zeroTime)), value) @ zeroTime
packed(TimeEnvelope { payload = TimeBox { value = after(value), }, }) @ zeroTime
```

The existing record-term production is shared with event arguments. RuleUse
bindings can also contain these terms. CNL remains paused; its existing identifier
arguments were mechanically adapted to the structured surface representation.
No new CNL syntax was added.

The source syntax for temporal indices is unchanged (`@ t`, `@ after t`). Nested
TimeExpr values still arise internally and can be carried through `@ t`. Rule and
Program applications continue using their existing named-constant argument syntax;
substitution traverses constructor terms already present in their bodies/bindings.

Every repeated event argument or record field consumes a term, and commas are
mandatory inside lists; only the trailing comma is optional. Recursive constructor
parsing consumes the `after(` opener before descending. Event parsing is boxed at
its shared combinator boundary to avoid propagating larger parser types through
all rule/default/experiment branches. The manual comparison parser accepts the
same extension, and SurfaceAST parity was checked for both new fixtures and the
existing fixture collection.

## Structural correspondence and substitution

| TimeExpr | Encoded Term |
|---|---|
| At(name), a constant | Const(name) |
| Var(name), a lambda parameter | Var(name) |
| After(t) | Constructor(After, encode(t)) |

TimeExpr::to_term and TimeExpr::from_term implement these maps. Decoding is
structural: callers must already know that the term has type PropositionTime.
It does not infer a bare constant's declared type from its spelling. Elaborated
events, parameter signatures and Program bindings supply that type guarantee.
Records and none do not decode to temporal expressions.

For valid temporal terms these laws hold by structural induction on the three
cases above:

```text
decode(encode(t)) = t
encode(decode(term)) = term
encode(substitute_time(t, sigma)) = substitute_term(encode(t), sigma)
```

SubstitutionValue still records Term or Time, retaining the representation from
which evidence was matched. Both expose the same structural value to substitution
and compatibility checking. A variable occurring as an argument and an index must
receive compatible values; after(zeroTime) in both positions matches, while two
successors in one position and one in the other does not. Constants are never
substitution binders, including constants homonymous with a remaining parameter.

The tests enumerate 27 expressions (three roots, depths 0 through 8), 54 binding
representations and 1,458 substitution comparisons. They also check nested records
and the homonymous constant/variable distinction. These finite tests supplement the
structural argument; they do not prove general arithmetic or query completeness.

## Matching and beta reduction

Forward and residual matching descend through constructor terms and canonical
record fields. They bind only parameters on the rule-pattern side. Constructor
values being matched must be ground; a target containing pending constructor
variables is rejected. There is no two-sided Prolog unification, occurs-check
search, arithmetic simplification or numeric lookup table.

Rule beta reduction, Program specialization and compile_program traverse nested
constructors/records. Temporal RuleUse bindings decode structurally into TimeExpr,
including `after(t)` while t is still free. Partial application preserves remaining
binders and the existing hygienic namespace of shared/local Program parameters.

## Decisive computation

Source-defined rules are in
`examples/experiments/temporal_constructor_rules.res`, importing `peano_after.res`.
The consumer `temporal_constructor_scenario.res` supplies operand spines and
requests, not calculated answers. Its computed scenario produces:

```text
add(nTwo, nThree, r23) @ zeroTime
  -> sum(r23) @ after^5(zeroTime)
  -> coefficient(polynomial, after^2(zeroTime), after^5(zeroTime)) @ zeroTime
  -> add(nZero, nTwo, r02) @ after^5(zeroTime)
  -> sum(r02) @ after^7(zeroTime)
  -> verified(polynomial, after^7(zeroTime)) @ zeroTime
```

Here `after^n` is documentation shorthand, not source syntax or a primitive.
The second sum uses the already-calculated five as its initial temporal index,
then the existing recursive addition program constructs two more successors.
No coefficient, sum or verified fact is seeded. The goal is only an expectation
for verification; close_program never adds it as evidence.

Other rules pack the computed five into a nested record after constructing two
successors, then structurally recover seven and reuse it as an index. A pattern
`after(value)` extracts four from the computed five. Imported provenance preserves
publish/resume/verify/pack/unpack/peel and the TemporalConstructorRules module;
addition proofs identify RightWalk, finish and PeanoAfter. Regular derivation
records are replayed against their actual witnesses and substitutions.

Constructor-bearing RuleUse bindings and partial applications are compared with
direct complete applications. A seeded inspect value activates the imported Echo
Program; absence of that instance leaves the definitions passive.

This is a value-transfer experiment, not coefficient-wise polynomial addition.
It removes the particular TimeExpr-to-argument boundary that stopped that earlier
experiment. PropositionTime is still not interchangeable with nominal Natural.

## Explicit State values

Value has a symbolic Constructor case mirroring the term. Materialization descends
structurally, including into records, without evaluating after. StateSnapshot and
Store retain their existing structure and Set equality/idempotence semantics.
Execution remains explicit; deriving an Effect never executes it or supplies Act.

A supplementary test stores after^6(zeroTime) inside a nested record using an
Effect whose argument is the computed five. The runtime's existing entity-reference
adapter supplies the concrete shelf ObjectId; the calculated temporal argument is
passed unchanged. The test does not change the existing distinction between named
entity constants and runtime object references. Input State and Known remain intact.

## Bounded verification

34 selected tests passed individually, never as a complete suite:

| Selection | Report under target/test-diagnostics |
|---|---|
| Seven constructor integration/type/beta/provenance/State tests | 20261008T233330909989Z-3621548 |
| Sixteen identity, module, Legal, defaults, effect and Peano regressions | 20261008T233443476381Z-3622831 |
| Eleven conversion, substitution, parser-parity and v1 library regressions | 20261008T233503085650Z-3623103 |

The regressions include all 25 Peano sums, all 25 products (including 4 * 4 = 16),
the sixteen modular Legal closures, and the four sale-default cases. The historical
time-export tests now check successful coefficient/exponent transfer and typed
argument constructors; the nominal Natural type rejection remains in place.

All verification used scripts/test_safe.py: 1 GiB total, no swap, one Cargo job and
one harness thread. An initial library compilation was stopped preventively at
approximately 776.58 MiB cgroup peak, without an OOM and before any test execution.
After boxing the Event parser boundary, a separate compile-only run succeeded:
report 20261008T232741443806Z-3616612, about 755.37 MiB peak RSS and 749.14 MiB cgroup
peak. Final incremental compilation was approximately 419.76 MiB for library tests
and 296.79 MiB for CLI tests. No limits or preventive controls were changed.
Selected tests remained at or below approximately 15.56 MiB peak RSS. Nine CNL
tests remain ignored.
